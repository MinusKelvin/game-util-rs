use game_util::prelude::*;
use game_util::GameloopCommand;
use glutin::*;

struct Game {
    context: WindowedContext<PossiblyCurrent>,
    lsize: dpi::LogicalSize,
    drift: f64,
    counter: f64,
    start: std::time::Instant,
    text: game_util::TextRenderer
}

impl game_util::Game for Game {
    fn update(&mut self) -> GameloopCommand {
        let time = std::time::Instant::now() - self.start;
        self.counter += 1.0/60.0;
        self.drift = self.counter - time.as_nanos() as f64 / 1_000_000_000.0;
        GameloopCommand::Continue
    }

    fn render(&mut self, alpha: f64, smooth_delta: f64) {
        let dpi = self.context.window().get_hidpi_factor();
        self.text.dpi = dpi as f32;
        self.text.screen_size = (self.lsize.width as f32, self.lsize.height as f32);

        self.text.draw_text(
            &format!(
                "FPS: {:.1}\nDrift: {:.3}\nAlpha: {:.1}\nDPI: {:.1}",
                1.0 / smooth_delta, self.drift, alpha, dpi
            ),
            15.0, 350.0,
            game_util::Alignment::Left,
            [255; 4], 32.0, 0
        );
        self.text.draw_text(
            concat!(
                "These characters aren't in Noto Sans,\n",
                "but we can still draw them because we have\n",
                "fallback fonts: 你好，世界！\n",
                "(that's \"Hello world!\" in Chinese)"
            ),
            15.0, 160.0,
            game_util::Alignment::Left,
            [0, 0, 0, 255], 28.0, 0
        );
        self.text.draw_text(
            "16px",
            10.0, 10.0,
            game_util::Alignment::Left,
            [0, 0, 0, 255], 16.0, 0
        );

        let (width, height): (u32, _) = self.lsize.to_physical(dpi).into();
        let (width, height) = (width as i32, height as i32);

        unsafe {
            gl::Viewport(0, 0, width, height);

            gl::ClearColor(0.25, 0.5, 1.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }

        self.text.render();

        self.context.swap_buffers().unwrap();
    }

    fn event(&mut self, event: WindowEvent, _: WindowId) -> GameloopCommand {
        match event {
            WindowEvent::CloseRequested => return GameloopCommand::Exit,
            WindowEvent::Resized(new_size) => {
                let psize = new_size.to_physical(self.context.window().get_hidpi_factor());
                self.context.resize(psize);
                self.lsize = new_size;
            }
            _ => {}
        }
        GameloopCommand::Continue
    }
}

fn main() {
    let mut events = EventsLoop::new();
    let (context, lsize) = game_util::create_context(
        WindowBuilder::new(),
        0, true,
        &mut events
    );

    let mut game = Game {
        context,
        lsize,
        drift: 0.0,
        counter: 0.0,
        start: std::time::Instant::now(),
        text: {
            use rusttype::Font;
            let mut t = game_util::TextRenderer::new();
            t.add_style(ArrayVec::from([
                Font::from_bytes(include_bytes!("NotoSans-Regular.ttf") as &[u8]).unwrap(),
                Font::from_bytes(include_bytes!("WenQuanYiMicroHei.ttf") as &[u8]).unwrap(),
            ]));
            t
        }
    };

    game_util::gameloop(&mut events, &mut game, 60.0, true);
}