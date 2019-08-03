use gl::types::*;
use std::ffi::CString;
use std::ptr::null_mut;

/// Compiles and links a shader program, checks for errors, and cleans up intermediate shaders.
pub fn compile_shader_program(vs_code: &str, fs_code: &str) -> Result<GLuint, String> {
    let vs = compile_shader(gl::VERTEX_SHADER, vs_code)?;
    let fs = compile_shader(gl::FRAGMENT_SHADER, fs_code)?;

    let program = link_program(&[vs, fs]);

    unsafe {
        gl::DeleteShader(vs);
        gl::DeleteShader(fs);
    }

    program
}

/// Compiles a shader and checks for errors.
pub fn compile_shader(shader_type: GLenum, code: &str) -> Result<GLuint, String> {
    unsafe {
        let shader = gl::CreateShader(shader_type);
        gl::ShaderSource(shader, 1, &(code.as_ptr() as *const i8), &(code.len() as i32));
        gl::CompileShader(shader);

        let mut status = 0;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);
        if status == gl::FALSE as i32 {
            let mut length = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut length);
            let mut buffer = Vec::with_capacity(length as usize);
            gl::GetShaderInfoLog(shader, length, null_mut(), buffer.as_mut_ptr() as *mut i8);
            buffer.set_len(length as usize);
            let log = CString::from_vec_unchecked(buffer);

            Err(format!(
                "Failed to compile {} shader. Info log: {}",
                match shader_type {
                    gl::FRAGMENT_SHADER => "fragment",
                    gl::VERTEX_SHADER => "vertex",
                    gl::GEOMETRY_SHADER => "geometry",
                    gl::TESS_CONTROL_SHADER => "tessellation control",
                    gl::TESS_EVALUATION_SHADER => "tessellation evaluation",
                    gl::COMPUTE_SHADER => "compute",
                    _ => "unknown"
                },
                log.to_string_lossy()
            ))
        } else {
            Ok(shader)
        }
    }
}

/// Links a shader program and checks for errors.
pub fn link_program(shaders: &[GLuint]) -> Result<GLuint, String> {
    unsafe {
        let program = gl::CreateProgram();
        for shader in shaders {
            gl::AttachShader(program, *shader);
        }
        gl::LinkProgram(program);

        let mut status = 0;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);
        if status == gl::FALSE as i32 {
            let mut length = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut length);
            let mut buffer = Vec::with_capacity(length as usize);
            gl::GetProgramInfoLog(program, length, null_mut(), buffer.as_mut_ptr() as *mut i8);
            let log = CString::from_vec_unchecked(buffer);
            
            Err(format!("Failed to link shader program. Info log: {}", log.to_string_lossy()))
        } else {
            Ok(program)
        }
    }
}

pub fn get_uniform_location(program: GLuint, name: &str) -> GLint {
    let cstr = CString::new(name).unwrap();
    unsafe { gl::GetUniformLocation(program, cstr.as_ptr()) }
}