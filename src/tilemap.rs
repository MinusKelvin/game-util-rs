use gl::types::*;
use crate::prelude::*;

/// Utility to draw a layer of a tilemap.
pub struct TilemapRenderer {
    shader: GLuint,
    tilemap: GLuint,
    proj_loc: GLint,
    size_loc: GLint,
    width: usize,
    height: usize,
}

impl TilemapRenderer {
    /// Creates a new `TilemapRenderer` for a map of the specified size.
    /// 
    /// Touches the following OpenGL state:
    /// - `GL_TEXTURE_2D` binding
    /// - `GL_UNPACK_ALIGNMENT` pixel store parameter
    pub fn new(width: usize, height: usize, tiles: &[u16]) -> Self {
        if tiles.len() != width * height {
            panic!(
                "Improper tile array length of {} for {}x{} tilemap", tiles.len(), width, height
            );
        }

        let mut tilemap = 0;
        unsafe {
            gl::GenTextures(1, &mut tilemap);
            gl::BindTexture(gl::TEXTURE_2D, tilemap);
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::R16UI as _,
                width as GLsizei, height as GLsizei,
                0,
                gl::RED_INTEGER,
                gl::UNSIGNED_SHORT,
                tiles.as_ptr() as _
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as _);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as _);
        }

        let shader = glutil::compile_shader_program(
            include_str!("shaders/tilemap-vertex.glsl"),
            include_str!("shaders/tilemap-fragment.glsl")
        ).unwrap();

        TilemapRenderer {
            width, height, shader, tilemap,
            proj_loc: glutil::get_uniform_location(shader, "proj").unwrap(),
            size_loc: glutil::get_uniform_location(shader, "size").unwrap()
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
            gl::BindTexture(gl::TEXTURE_2D, self.tilemap);
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);

            gl::TexSubImage2D(
                gl::TEXTURE_2D,
                0,
                x as GLint, y as GLint,
                width as GLsizei, height as GLsizei,
                gl::RED_INTEGER,
                gl::UNSIGNED_SHORT,
                tiles.as_ptr() as _
            );
        }
    }

    /// Renders the tilemap using the given tileset.
    /// 
    /// The bottom-left corner of the tilemap is at (0, 0), and the top-right corner is at
    /// (width, height).
    /// 
    /// Touches the following OpenGL state:
    /// - `GL_TEXTURE_2D` binding
    /// - `GL_TEXTURE_2D_ARRAY` binding
    /// - Active texture
    /// - Current shader program
    pub fn render(&self, camera: Transform3D<f32>, tileset: GLuint) {
        unsafe {
            gl::UseProgram(self.shader);

            gl::Uniform1i(glutil::get_uniform_location(self.shader, "tileset").unwrap(), 1);
            gl::Uniform2f(self.size_loc, self.width as f32, self.height as f32);
            let camera_matrix = camera.to_row_major_array();
            gl::UniformMatrix4fv(self.proj_loc, 1, gl::FALSE, camera_matrix.as_ptr());

            gl::BindTexture(gl::TEXTURE_2D, self.tilemap);
            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, tileset);
            gl::ActiveTexture(gl::TEXTURE0);

            gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
        }
    }
}