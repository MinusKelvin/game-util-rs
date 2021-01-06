use std::path::PathBuf;

use crate::prelude::*;
use futures::channel::oneshot;

#[derive(Clone)]
pub struct Gl(std::rc::Rc<glow::Context>);

impl Gl {
    pub(crate) fn new(gl: glow::Context) -> Self {
        Gl(std::rc::Rc::new(gl))
    }
}

impl std::ops::Deref for Gl {
    type Target = glow::Context;
    fn deref(&self) -> &glow::Context {
        &self.0
    }
}

async fn load_image(source: &str) -> Result<image::RgbaImage, String> {
    let (s, r) = oneshot::channel();
    let source = PathBuf::from(source);
    std::thread::spawn(|| {
        s.send(image::open(source)).ok();
    });
    r.await
        .unwrap()
        .map_err(|e| e.to_string())
        .map(|img| img.to_rgba8())
}

pub async fn load_texture_2d(gl: &Gl, source: &str) -> Result<glow::Texture, String> {
    let image = load_image(source).await?;
    unsafe {
        let texture = gl.create_texture()?;
        gl.bind_texture(glow::TEXTURE_2D, Some(texture));
        gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            glow::RGBA8 as i32,
            image.width() as i32,
            image.height() as i32,
            0,
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            Some(&image),
        );
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAX_LEVEL, 0);

        Ok(texture)
    }
}

pub async fn load_texture_layer(
    gl: &Gl,
    source: &str,
    texture: glow::Texture,
    layer: i32,
) -> Result<(), String> {
    let image = load_image(source).await?;
    unsafe {
        gl.bind_texture(glow::TEXTURE_2D_ARRAY, Some(texture));
        gl.tex_sub_image_3d(
            glow::TEXTURE_2D_ARRAY,
            0,
            0,
            0,
            layer,
            image.width() as i32,
            image.height() as i32,
            1,
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            glow::PixelUnpackData::Slice(&image),
        );
    }
    Ok(())
}
