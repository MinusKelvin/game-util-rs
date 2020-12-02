use rusttype::*;
use rusttype::gpu_cache::*;
use gl::types::*;
use crate::prelude::*;

pub struct TextRenderer {
    styles: Vec<Vec<(usize, Font<'static>, f32)>>,
    cache: gpu_cache::Cache<'static>,
    render_queue: Vec<(PositionedGlyph<'static>, usize, [u8; 4])>,
    tex: GLuint,
    vbo: GLuint,
    vbo_buf: Vec<TextVertex>,
    tex_size: i32,
    next_id: usize,
    shader: GLuint,
    proj_loc: GLint,

    pub dpi: f32,
    pub screen_size: (f32, f32)
}

impl TextRenderer {
    /// Constructor.
    /// 
    /// Touches the following OpenGL state:
    /// - `GL_TEXTURE_2D` binding
    pub fn new() -> TextRenderer {
        let tex_size = 512;

        let mut tex = 0;
        let mut vbo = 0;
        let cache;
        let shader;
        let proj_loc;

        unsafe {
            gl::GenBuffers(1, &mut vbo);

            gl::GenTextures(1, &mut tex);
            gl::BindTexture(gl::TEXTURE_2D, tex);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAX_LEVEL, 0);
            cache = allocate(Cache::builder(), tex_size);

            shader = crate::glutil::compile_shader_program(
                include_str!("shaders/text-vertex.glsl"),
                include_str!("shaders/text-fragment.glsl")
            ).unwrap();
            proj_loc = crate::glutil::get_uniform_location(shader, "proj").unwrap();
        }

        TextRenderer {
            styles: vec![],
            cache,
            render_queue: vec![],
            tex, vbo,
            vbo_buf: vec![],
            tex_size,
            next_id: 0,
            shader,
            proj_loc,
            dpi: 1.0,
            screen_size: (0.0, 0.0)
        }
    }

    /// Actually draws the queued glyphs.
    /// 
    /// Touches the following OpenGL state:
    /// - `GL_TEXTURE_2D` binding
    /// - `GL_ARRAY_BUFFER` binding
    /// - `GL_UNPACK_ALIGNMENT` pixel store parameter
    /// - Current shader program
    /// - Vertex attribute arrays for indices 0, 1, 2
    pub fn render(&mut self) {
        for (glyph, font_id, _) in self.render_queue.iter().cloned() {
            self.cache.queue_glyph(font_id, glyph);
        }

        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.tex);
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);

            while let Err(_) = self.cache.cache_queued(|rect, data| gl::TexSubImage2D(
                gl::TEXTURE_2D,
                0,
                rect.min.x as i32, rect.min.y as i32,
                rect.width() as i32, rect.height() as i32,
                gl::RED,
                gl::UNSIGNED_BYTE,
                data.as_ptr() as *const _
            )) {
                self.tex_size *= 2;
                self.cache = allocate(self.cache.to_builder(), self.tex_size);
                for (glyph, font_id, _) in self.render_queue.iter().cloned() {
                    self.cache.queue_glyph(font_id, glyph);
                }
            }
        }

        self.vbo_buf.clear();

        for &(ref glyph, font_id, color) in self.render_queue.iter() {
            if let Some((tex, pix)) = self.cache.rect_for(font_id, glyph).unwrap() {
                self.vbo_buf.push(TextVertex {
                    pos: vec2(pix.min.x as f32, pix.min.y as f32),
                    tex: vec2(tex.min.x, tex.min.y),
                    color
                });
                self.vbo_buf.push(TextVertex {
                    pos: vec2(pix.min.x as f32, pix.max.y as f32),
                    tex: vec2(tex.min.x, tex.max.y),
                    color
                });
                self.vbo_buf.push(TextVertex {
                    pos: vec2(pix.max.x as f32, pix.min.y as f32),
                    tex: vec2(tex.max.x, tex.min.y),
                    color
                });

                self.vbo_buf.push(TextVertex {
                    pos: vec2(pix.max.x as f32, pix.min.y as f32),
                    tex: vec2(tex.max.x, tex.min.y),
                    color
                });
                self.vbo_buf.push(TextVertex {
                    pos: vec2(pix.min.x as f32, pix.max.y as f32),
                    tex: vec2(tex.min.x, tex.max.y),
                    color
                });
                self.vbo_buf.push(TextVertex {
                    pos: vec2(pix.max.x as f32, pix.max.y as f32),
                    tex: vec2(tex.max.x, tex.max.y),
                    color
                });
            }
        }

        unsafe {
            let data: &[TextVertex] = &self.vbo_buf;
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                std::mem::size_of_val(data) as isize,
                data.as_ptr() as *const _,
                gl::STREAM_DRAW
            );
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 20, 0 as *const _);
            gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, 20, 8 as *const _);
            gl::VertexAttribPointer(2, 4, gl::UNSIGNED_BYTE, gl::TRUE, 20, 16 as *const _);
            gl::EnableVertexAttribArray(0);
            gl::EnableVertexAttribArray(1);
            gl::EnableVertexAttribArray(2);

            gl::UseProgram(self.shader);

            let mat = euclid::default::Transform3D::ortho(
                0.0, self.screen_size.0 * self.dpi,
                self.screen_size.1 * self.dpi, 0.0,
                -1.0, 1.0
            ).to_array();
            gl::UniformMatrix4fv(self.proj_loc, 1, gl::FALSE, mat.as_ptr());

            gl::DrawArrays(gl::TRIANGLES, 0, data.len() as i32);
            
            gl::DisableVertexAttribArray(0);
            gl::DisableVertexAttribArray(1);
            gl::DisableVertexAttribArray(2);
        }

        self.render_queue.clear();
    }

    pub fn add_style(&mut self, fonts: impl IntoIterator<Item=Font<'static>>) -> usize {
        let index = self.styles.len();
        self.styles.push(vec![]);
        for font in fonts {
            self.add_fallback_font(index, font);
        }
        if self.styles[index].is_empty() {
            panic!("Must have at least one font to add a font style!")
        }
        index
    }

    pub fn add_fallback_font(&mut self, style: usize, font: Font<'static>) {
        if style >= self.styles.len() {
            panic!("Invalid style {} (there are {} styles)", style, self.styles.len());
        }
        let cap_height = font.glyph('N')
            .scaled(Scale::uniform(1.0))
            .exact_bounding_box().unwrap()
            .height();
        let height = cap_height - font.v_metrics(Scale::uniform(1.0)).descent;
        self.styles[style].push((self.next_id, font, 1.0 / height));
        self.next_id += 1;
    }

    /// Lays out and measures text.
    pub fn layout(&self, text: &str, size: f32, style: usize) -> LaidOutText {
        if style >= self.styles.len() {
            panic!("Invalid style {} (there are {} styles)", style, self.styles.len());
        }

        let mut prev_glyph = None;
        let mut left_side_bearing = None;
        let mut position = 0.0;
        let mut glyphs = vec![];

        for chr in text.chars() {
            let (font_id, font, glyph, relative_scale) = pick_font(&self.styles[style], chr);
            let scale = Scale::uniform(size * relative_scale);

            let glyph = glyph.scaled(scale);
            glyphs.push((position, Glyph {
                glyph: glyph.clone(), font_id
            }));

            let hmetrics = glyph.h_metrics();
            position += hmetrics.advance_width;

            if left_side_bearing.is_none() {
                left_side_bearing = Some(hmetrics.left_side_bearing);
            }

            position += match prev_glyph {
                Some((fid, id)) if fid == font_id => font.pair_kerning(scale, id, glyph.id()),
                _ => 0.0
            };

            prev_glyph = Some((font_id, glyph.id()));
        }

        LaidOutText {
            width: position,
            left_side_bearing: left_side_bearing.unwrap_or(0.0),
            vertical: self.styles[style][0].1.v_metrics(
                Scale::uniform(size * self.styles[style][0].2)
            ),
            glyphs
        }
    }

    pub fn draw_glyph(&mut self, x: f32, y: f32, color: [u8; 4], glyph: Glyph) {
        let x = x * self.dpi;
        let y = (self.screen_size.1 - y) * self.dpi;
        let scale = glyph.glyph.scale();
        let scale = Scale {
            x: scale.x * self.dpi,
            y: scale.y * self.dpi
        };
        let rt_glyph = glyph.glyph.into_unscaled()
            .scaled(scale)
            .positioned(Point { x, y });
        self.render_queue.push((rt_glyph, glyph.font_id, color));
    }

    pub fn draw_text(
        &mut self,
        text: &str,
        x: f32, mut y: f32,
        alignment: Alignment,
        color: [u8; 4], size: f32, style: usize
    ) {
        for line in text.lines() {
            let LaidOutText {
                vertical, glyphs, width, ..
            } = self.layout(line, size, style);

            let x = match alignment {
                Alignment::Left => x,
                Alignment::Center => x - width / 2.0,
                Alignment::Right => x - width,
            };

            for (offset, glyph) in glyphs {
                self.draw_glyph(x + offset, y, color, glyph);
            }

            y -= vertical.ascent - vertical.descent + vertical.line_gap;
        }
    }
}

impl Drop for TextRenderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.tex);
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteProgram(self.shader)
        }
    }
}

unsafe fn allocate(builder: CacheBuilder, tex_size: i32) -> Cache<'static> {
    gl::TexImage2D(
        gl::TEXTURE_2D,
        0,
        gl::R8 as i32,
        tex_size, tex_size,
        0,
        gl::RED,
        gl::UNSIGNED_BYTE,
        0 as *const _
    );
    builder.dimensions(tex_size as u32, tex_size as u32).build()
}

fn pick_font<'a>(
    fonts: &'a [(usize, Font<'static>, f32)],
    chr: char
) -> (usize, &'a Font<'static>, rusttype::Glyph<'static>, f32) {
    for &(id, ref font, relative_scale) in fonts {
        let glyph = font.glyph(chr);
        if glyph.id().0 != 0 {
            return (id, font, glyph, relative_scale)
        }
    }

    let first = fonts.first().unwrap();
    (first.0, &first.1, first.1.glyph(chr), 1.0)
}

#[repr(C)]
struct TextVertex {
    pos: Vec2<f32>,
    tex: Vec2<f32>,
    color: [u8; 4]
}

#[derive(Clone)]
pub struct Glyph {
    glyph: ScaledGlyph<'static>,
    font_id: usize
}

#[derive(Clone)]
pub struct LaidOutText {
    pub width: f32,
    pub left_side_bearing: f32,
    pub vertical: VMetrics,
    pub glyphs: Vec<(f32, Glyph)>
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Alignment {
    Left, Center, Right
}