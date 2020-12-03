use crate::prelude::*;

pub struct SpriteBatch {
    pub pixels_per_unit: f32,
    gl: Gl,
    shader: glow::Shader,
    proj_loc: glow::UniformLocation,
    tex: glow::Texture,
    vbo: glow::Buffer,
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
    pub fn new(gl: &Gl, shader: glow::Shader, tex: glow::Shader) -> Result<Self, String> {
        let proj_loc = glutil::get_uniform_location(gl, shader, "proj")?;
        let vbo = unsafe { gl.create_buffer()? };
        Ok(SpriteBatch {
            gl: gl.clone(),
            pixels_per_unit: 1.0,
            shader, tex, vbo,
            proj_loc,
            buffer: vec![]
        })
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
    /// Touches the following Open_GL state:
    /// - `GL_TEXTURE_2D_ARRAY` binding
    /// - `GL_ARRAY_BUFFER` binding
    /// - Current shader program
    /// - Vertex attribute arrays for indices 0, 1, 2
    pub fn render(&mut self, camera: Transform3D<f32>) {
        unsafe {
            self.gl.bind_texture(glow::TEXTURE_2D_ARRAY, Some(self.tex));
            
            self.gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
            self.gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                glutil::as_u8_slice(&self.buffer),
                glow::STREAM_DRAW
            );
            
            self.gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 24, 0);
            self.gl.vertex_attrib_pointer_f32(1, 3, glow::FLOAT, false, 24, 8);
            self.gl.vertex_attrib_pointer_f32(2, 4, glow::UNSIGNED_BYTE, true, 24, 20);
            self.gl.enable_vertex_attrib_array(0);
            self.gl.enable_vertex_attrib_array(1);
            self.gl.enable_vertex_attrib_array(2);
            
            self.gl.use_program(Some(self.shader));
            let mat = camera.to_array();
            self.gl.uniform_matrix_4_f32_slice(Some(&self.proj_loc), false, &mat);
            
            self.gl.draw_arrays(glow::TRIANGLES, 0, self.buffer.len() as i32);
            
            self.gl.disable_vertex_attrib_array(0);
            self.gl.disable_vertex_attrib_array(1);
            self.gl.disable_vertex_attrib_array(2);
        }
        self.buffer.clear();
    }
}

impl Drop for SpriteBatch {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_buffer(self.vbo);
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

pub fn sprite_shader(gl: &Gl) -> glow::Program {
    glutil::compile_shader_program(
        gl,
        include_str!("shaders/sprite-vertex.glsl"),
        include_str!("shaders/sprite-fragment.glsl")
    ).unwrap()
}