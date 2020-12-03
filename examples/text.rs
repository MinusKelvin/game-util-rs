use game_util::prelude::*;
use game_util::GameloopCommand;
use glutin::*;
use glutin::event_loop::EventLoop;
use glutin::event::WindowEvent;
use glutin::window::{ WindowBuilder, WindowId };

struct Game {
    context: WindowedContext<PossiblyCurrent>,
    gl: Gl,
    psize: dpi::PhysicalSize<u32>,
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
        let dpi = self.context.window().scale_factor();
        let lsize = self.psize.to_logical::<f64>(dpi);
        self.text.dpi = dpi as f32;
        self.text.screen_size = (lsize.width as f32, lsize.height as f32);

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

        unsafe {
            self.gl.viewport(0, 0, self.psize.width as i32, self.psize.height as i32);

            self.gl.clear_color(0.25, 0.5, 1.0, 1.0);
            self.gl.clear(glow::COLOR_BUFFER_BIT);

            self.gl.enable(glow::BLEND);
            self.gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
        }

        self.text.render();

        self.context.swap_buffers().unwrap();
    }

    fn event(&mut self, event: WindowEvent, _: WindowId) -> GameloopCommand {
        match event {
            WindowEvent::CloseRequested => return GameloopCommand::Exit,
            WindowEvent::Resized(new_size) => {
                self.context.resize(new_size);
                self.psize = new_size;
            }
            _ => {}
        }
        GameloopCommand::Continue
    }
}

fn main() {
    let mut events = EventLoop::new();
    let (context, gl) = game_util::create_context(
        WindowBuilder::new(),
        0, true,
        &mut events
    ).unwrap();

    let game = Game {
        psize: context.window().inner_size(),
        context,
        drift: 0.0,
        counter: 0.0,
        start: std::time::Instant::now(),
        text: {
            use rusttype::Font;
            let mut t = game_util::TextRenderer::new(&gl).unwrap();
            t.add_style(ArrayVec::from([
                Font::try_from_bytes(include_bytes!("NotoSans-Regular.ttf") as &[u8]).unwrap(),
                Font::try_from_bytes(include_bytes!("WenQuanYiMicroHei.ttf") as &[u8]).unwrap(),
            ]));
            t
        },
        gl,
    };

    game_util::gameloop(events, game, 60.0, true);
}