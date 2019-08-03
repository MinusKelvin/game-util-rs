use game_util::prelude::*;
use game_util::GameloopCommand;

struct Game {
    context: glutin::WindowedContext<glutin::PossiblyCurrent>,
    drift: f64,
    counter: f64,
    start: std::time::Instant
}

impl game_util::Game for Game {
    fn update(&mut self) -> GameloopCommand {
        let time = std::time::Instant::now() - self.start;
        self.counter += 1.0/60.0;
        self.drift = self.counter - time.as_nanos() as f64 / 1_000_000_000.0;
        GameloopCommand::Continue
    }

    fn render(&mut self, alpha: f64, fps: f64) {
        self.context.window().set_title(&format!(
            "FPS: {:.1}, Drift: {:.3}, Alpha: {:.1}",
            fps, self.drift, alpha
        ));

        unsafe {
            gl::ClearColor(0.25, 0.5, 1.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        self.context.swap_buffers().unwrap();
    }

    fn event(&mut self, event: WindowEvent, _: glutin::WindowId) -> GameloopCommand {
        match event {
            WindowEvent::CloseRequested => return GameloopCommand::Exit,
            WindowEvent::Resized(new_size) => {
                let psize = new_size.to_physical(self.context.window().get_hidpi_factor());
                self.context.resize(psize);
            }
            _ => {}
        }
        GameloopCommand::Continue
    }
}

fn main() {
    let mut events = glutin::EventsLoop::new();
    let (context, _lsize) = game_util::create_context(
        glutin::WindowBuilder::new(),
        0, true,
        &mut events
    );

    let mut game = Game {
        context,
        drift: 0.0,
        counter: 0.0,
        start: std::time::Instant::now()
    };

    game_util::gameloop(&mut events, &mut game, 60.0, true);
}