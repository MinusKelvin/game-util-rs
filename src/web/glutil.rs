use crate::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::HtmlImageElement;

#[derive(Clone)]
pub struct Gl(std::rc::Rc<(glow::Context, web_sys::WebGl2RenderingContext)>);

impl Gl {
    pub(crate) fn new(gl: web_sys::WebGl2RenderingContext) -> Self {
        Gl(std::rc::Rc::new((
            glow::Context::from_webgl2_context(gl.clone()),
            gl,
        )))
    }
}

impl std::ops::Deref for Gl {
    type Target = glow::Context;
    fn deref(&self) -> &glow::Context {
        &self.0 .0
    }
}

async fn load_image(source: &str) -> Result<HtmlImageElement, String> {
    let image = HtmlImageElement::new().unwrap();
    image.set_src(source);
    JsFuture::from(image.decode()).await.ok();
    Ok(image)
}

pub async fn load_texture_2d(gl: &Gl, source: &str) -> Result<glow::Texture, String> {
    let image = load_image(source).await?;
    let texture;
    unsafe {
        texture = gl.create_texture()?;
        gl.bind_texture(glow::TEXTURE_2D, Some(texture));
        gl.tex_image_2d_with_html_image(
            glow::TEXTURE_2D,
            0,
            glow::RGBA8 as i32,
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            &image,
        );
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAX_LEVEL, 0);
    }
    Ok(texture)
}

pub async fn load_texture_layer(
    gl: &Gl,
    source: &str,
    texture: glow::Texture,
    layer: i32,
) -> Result<(), String> {
    let image = load_image(source).await?;
    unsafe {
        gl.bind_texture(glow::TEXTURE_2D_ARRAY, Some(texture));
        gl.0 .1
            .tex_sub_image_3d_with_html_image_element(
                glow::TEXTURE_2D_ARRAY,
                0,
                0,
                0,
                layer,
                image.width() as i32,
                image.height() as i32,
                1,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                &image,
            )
            .ok();
    }
    Ok(())
}
