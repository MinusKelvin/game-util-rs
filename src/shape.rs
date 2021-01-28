use crate::prelude::*;

pub struct ShapeRenderer {
    pub pixels_per_unit: f32,
    gl: Gl,
    vertices: Vec<ShapeVertex>,
    indices: Vec<u32>,
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
            vertices: vec![],
            indices: vec![],
            vbo,
            ibo,
            shader,
            proj_loc,
        })
    }

    pub fn convex_polygon(&mut self, points: &[Point2<f32>], color: [u8; 4]) {
        assert!(points.len() >= 3);
        let zero_index = self.vertices.len() as u32;
        self.vertices
            .extend(points.iter().map(|&pos| ShapeVertex { pos, color }));
        self.indices.reserve((points.len() - 2) * 3);
        for i in 2..points.len() as u32 {
            self.indices.push(zero_index);
            self.indices.push(zero_index + i - 1);
            self.indices.push(zero_index + i);
        }
    }

    pub fn rectangle(&mut self, rect: Rect<f32>, color: [u8; 4]) {
        self.convex_polygon(
            &[
                rect.min(),
                point2(rect.min_x(), rect.max_y()),
                rect.max(),
                point2(rect.max_x(), rect.min_y()),
            ],
            color,
        )
    }

    pub fn line(&mut self, from: Point2<f32>, to: Point2<f32>, thickness: f32, color: [u8; 4]) {
        let direction = (to - from).normalize();
        let normal = vec2(-direction.y, direction.x) * thickness / 2.0;
        self.convex_polygon(
            &[from - normal, from + normal, to + normal, to - normal],
            color,
        )
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
                glutil::as_u8_slice(&self.vertices),
                glow::STREAM_DRAW,
            );
            self.gl
                .bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.ibo));
            self.gl.buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                glutil::as_u8_slice(&self.indices),
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
                self.indices.len() as i32,
                glow::UNSIGNED_INT,
                0,
            );

            self.gl.disable_vertex_attrib_array(0);
            self.gl.disable_vertex_attrib_array(1);
        }
        self.indices.clear();
        self.vertices.clear();
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
