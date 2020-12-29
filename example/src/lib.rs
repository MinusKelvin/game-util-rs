use game_util::prelude::*;
use game_util::rusttype::Font;
use game_util::winit::dpi::PhysicalSize;
use game_util::winit::event::WindowEvent;
use game_util::winit::window::{WindowBuilder, WindowId};
use game_util::GameloopCommand;
use instant::Instant;

struct Game {
    gl: Gl,
    psize: PhysicalSize<u32>,
    drift: f64,
    counter: f64,
    start: Instant,
    dpi: f64,
    text: game_util::TextRenderer,
}

impl game_util::Game for Game {
    type UserEvent = ();

    fn update(&mut self) -> GameloopCommand {
        let time = Instant::now() - self.start;
        self.counter += 1.0 / 60.0;
        self.drift = self.counter - time.as_secs_f64();
        GameloopCommand::Continue
    }

    fn render(&mut self, alpha: f64, smooth_delta: f64) {
        let lsize = self.psize.to_logical::<f64>(self.dpi);
        self.text.dpi = self.dpi as f32;
        self.text.screen_size = (lsize.width as f32, lsize.height as f32);

        self.text.draw_text(
            &format!(
                "FPS: {:.1}\nDrift: {:.3}\nAlpha: {:.1}\nDPI: {:.1}",
                1.0 / smooth_delta,
                self.drift,
                alpha,
                self.dpi
            ),
            15.0,
            350.0,
            game_util::Alignment::Left,
            [255; 4],
            32.0,
            0,
        );
        self.text.draw_text(
            concat!(
                "These characters aren't in Noto Sans,\n",
                "but we can still draw them because we have\n",
                "fallback fonts: 你好，世界！\n",
                "(that's \"Hello world!\" in Chinese)"
            ),
            15.0,
            160.0,
            game_util::Alignment::Left,
            [0, 0, 0, 255],
            28.0,
            0,
        );
        self.text.draw_text(
            "16px",
            10.0,
            10.0,
            game_util::Alignment::Left,
            [0, 0, 0, 255],
            16.0,
            0,
        );

        self.text.draw_text(
            &unsafe { self.gl.get_parameter_string(glow::VERSION) },
            100.0,
            10.0,
            game_util::Alignment::Left,
            [0, 0, 0, 255],
            16.0,
            0,
        );

        unsafe {
            self.gl
                .viewport(0, 0, self.psize.width as i32, self.psize.height as i32);

            self.gl.clear_color(0.25, 0.5, 1.0, 1.0);
            self.gl.clear(glow::COLOR_BUFFER_BIT);

            self.gl.enable(glow::BLEND);
            self.gl
                .blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
        }

        self.text.render();
    }

    fn event(&mut self, event: WindowEvent, _: WindowId) -> GameloopCommand {
        match event {
            WindowEvent::CloseRequested => return GameloopCommand::Exit,
            WindowEvent::Resized(new_size) => {
                self.psize = new_size;
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => self.dpi = scale_factor,
            _ => {}
        }
        GameloopCommand::Continue
    }

    fn user_event(&mut self, _: ()) -> GameloopCommand {
        GameloopCommand::Continue
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen(start))]
pub fn main() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    game_util::platform::launch(WindowBuilder::new(), 60.0, true, |window, gl, _, _| {
        let dpi = window.scale_factor();
        let psize = window.inner_size();
        async move {
            Game {
                psize,
                dpi,
                drift: 0.0,
                counter: 0.0,
                start: Instant::now(),
                text: {
                    let mut t = game_util::TextRenderer::new(&gl).unwrap();
                    t.add_style(ArrayVec::from([
                        Font::try_from_bytes(include_bytes!("NotoSans-Regular.ttf") as &[u8])
                            .unwrap(),
                        Font::try_from_bytes(include_bytes!("WenQuanYiMicroHei.ttf") as &[u8])
                            .unwrap(),
                    ]));
                    t
                },
                gl,
            }
        }
    });
}
