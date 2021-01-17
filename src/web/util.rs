use crate::gameloop::*;
use crate::prelude::*;

use bincode::Options;
use js_sys::JsString;
use serde::de::DeserializeOwned;
use std::future::Future;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlElement;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{EventLoop, EventLoopProxy};
use winit::platform::web::WindowExtWebSys;
use winit::window::{Window, WindowBuilder, WindowId};

pub fn launch<G, F>(
    wb: WindowBuilder,
    ups: f64,
    lockstep: bool,
    init: impl FnOnce(&Window, Gl, EventLoopProxy<G::UserEvent>, LocalExecutor) -> F,
) where
    G: Game + 'static,
    F: Future<Output = G> + 'static,
{
    let el = EventLoop::with_user_event();

    let document = web_sys::window().unwrap().document().unwrap();
    let container = document.body().unwrap();
    let w = container.client_width();
    let h = container.client_height();
    let window = wb
        .with_inner_size(LogicalSize::new(w, h))
        .build(&el)
        .unwrap();

    let attributes = js_sys::Object::new();
    js_sys::Reflect::set(&attributes, &"alpha".into(), &false.into()).unwrap();
    let gl = Gl::new(
        window
            .canvas()
            .get_context_with_context_options("webgl2", &attributes)
            .unwrap()
            .unwrap()
            .dyn_into()
            .unwrap(),
    );

    unsafe {
        gl.bind_vertex_array(gl.create_vertex_array().ok());
    }

    let game_future = init(
        &window,
        gl,
        el.create_proxy(),
        LocalExecutor { _private: () },
    );
    spawn_local(async move {
        let game = GamePlatformWrapper {
            game: game_future.await,
            container,
            window,
        };

        game.container
            .append_with_node_1(&game.window.canvas())
            .unwrap();
        game.window.canvas().focus();

        webutil::global::set_timeout(0, move || gameloop(el, game, ups, lockstep)).forget();
    });
}

struct GamePlatformWrapper<G: Game> {
    game: G,
    container: HtmlElement,
    window: Window,
}

#[derive(Clone)]
pub struct LocalExecutor {
    _private: (),
}

impl<G: Game> Game for GamePlatformWrapper<G> {
    type UserEvent = G::UserEvent;

    fn update(&mut self) -> GameloopCommand {
        self.game.update()
    }

    fn render(&mut self, alpha: f64, smooth_delta: f64) {
        self.game.render(alpha, smooth_delta);
    }

    fn event(&mut self, event: WindowEvent, window: WindowId) -> GameloopCommand {
        self.game.event(event, window)
    }

    fn user_event(&mut self, event: G::UserEvent) -> GameloopCommand {
        self.game.user_event(event)
    }

    fn begin_frame(&mut self) {
        let w = self.container.client_width();
        let h = self.container.client_height();
        self.window.set_inner_size(LogicalSize::new(w, h));

        self.game.begin_frame()
    }
}

impl LocalExecutor {
    pub fn spawn(&self, f: impl Future<Output = ()> + 'static) {
        spawn_local(f);
    }
}

pub async fn load_binary(source: &str) -> Result<Vec<u8>, String> {
    let buffer = super::load_buffer(source).await?;
    Ok(js_sys::Uint8Array::new(&buffer).to_vec())
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(catch, js_namespace = localStorage, js_name = getItem)]
    fn get_item(key: &str) -> Result<Option<JsString>, JsValue>;

    #[wasm_bindgen(catch, js_namespace = localStorage, js_name = setItem)]
    fn set_item(key: &str, value: &JsString) -> Result<(), JsValue>;
}

pub fn store<T: Serialize>(key: &str, value: &T) -> Result<(), String> {
    let mut serialized = bincode::serialize(value).map_err(|e| e.to_string())?;
    if serialized.len() % 2 != 0 {
        serialized.push(0);
    }
    if serialized.len() > 5 * 1024 * 1024 {
        web_sys::console::warn_1(&JsValue::from_str(&format!(
            "Local storage object '{}' exceeds 5 MB",
            key
        )));
    }
    let value = JsString::from_char_code(unsafe {
        // View the even-length [u8] as a [u16].
        // This is little-endian because wasm32 is little-endian.
        std::slice::from_raw_parts(serialized.as_ptr() as *const _, serialized.len() / 2)
    });
    set_item(key, &value).map_err(super::js_err)
}

pub fn load<T: DeserializeOwned>(key: &str) -> Result<Option<T>, String> {
    let data = match get_item(key) {
        Ok(Some(v)) => v.iter().collect::<Vec<_>>(),
        Ok(None) => return Ok(None),
        Err(e) => return Err(super::js_err(e)),
    };
    let data = unsafe {
        // View the [u16] as a [u8].
        // This is little-endian because wasm32 is little-endian.
        std::slice::from_raw_parts(data.as_ptr() as *const _, data.len() * 2)
    };
    bincode::options()
        .allow_trailing_bytes()
        .deserialize(data)
        .map_err(|e| e.to_string())
}
