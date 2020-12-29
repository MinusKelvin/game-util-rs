use crate::gameloop::*;
use crate::prelude::*;

use futures::executor::{LocalPool, LocalSpawner};
use futures::task::LocalSpawnExt;
use glutin::{Api, GlRequest, PossiblyCurrent, WindowedContext};
use std::future::Future;
use winit::event::WindowEvent;
use winit::event_loop::{EventLoop, EventLoopProxy};
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

    let context = glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_gl(GlRequest::Specific(Api::OpenGlEs, (3, 0)))
        .build_windowed(wb, &el)
        .unwrap();

    let context = unsafe { context.make_current() }.unwrap();

    let gl =
        Gl::new(unsafe { glow::Context::from_loader_function(|s| context.get_proc_address(s)) });

    unsafe {
        gl.bind_vertex_array(gl.create_vertex_array().ok());
    }

    let mut pool = futures::executor::LocalPool::new();
    let spawner = LocalExecutor {
        spawner: pool.spawner(),
    };

    let game = GamePlatformWrapper {
        game: pool.run_until(init(context.window(), gl, el.create_proxy(), spawner)),
        context,
        pool,
    };

    gameloop(el, game, ups, lockstep);
}

struct GamePlatformWrapper<G: Game> {
    game: G,
    context: WindowedContext<PossiblyCurrent>,
    pool: LocalPool,
}

#[derive(Clone)]
pub struct LocalExecutor {
    spawner: LocalSpawner,
}

impl<G: Game> Game for GamePlatformWrapper<G> {
    type UserEvent = G::UserEvent;

    fn update(&mut self) -> GameloopCommand {
        self.game.update()
    }

    fn render(&mut self, alpha: f64, smooth_delta: f64) {
        self.game.render(alpha, smooth_delta);
        self.context.swap_buffers().unwrap();
    }

    fn event(&mut self, event: WindowEvent, window: WindowId) -> GameloopCommand {
        if let WindowEvent::Resized(size) = event {
            self.context.resize(size);
        }
        self.game.event(event, window)
    }

    fn user_event(&mut self, event: G::UserEvent) -> GameloopCommand {
        self.game.user_event(event)
    }

    fn begin_frame(&mut self) {
        self.pool.run_until_stalled();
        self.game.begin_frame()
    }
}

impl LocalExecutor {
    pub fn spawn(&self, f: impl Future<Output = ()> + 'static) {
        self.spawner.spawn_local(f).unwrap();
    }
}
