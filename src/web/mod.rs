use js_sys::{ArrayBuffer, Error};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

pub mod glutil;
pub mod sound;
pub mod util;

async fn load_buffer(source: &str) -> Result<ArrayBuffer, String> {
    let response: web_sys::Response =
        JsFuture::from(web_sys::window().unwrap().fetch_with_str(source))
            .await
            .map_err(js_err)?
            .dyn_into()
            .unwrap();
    if !response.ok() {
        return Err(format!(
            "Server responded with {} {}",
            response.status(),
            response.status_text()
        ));
    }
    let buffer = JsFuture::from(response.array_buffer().unwrap())
        .await
        .unwrap()
        .dyn_into()
        .unwrap();
    Ok(buffer)
}

fn js_err(err: JsValue) -> String {
    match err.dyn_into::<Error>() {
        Ok(err) => err.to_string().into(),
        Err(err) => match err.as_string() {
            Some(msg) => msg,
            None => "Unrecognized JS Error type".to_owned(),
        },
    }
}
