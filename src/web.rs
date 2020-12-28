use crate::prelude::*;
use crate::gameloop::*;

use std::future::Future;
use winit::window::{ WindowBuilder, WindowId, Window };
use winit::event_loop::{ EventLoop, EventLoopProxy };
use winit::event::WindowEvent;
use winit::dpi::LogicalSize;
use winit::platform::web::WindowExtWebSys;
use web_sys::HtmlElement;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

pub fn launch<G, F>(
    wb: WindowBuilder,
    ups: f64,
    lockstep: bool,
    init: impl FnOnce(&Window, Gl, EventLoopProxy<G::UserEvent>, LocalExecutor) -> F
)
where
    G: Game + 'static,
    F: Future<Output = G> + 'static
{
    let el = EventLoop::with_user_event();

    let document = web_sys::window().unwrap().document().unwrap();
    let container = document.body().unwrap();
    let window = wb.build(&el).unwrap();

    container.append_with_node_1(&window.canvas()).unwrap();

    let attributes = js_sys::Object::new();
    js_sys::Reflect::set(&attributes, &"alpha".into(), &false.into()).unwrap();
    let gl = Gl::new(glow::Context::from_webgl2_context(
        window.canvas().get_context_with_context_options("webgl2", &attributes)
            .unwrap().unwrap().dyn_into().unwrap()
    ));

    unsafe {
        gl.bind_vertex_array(gl.create_vertex_array().ok());
    }

    let game_future = init(&window, gl, el.create_proxy(), LocalExecutor { _private: () });
    spawn_local(async move {
        let game = GamePlatformWrapper {
            game: game_future.await,
            container,
            window
        };

        gameloop(el, game, ups, lockstep);
    });
}

struct GamePlatformWrapper<G: Game> {
    game: G,
    container: HtmlElement,
    window: Window
}

#[derive(Clone)]
pub struct LocalExecutor {
    _private: ()
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
