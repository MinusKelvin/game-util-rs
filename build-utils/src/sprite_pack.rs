use image::{GenericImage, RgbaImage};
use regex::Regex;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::ops::Deref;
use std::path::Path;
use texture_packer::exporter::ImageExporter;
use texture_packer::importer::ImageImporter;
use texture_packer::{MultiTexturePacker, Rect, TexturePackerConfig};

pub fn gen_sprites(root: impl AsRef<Path>, target: impl AsRef<Path>, size: u32) {
    let mut packer = MultiTexturePacker::new_skyline(TexturePackerConfig {
        max_width: size,
        max_height: size,
        ..Default::default()
    });

    let mut entries = HashMap::new();

    let root = root.as_ref();
    println!("cargo:rerun-if-changed={}", root.display());
    process_dir(&mut entries, &mut packer, root, None);

    let target = target.as_ref();
    std::fs::create_dir_all(target).unwrap();
    for (i, page) in packer.get_pages().iter().enumerate() {
        let img = ImageExporter::export(page).unwrap();
        img.save(target.join(&format!("{}.png", i))).unwrap();
    }

    for (name, k) in &entries {
        match k {
            Kind::Array(v) => {
                for (i, o) in v.iter().enumerate() {
                    if o.is_none() {
                        panic!("index {} of sprite array {} is missing", i, name);
                    }
                }
            }
            Kind::Just(_) => {}
        }
    }

    let mut sprites = BufWriter::new(File::create(target.join("sprites.rs")).unwrap());

    write!(
        sprites,
        "mod sprites {{
        use game_util::Sprite;
        use game_util::prelude::*;
        use game_util::image;
        pub struct Sprites {{"
    )
    .unwrap();

    for (name, kind) in &entries {
        match kind {
            Kind::Just(_) => write!(sprites, "pub {}: Sprite,", name).unwrap(),
            Kind::Array(v) => write!(sprites, "pub {}: [Sprite; {}],", name, v.len()).unwrap(),
        }
    }

    writeln!(sprites, "}}").unwrap();

    write!(
        sprites,
        r#"
        impl Sprites {{
            pub async fn load(gl: &Gl, base: &str) -> Result<(Self, glow::Texture), String> {{
                let tex;
                unsafe {{
                    tex = gl.create_texture()?;
                    gl.bind_texture(glow::TEXTURE_2D_ARRAY, Some(tex));
                    gl.tex_image_3d(
                        glow::TEXTURE_2D_ARRAY,
                        0,
                        glow::RGBA8 as _,
                        {}, {0}, {},
                        0,
                        glow::RGBA,
                        glow::UNSIGNED_BYTE,
                        None
                    );
                    gl.tex_parameter_i32(
                        glow::TEXTURE_2D_ARRAY, glow::TEXTURE_MIN_FILTER, glow::LINEAR as _
                    );
        "#,
        size,
        packer.get_pages().len()
    )
    .unwrap();

    writeln!(sprites, "game_util::futures_util::join!(").unwrap();
    for i in 0..packer.get_pages().len() {
        writeln!(
            sprites,
            "game_util::gltuil::load_texture_layer(
                gl, base.to_owned() + \"/{i}.png\", tex, {size}, {size}, {i}
            ),",
            i = i,
            size = size,
        ).unwrap();
    }
    writeln!(sprites, ");").unwrap();

    write!(sprites, "}} Ok((Sprites {{").unwrap();

    for (name, kind) in &entries {
        write!(sprites, "{}: ", name).unwrap();
        match kind {
            Kind::Just(data) => write_sprite(&mut sprites, data, size),
            Kind::Array(v) => {
                write!(sprites, "[").unwrap();
                for data in v {
                    write_sprite(&mut sprites, data.as_ref().unwrap(), size);
                }
                write!(sprites, "],").unwrap();
            }
        }

        fn write_sprite(sprites: &mut impl Write, data: &Data, size: u32) {
            write!(
                sprites,
                "Sprite {{\
                    tex: rect({}.0 / {}.0, {}.0 / {1}.0, {}.0 / {1}.0, {}.0 / {1}.0),\
                    trimmed_size: size2({}.0, {}.0),\
                    real_size: size2({}.0, {}.0),\
                    layer: {}.0,\
                    rotated: {}\
                }},",
                data.tex.x,
                size,
                data.tex.y,
                data.tex.w,
                data.tex.h,
                if data.rotated { data.tex.h } else { data.tex.w },
                if data.rotated { data.tex.w } else { data.tex.h },
                data.real_size.0,
                data.real_size.1,
                data.layer,
                data.rotated
            )
            .unwrap();
        }
    }

    write!(sprites, "}}, tex))}}}}}}").unwrap();
}

fn process_dir(
    entries: &mut HashMap<String, Kind>,
    packer: &mut MultiTexturePacker<RgbaImage>,
    path: &Path,
    field_name: Option<String>,
) {
    for entry in std::fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        println!("cargo:rerun-if-changed={}", entry.path().display());
        let t = entry.file_type().unwrap();
        let file_name = entry.file_name();
        let (name, array) = process_name(
            field_name.as_ref().map(Deref::deref),
            &file_name.to_string_lossy(),
        );

        if t.is_dir() {
            process_dir(entries, packer, &entry.path(), Some(name));
        } else if t.is_file() {
            let key = match array {
                Some(i) => format!("{}[{}]", name, i),
                None => name.clone(),
            };
            let data = process_img(packer, &key, &entry.path());

            if let Some(i) = array {
                let v = entries.entry(name.clone()).or_insert(Kind::Array(vec![]));
                match v {
                    Kind::Array(v) => {
                        while v.len() <= i {
                            v.push(None);
                        }
                        if v[i].is_some() {
                            panic!("??? two of the same index?");
                        }
                        v[i] = Some(data);
                    }
                    Kind::Just(_) => panic!("mixing sprite and array of sprites at {}", name),
                }
            } else {
                match entries.entry(name.clone()) {
                    Entry::Occupied(_) => {
                        panic!("there's already a sprite called {}", name);
                    }
                    Entry::Vacant(e) => {
                        e.insert(Kind::Just(data));
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
enum Kind {
    Just(Data),
    Array(Vec<Option<Data>>),
}

#[derive(Debug)]
struct Data {
    tex: Rect,
    real_size: (u32, u32),
    layer: usize,
    rotated: bool,
}

fn process_name(parent_name: Option<&str>, name: &str) -> (String, Option<usize>) {
    lazy_static::lazy_static! {
        static ref REGEX: Regex = Regex::new(r"^([_a-zA-Z][_\w]*)(?:.(\d+))?\.\w+$").unwrap();
    };

    match REGEX.captures(name) {
        Some(caps) => {
            let name = caps.get(1).unwrap().as_str();
            let name = match parent_name {
                Some(p) => format!("{}_{}", p, name),
                None => name.to_owned(),
            };
            let index = caps.get(2).map(|m| m.as_str().parse().unwrap());
            (name, index)
        }
        None => panic!("invalid name: {}", name),
    }
}

fn process_img(packer: &mut MultiTexturePacker<RgbaImage>, key: &str, path: &Path) -> Data {
    let mut img = ImageImporter::import_from_file(path).unwrap().to_rgba();

    let width = img.width();
    let height = img.height();

    let mut add_top_border = false;
    let mut add_bottom_border = false;
    for x in 0..width {
        if img.get_pixel(x, 0).0[3] != 0 {
            add_top_border = true;
        }
        if img.get_pixel(x, height - 1).0[3] != 0 {
            add_bottom_border = true;
        }
    }

    let mut add_left_border = false;
    let mut add_right_border = false;
    for y in 0..height {
        if img.get_pixel(0, y).0[3] != 0 {
            add_left_border = true;
        }
        if img.get_pixel(width - 1, y).0[3] != 0 {
            add_right_border = true;
        }
    }

    if add_right_border || add_left_border || add_top_border || add_bottom_border {
        let new_w = add_left_border as u32 + add_right_border as u32 + width;
        let new_h = add_top_border as u32 + add_bottom_border as u32 + height;
        let offset_x = add_left_border as u32;
        let offset_y = add_top_border as u32;

        let base = img;
        img = RgbaImage::new(new_w, new_h);

        img.copy_from(&base, offset_x, offset_y).unwrap();

        for x in 0..width {
            if add_top_border {
                img.put_pixel(offset_x + x, 0, *base.get_pixel(x, 0));
            }
            if add_bottom_border {
                img.put_pixel(
                    offset_x + x,
                    img.height() - 1,
                    *base.get_pixel(x, height - 1),
                );
            }
        }

        for y in 0..height {
            if add_left_border {
                img.put_pixel(0, offset_y + y, *base.get_pixel(0, y));
            }
            if add_right_border {
                img.put_pixel(img.width() - 1, offset_y + y, *base.get_pixel(width - 1, y));
            }
        }

        if add_left_border && add_top_border {
            img.put_pixel(0, 0, *base.get_pixel(0, 0));
        }
        if add_left_border && add_bottom_border {
            img.put_pixel(0, img.height() - 1, *base.get_pixel(0, height - 1));
        }
        if add_right_border && add_top_border {
            img.put_pixel(img.width() - 1, 0, *base.get_pixel(width - 1, 0));
        }
        if add_right_border && add_bottom_border {
            img.put_pixel(
                img.width() - 1,
                img.height() - 1,
                *base.get_pixel(width - 1, height - 1),
            );
        }
    }

    packer.pack_own(key.to_string(), img).unwrap();
    let mut frame = None;
    for (i, page) in packer.get_pages().iter().enumerate() {
        if let Some(f) = page.get_frame(&key) {
            frame = Some((i, f));
        }
    }
    let (layer, frame) = frame.unwrap();

    let mut data = Data {
        tex: frame.frame,
        real_size: (width, height),
        layer,
        rotated: frame.rotated,
    };

    if add_top_border {
        data.tex.h -= 1;
        data.tex.y += 1;
    }
    if add_bottom_border {
        data.tex.h -= 1;
    }

    if add_left_border {
        data.tex.w -= 1;
        data.tex.x += 1;
    }
    if add_right_border {
        data.tex.w -= 1;
    }

    data
}
