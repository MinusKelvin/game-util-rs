use scopeguard::defer;
use crate::prelude::*;

pub fn compile_shader_program(
    gl: &Gl, vs_code: &str, fs_code: &str
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
    gl: &Gl, shader: glow::Program, name: &str
) -> Result<glow::UniformLocation, String> {
    unsafe { gl.get_uniform_location(shader, name) }.ok_or_else(
        || format!("Could not find uniform named `{}`.", name)
    )
}

/// Convinience function to load RGBA8 textures.
pub fn load_texture(
    gl: &glow::Context, data: &[u8], format: image::ImageFormat
) -> Result<glow::Texture, String> {
    unsafe {
        let tex = gl.create_texture()?;
        gl.bind_texture(glow::TEXTURE_2D, Some(tex));

        let img = image::load_from_memory_with_format(data, format).unwrap();

        if let image::DynamicImage::ImageRgba8(img) = img {
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA8 as _,
                img.width() as i32, img.height() as i32,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                Some(&img)
            );
        } else {
            return Err("Not an RGBA image".to_owned());
        }

        gl.generate_mipmap(glow::TEXTURE_2D);

        Ok(tex)
    }
}

pub fn load_texture_array(
    gl: &glow::Context, data: &[u8], format: image::ImageFormat, tiles_wide: u32, tiles_high: u32
) -> Result<glow::Texture, String> {
    use image::GenericImageView;
    unsafe {
        let tex = gl.create_texture()?;
        gl.bind_texture(glow::TEXTURE_2D_ARRAY, Some(tex));

        let img = image::load_from_memory_with_format(data, format).unwrap();
        let tw = img.width()/tiles_wide;
        let th = img.height()/tiles_high;
        let mut data = Vec::with_capacity(4 * (tw*tiles_wide * th*tiles_high) as usize);
        for ty in 0..tiles_high {
            for tx in 0..tiles_wide {
                for y in (0..th).rev() {
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

        gl.tex_image_3d(
            glow::TEXTURE_2D_ARRAY,
            0,
            glow::RGBA8 as _,
            tw as _, th as _, (tiles_wide*tiles_high) as _,
            0,
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            Some(&data)
        );

        gl.generate_mipmap(glow::TEXTURE_2D_ARRAY);
        
        gl.tex_parameter_i32(
            glow::TEXTURE_2D_ARRAY, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as i32
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D_ARRAY, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as i32
        );

        Ok(tex)
    }
}

pub fn as_u8_slice<T>(data: &[T]) -> &[u8] {
    let size = std::mem::size_of_val(data);
    unsafe {
        std::slice::from_raw_parts(data.as_ptr() as *const _, size)
    }
}
