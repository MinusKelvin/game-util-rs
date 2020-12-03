use crate::prelude::*;
use scopeguard::ScopeGuard;

/// Utility to draw a layer of a tilemap.
pub struct TilemapRenderer {
    gl: Gl,
    shader: glow::Program,
    tilemap: glow::Texture,
    proj_loc: glow::UniformLocation,
    size_loc: glow::UniformLocation,
    offset_loc: glow::UniformLocation,
    tilemap_size_loc: glow::UniformLocation,
    tileset_loc: glow::UniformLocation,
    width: usize,
    height: usize,
}

impl TilemapRenderer {
    /// Creates a new `TilemapRenderer` for a map of the specified size.
    /// 
    /// Touches the following OpenGL state:
    /// - `GL_TEXTURE_2D` binding
    /// - `GL_UNPACK_ALIGNMENT` pixel store parameter
    pub fn new(
        gl: &Gl, shader: glow::Program, width: usize, height: usize, tiles: &[u16]
    ) -> Result<Self, String> {
        if tiles.len() != width * height {
            panic!(
                "Improper tile array length of {} for {}x{} tilemap", tiles.len(), width, height
            );
        }

        unsafe {
            let tilemap = scopeguard::guard(
                gl.create_texture()?,
                |tex| gl.delete_texture(tex)
            );
            gl.bind_texture(glow::TEXTURE_2D, Some(*tilemap));
            gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1);
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::R16UI as i32,
                width as i32, height as i32,
                0,
                glow::RED_INTEGER,
                glow::UNSIGNED_SHORT,
                Some(glutil::as_u8_slice(tiles))
            );
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::NEAREST as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::NEAREST as i32);

            Ok(TilemapRenderer {
                gl: gl.clone(),
                proj_loc: glutil::get_uniform_location(gl, shader, "proj")?,
                size_loc: glutil::get_uniform_location(gl, shader, "size")?,
                offset_loc: glutil::get_uniform_location(gl, shader, "offset")?,
                tilemap_size_loc: glutil::get_uniform_location(gl, shader, "tilemapSize")?,
                tileset_loc: glutil::get_uniform_location(gl, shader, "tileset")?,
                width, height, shader,
                tilemap: ScopeGuard::into_inner(tilemap),
            })
        }
    }

    /// Updates a section of the tilemap.
    /// 
    /// Touches the following OpenGL state:
    /// - `GL_TEXTURE_2D` binding
    /// - `GL_UNPACK_ALIGNMENT` pixel store parameter
    pub fn update(&mut self, x: usize, y: usize, width: usize, height: usize, tiles: &[u16]) {
        if tiles.len() != width * height {
            panic!(
                "Improper tile array length of {} for {}x{} tilemap", tiles.len(), width, height
            );
        }

        if x + width > self.width || y + height > self.height {
            panic!("Tilemap update area outside of tilemap bounds");
        }

        unsafe {
            self.gl.bind_texture(glow::TEXTURE_2D, Some(self.tilemap));
            self.gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1);

            self.gl.tex_sub_image_2d(
                glow::TEXTURE_2D,
                0,
                x as i32, y as i32,
                width as i32, height as i32,
                glow::RED_INTEGER,
                glow::UNSIGNED_SHORT,
                glow::PixelUnpackData::Slice(glutil::as_u8_slice(tiles))
            );
        }
    }

    /// Renders the tilemap using the given tileset.
    /// 
    /// The bottom-left corner of the tilemap is at (0, 0), and the top-right corner is at
    /// (width, height).
    /// 
    /// See also: `Self::render_section`
    pub fn render(&self, camera: Transform3D<f32>, tileset: glow::Texture) {
        self.render_section(camera, tileset, rect(0.0, 0.0, self.width as f32, self.height as f32))
    }
    
    /// Renders the given section of the tilemap using the given tileset.
    /// 
    /// The bottom-left corner of the tilemap section is at (0, 0), and the top-right corner is at
    /// (rect.width, rect.height).
    /// 
    /// Touches the following OpenGL state:
    /// - `GL_TEXTURE_2D` binding
    /// - `GL_TEXTURE_2D_ARRAY` binding
    /// - Active texture (set to 0)
    /// - Current shader program
    pub fn render_section(&self, camera: Transform3D<f32>, tileset: glow::Texture, rect: Rect<f32>) {
        unsafe {
            self.gl.use_program(Some(self.shader));
        
            self.gl.uniform_1_i32(Some(&self.tileset_loc), 1);
            self.gl.uniform_2_f32(Some(&self.size_loc), rect.size.width, rect.size.height);
            self.gl.uniform_2_f32(Some(&self.offset_loc), rect.origin.x, rect.origin.y);
            self.gl.uniform_2_i32(
                Some(&self.tilemap_size_loc), self.width as i32, self.height as i32
            );
            let camera_matrix = camera.to_array();
            self.gl.uniform_matrix_4_f32_slice(Some(&self.proj_loc), false, &camera_matrix);
    
            self.gl.active_texture(glow::TEXTURE1);
            self.gl.bind_texture(glow::TEXTURE_2D_ARRAY, Some(tileset));
            self.gl.active_texture(glow::TEXTURE0);
            self.gl.bind_texture(glow::TEXTURE_2D, Some(self.tilemap));
        
            self.gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);
        }
    }
}

impl Drop for TilemapRenderer {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_texture(self.tilemap);
        }
    }
}

pub fn tilemap_shader(gl: &Gl) -> glow::Program {
    glutil::compile_shader_program(
        gl,
        include_str!("shaders/tilemap-vertex.glsl"),
        include_str!("shaders/tilemap-fragment.glsl")
    ).unwrap()
}