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

pub fn get_uniform_location(program: GLuint, name: &str) -> Result<GLint, UniformNotFound> {
    let cstr = CString::new(name).unwrap();
    let loc = unsafe { gl::GetUniformLocation(program, cstr.as_ptr()) };
    if loc == -1 {
        Err(UniformNotFound)
    } else {
        Ok(loc)
    }
}

#[derive(Debug, Clone)]
pub struct UniformNotFound;

impl std::fmt::Display for UniformNotFound {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "Uniform not found")
    }
}

impl std::error::Error for UniformNotFound {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

/// Convinience function to load RGBA8 textures.
pub fn load_texture(data: &[u8], format: image::ImageFormat) -> GLuint {
    let mut tex = 0;
    unsafe {
        gl::GenTextures(1, &mut tex);
        gl::BindTexture(gl::TEXTURE_2D, tex);

        if let image::DynamicImage::ImageRgba8(img) =
                image::load_from_memory_with_format(data, format).unwrap() {
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA8 as _,
                img.width() as GLsizei, img.height() as GLsizei,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                img.as_ptr() as _
            );
        } else {
            panic!("Not an RGBA8 image");
        }

        gl::GenerateMipmap(gl::TEXTURE_2D);
    }
    tex
}

pub fn load_texture_array(
    data: &[u8], format: image::ImageFormat, tiles_wide: u32, tiles_high: u32
) -> GLuint {
    use image::GenericImageView;
    let mut tex = 0;
    unsafe {
        gl::GenTextures(1, &mut tex);
        gl::BindTexture(gl::TEXTURE_2D_ARRAY, tex);

        let img = image::load_from_memory_with_format(data, format).unwrap();
        let tw = img.width()/tiles_wide;
        let th = img.height()/tiles_high;
        let mut data = Vec::with_capacity(4 * (tw*tiles_wide * th*tiles_high) as usize);
        for ty in 0..tiles_high {
            for tx in 0..tiles_wide {
                for y in 0..th {
                    for x in 0..tw {
                        let pixel = img.get_pixel(tx*tw+x, ty*th+y);
                        data.push(pixel[0]);
                        data.push(pixel[1]);
                        data.push(pixel[2]);
                        data.push(pixel[3]);
                    }
                }
            }
        }

        gl::TexImage3D(
            gl::TEXTURE_2D_ARRAY,
            0,
            gl::RGBA8 as _,
            tw as _, th as _, (tiles_wide*tiles_high) as _,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            data.as_ptr() as _
        );

        gl::GenerateMipmap(gl::TEXTURE_2D_ARRAY);
    }
    tex
}