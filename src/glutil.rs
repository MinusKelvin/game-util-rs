use crate::prelude::*;
use scopeguard::defer;

pub use crate::backend::glutil::*;

pub fn compile_shader_program(
    gl: &Gl,
    vs_code: &str,
    fs_code: &str,
) -> Result<glow::Program, String> {
    let vs = compile_shader(gl, glow::VERTEX_SHADER, vs_code)?;
    defer!(unsafe { gl.delete_shader(vs) });
    let fs = compile_shader(gl, glow::FRAGMENT_SHADER, fs_code)?;
    defer!(unsafe { gl.delete_shader(fs) });

    link_program(gl, &[vs, fs])
}

pub fn compile_shader(gl: &Gl, shader_type: u32, code: &str) -> Result<glow::Shader, String> {
    unsafe {
        let shader = gl.create_shader(shader_type)?;
        gl.shader_source(shader, code);
        gl.compile_shader(shader);

        if !gl.get_shader_compile_status(shader) {
            let info_log = gl.get_shader_info_log(shader);
            gl.delete_shader(shader);
            Err(info_log)
        } else {
            Ok(shader)
        }
    }
}

pub fn link_program(gl: &Gl, shaders: &[glow::Shader]) -> Result<glow::Program, String> {
    unsafe {
        let program = gl.create_program()?;
        for &shader in shaders {
            gl.attach_shader(program, shader);
        }
        gl.link_program(program);

        if !gl.get_program_link_status(program) {
            let info_log = gl.get_program_info_log(program);
            gl.delete_program(program);
            Err(info_log)
        } else {
            Ok(program)
        }
    }
}

pub fn get_uniform_location(
    gl: &Gl,
    shader: glow::Program,
    name: &str,
) -> Result<glow::UniformLocation, String> {
    unsafe { gl.get_uniform_location(shader, name) }
        .ok_or_else(|| format!("Could not find uniform named `{}`.", name))
}

pub fn as_u8_slice<T>(data: &[T]) -> &[u8] {
    let size = std::mem::size_of_val(data);
    unsafe { std::slice::from_raw_parts(data.as_ptr() as *const _, size) }
}
