use crate::prelude::*;
use gl::types::*;

pub struct SpriteBatch {
    pub pixels_per_unit: f32,
    shader: GLuint,
    proj_loc: GLint,
    tex: GLuint,
    vbo: GLuint,
    buffer: Vec<SpriteVertex>
}

#[derive(Copy, Clone)]
#[repr(C)]
struct SpriteVertex {
    pos: Point2<f32>,
    tex: Point3<f32>,
    color: [u8; 4]
}

impl SpriteBatch {
    pub fn new(shader: GLuint, tex: GLuint) -> Self {
        let mut vbo = 0;
        unsafe {
            gl::GenBuffers(1, &mut vbo);
        }
        SpriteBatch {
            pixels_per_unit: 1.0,
            shader, tex, vbo,
            proj_loc: glutil::get_uniform_location(shader, "proj").unwrap(),
            buffer: vec![]
        }
    }
    
    fn draw_points(&mut self, sprite: &Sprite, points: [Point2<f32>; 4], color: [u8; 4]) {
        let bl_tex = sprite.tex.origin + vec2(0.0, sprite.tex.size.height);
        let tl_tex = sprite.tex.origin + vec2(0.0, 0.0);
        let br_tex = sprite.tex.origin + vec2(sprite.tex.size.width, sprite.tex.size.height);
        let tr_tex = sprite.tex.origin + vec2(sprite.tex.size.width, 0.0);
        let bl = SpriteVertex {
            pos: points[0], color,
            tex: if sprite.rotated { tl_tex } else { bl_tex }.extend(sprite.layer as f32),
        };
        let tl = SpriteVertex {
            pos: points[1], color,
            tex: if sprite.rotated { tr_tex } else { tl_tex }.extend(sprite.layer as f32),
        };
        let br = SpriteVertex {
            pos: points[2], color,
            tex: if sprite.rotated { bl_tex } else { br_tex }.extend(sprite.layer as f32),
        };
        let tr = SpriteVertex {
            pos: points[3], color,
            tex: if sprite.rotated { br_tex } else { tr_tex }.extend(sprite.layer as f32),
        };
        
        self.buffer.push(bl);
        self.buffer.push(tl);
        self.buffer.push(br);
        self.buffer.push(tl);
        self.buffer.push(br);
        self.buffer.push(tr);
    }
    
    pub fn draw_transform(&mut self, sprite: &Sprite, transform: Transform2D<f32>, color: [u8; 4]) {
        self.draw_points(sprite, [
            transform.transform_point(point2(
                -sprite.trimmed_size.width / 2.0 / self.pixels_per_unit,
                -sprite.trimmed_size.height / 2.0 / self.pixels_per_unit
            )),
            transform.transform_point(point2(
                -sprite.trimmed_size.width / 2.0 / self.pixels_per_unit,
                sprite.trimmed_size.height / 2.0 / self.pixels_per_unit
            )),
            transform.transform_point(point2(
                sprite.trimmed_size.width / 2.0 / self.pixels_per_unit,
                -sprite.trimmed_size.height / 2.0 / self.pixels_per_unit
            )),
            transform.transform_point(point2(
                sprite.trimmed_size.width / 2.0 / self.pixels_per_unit,
                sprite.trimmed_size.height / 2.0 / self.pixels_per_unit
            )),
        ], color);
    }
    
    pub fn draw(&mut self, sprite: &Sprite, pos: Point2<f32>, color: [u8; 4]) {
        self.draw_points(sprite, [
            pos + vec2(
                -sprite.trimmed_size.width / 2.0 / self.pixels_per_unit,
                -sprite.trimmed_size.height / 2.0 / self.pixels_per_unit
            ),
            pos + vec2(
                -sprite.trimmed_size.width / 2.0 / self.pixels_per_unit,
                sprite.trimmed_size.height / 2.0 / self.pixels_per_unit
            ),
            pos + vec2(
                sprite.trimmed_size.width / 2.0 / self.pixels_per_unit,
                -sprite.trimmed_size.height / 2.0 / self.pixels_per_unit
            ),
            pos + vec2(
                sprite.trimmed_size.width / 2.0 / self.pixels_per_unit,
                sprite.trimmed_size.height / 2.0 / self.pixels_per_unit
            )
        ], color);
    }
    
    /// Actually draws the queued glyphs.
    /// 
    /// Touches the following OpenGL state:
    /// - `GL_TEXTURE_2D_ARRAY` binding
    /// - `GL_ARRAY_BUFFER` binding
    /// - Current shader program
    /// - Vertex attribute arrays for indices 0, 1, 2
    pub fn render(&mut self, camera: Transform3D<f32>) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, self.tex);
            
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            let data: &[_] = &self.buffer;
            gl::BufferData(
                gl::ARRAY_BUFFER,
                std::mem::size_of_val(data) as isize,
                data.as_ptr() as *const _,
                gl::STREAM_DRAW
            );
            
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 24, 0 as *const _);
            gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, 24, 8 as *const _);
            gl::VertexAttribPointer(2, 4, gl::UNSIGNED_BYTE, gl::TRUE, 24, 20 as *const _);
            gl::EnableVertexAttribArray(0);
            gl::EnableVertexAttribArray(1);
            gl::EnableVertexAttribArray(2);
            
            gl::UseProgram(self.shader);
            let mat = camera.to_row_major_array();
            gl::UniformMatrix4fv(self.proj_loc, 1, gl::FALSE, mat.as_ptr());
            
            gl::DrawArrays(gl::TRIANGLES, 0, self.buffer.len() as i32);
            
            gl::DisableVertexAttribArray(0);
            gl::DisableVertexAttribArray(1);
            gl::DisableVertexAttribArray(2);
        }
        self.buffer.clear();
    }
}

impl Drop for SpriteBatch {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.vbo);
        }
    }
}

pub struct Sprite {
    pub tex: Rect<f32>,
    pub trimmed_size: Size2<f32>,
    pub real_size: Size2<f32>,
    pub layer: f32,
    pub rotated: bool
}

pub fn sprite_shader() -> GLuint {
    glutil::compile_shader_program(
        include_str!("shaders/sprite-vertex.glsl"),
        include_str!("shaders/sprite-fragment.glsl")
    ).unwrap()
}