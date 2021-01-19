use crate::gameloop::*;
use crate::prelude::*;

use futures::channel::oneshot;
use futures::executor::{LocalPool, LocalSpawner};
use futures::task::LocalSpawnExt;
use glutin::{Api, GlRequest, PossiblyCurrent, WindowedContext};
use serde::de::DeserializeOwned;
use std::{future::Future, path::PathBuf};
use winit::event::WindowEvent;
use winit::event_loop::{EventLoop, EventLoopProxy};
use winit::window::{Window, WindowBuilder};

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
        .build_windowed(wb.with_visible(false), &el)
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
    game.context.window().set_visible(true);

    gameloop(el, game, ups, lockstep);
}

pub(crate) struct GamePlatformWrapper<G> {
    game: G,
    context: WindowedContext<PossiblyCurrent>,
    pool: LocalPool,
}

#[derive(Clone)]
pub struct LocalExecutor {
    spawner: LocalSpawner,
}

impl<G: Game> GamePlatformWrapper<G> {
    pub(crate) fn update(&mut self) -> GameloopCommand {
        self.game.update(self.context.window())
    }

    pub(crate) fn render(&mut self, alpha: f64, smooth_delta: f64) {
        self.game.render(self.context.window(), alpha, smooth_delta);
        self.pool.run_until_stalled();
        self.context.swap_buffers().unwrap();
    }

    pub(crate) fn event(&mut self, event: WindowEvent) -> GameloopCommand {
        if let WindowEvent::Resized(size) = event {
            self.context.resize(size);
        }
        self.game.event(self.context.window(), event)
    }

    pub(crate) fn user_event(&mut self, event: G::UserEvent) -> GameloopCommand {
        self.game.user_event(self.context.window(), event)
    }

    pub(crate) fn begin_frame(&mut self) {
        self.pool.run_until_stalled();
    }
}

impl LocalExecutor {
    pub fn spawn(&self, f: impl Future<Output = ()> + 'static) {
        self.spawner.spawn_local(f).unwrap();
    }
}

pub async fn load_binary(source: &str) -> Result<Vec<u8>, String> {
    let (s, r) = oneshot::channel();
    let path = PathBuf::from(source);
    std::thread::spawn(|| s.send(std::fs::read(path).map_err(|e| e.to_string())));
    r.await.unwrap()
}

pub fn store<T: Serialize>(key: &str, value: &T, human_readable: bool) -> Result<(), String> {
    let extension = if human_readable { "yaml" } else { "dat" };
    let to = std::io::BufWriter::new(
        std::fs::File::create(format!("{}.{}", key, extension)).map_err(|e| e.to_string())?,
    );
    if human_readable {
        serde_yaml::to_writer(to, value).map_err(|e| e.to_string())
    } else {
        bincode::serialize_into(to, &value).map_err(|e| e.to_string())
    }
}

pub fn load<T: DeserializeOwned>(key: &str, human_readable: bool) -> Result<Option<T>, String> {
    let extension = if human_readable { "yaml" } else { "dat" };
    let from = match std::fs::File::open(format!("{}.{}", key, extension)) {
        Ok(f) => std::io::BufReader::new(f),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.to_string()),
    };
    if human_readable {
        serde_yaml::from_reader(from)
            .map_err(|e| e.to_string())
            .map(Some)
    } else {
        bincode::deserialize_from(from)
            .map_err(|e| e.to_string())
            .map(Some)
    }
}
