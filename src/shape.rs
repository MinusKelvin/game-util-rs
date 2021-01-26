use crate::prelude::*;

use lyon_tessellation::path::Path;
use lyon_tessellation::{
    BuffersBuilder, FillOptions, FillTessellator, FillVertex, StrokeOptions, StrokeTessellator,
    StrokeVertex, TessellationError, VertexBuffers,
};

pub struct ShapeRenderer {
    pub pixels_per_unit: f32,
    gl: Gl,
    buffers: VertexBuffers<ShapeVertex, u32>,
    vbo: glow::Buffer,
    ibo: glow::Buffer,
    shader: glow::Program,
    proj_loc: glow::UniformLocation,
}

impl ShapeRenderer {
    pub fn new(gl: &Gl, shader: glow::Program) -> Result<Self, String> {
        let (vbo, ibo, proj_loc);
        unsafe {
            vbo = gl.create_buffer()?;
            ibo = gl.create_buffer()?;
            proj_loc = glutil::get_uniform_location(gl, shader, "proj")?;
        }
        Ok(ShapeRenderer {
            pixels_per_unit: 1.0,
            gl: gl.clone(),
            buffers: VertexBuffers::new(),
            vbo,
            ibo,
            shader,
            proj_loc,
        })
    }

    pub fn fill_path(&mut self, path: &Path, color: [u8; 4]) -> Result<(), TessellationError> {
        let mut builder =
            BuffersBuilder::new(&mut self.buffers, |vertex: FillVertex| ShapeVertex {
                pos: vertex.position(),
                color,
            });

        let mut tessellator = FillTessellator::new();
        tessellator.tessellate_path(
            path,
            &FillOptions::tolerance(self.pixels_per_unit * FillOptions::DEFAULT_TOLERANCE),
            &mut builder,
        )?;

        Ok(())
    }

    pub fn stroke_path(
        &mut self,
        path: &Path,
        thickness: f32,
        color: [u8; 4],
    ) -> Result<(), TessellationError> {
        let mut builder =
            BuffersBuilder::new(&mut self.buffers, |vertex: StrokeVertex| ShapeVertex {
                pos: vertex.position(),
                color: color,
            });

        let mut tessellator = StrokeTessellator::new();
        tessellator.tessellate_path(
            path,
            &StrokeOptions::tolerance(self.pixels_per_unit * StrokeOptions::DEFAULT_TOLERANCE)
                .with_line_width(thickness),
            &mut builder,
        )?;

        Ok(())
    }

    /// Actually draws the queued shapes.
    ///
    /// Touches the following Open_GL state:
    /// - `GL_ARRAY_BUFFER` binding
    /// - `GL_ELEMENT_ARRAY_BUFFER` binding
    /// - Current shader program
    /// - Vertex attribute arrays for indices 0, 1
    pub fn render(&mut self, camera: Transform3D<f32>) {
        unsafe {
            self.gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
            self.gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                glutil::as_u8_slice(&self.buffers.vertices),
                glow::STREAM_DRAW,
            );
            self.gl
                .bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.ibo));
            self.gl.buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                glutil::as_u8_slice(&self.buffers.indices),
                glow::STREAM_DRAW,
            );

            self.gl
                .vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 12, 0);
            self.gl
                .vertex_attrib_pointer_f32(1, 4, glow::UNSIGNED_BYTE, true, 12, 8);
            self.gl.enable_vertex_attrib_array(0);
            self.gl.enable_vertex_attrib_array(1);

            self.gl.use_program(Some(self.shader));
            let mat = camera.to_array();
            self.gl
                .uniform_matrix_4_f32_slice(Some(&self.proj_loc), false, &mat);

            self.gl.draw_elements(
                glow::TRIANGLES,
                self.buffers.indices.len() as i32,
                glow::UNSIGNED_INT,
                0,
            );

            self.gl.disable_vertex_attrib_array(0);
            self.gl.disable_vertex_attrib_array(1);
        }
        self.buffers.indices.clear();
        self.buffers.vertices.clear();
    }
}

impl Drop for ShapeRenderer {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_buffer(self.vbo);
            self.gl.delete_buffer(self.ibo);
        }
    }
}

#[repr(C)]
struct ShapeVertex {
    pos: Point2<f32>,
    color: [u8; 4],
}

pub fn shape_shader(gl: &Gl) -> glow::Program {
    glutil::compile_shader_program(
        gl,
        include_str!("shaders/shape.vert.glsl"),
        include_str!("shaders/shape.frag.glsl"),
    )
    .unwrap()
}
