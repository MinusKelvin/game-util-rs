use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::Blob;

pub mod glutil;
pub mod sound;
pub mod util;

async fn load_blob(source: &str) -> Result<Blob, String> {
    match JsFuture::from(web_sys::window().unwrap().fetch_with_str(source)).await {
        Ok(v) => {
            let response: web_sys::Response = v.dyn_into().unwrap();
            if !response.ok() {
                return Err(format!(
                    "Server responded with {} {}",
                    response.status(),
                    response.status_text()
                ));
            }
            let blob = JsFuture::from(response.blob().unwrap())
                .await
                .unwrap()
                .dyn_into()
                .unwrap();
            Ok(blob)
        }
        Err(_) => Err("fetch promise rejected".to_string()),
    }
}
