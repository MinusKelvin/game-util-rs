use game_util::prelude::*;
use game_util::rusttype::Font;
use game_util::sound::{Sound, SoundService};
use game_util::sprite::SpriteBatch;
use game_util::text::{Alignment, TextRenderer};
use game_util::winit::dpi::PhysicalSize;
use game_util::winit::event::{ElementState, MouseButton, WindowEvent};
use game_util::winit::window::{WindowBuilder, Window};
use game_util::GameloopCommand;
use instant::Instant;

struct Game {
    gl: Gl,
    psize: PhysicalSize<u32>,
    drift: f64,
    counter: f64,
    start: Instant,
    dpi: f64,
    text: TextRenderer,
    ball_pos: Point2<f32>,
    ball_prev_pos: Point2<f32>,
    ball_vel: Vec2<f32>,
    mouse_pos: Point2<f32>,
    mouse_in_window: bool,
    sprites: sprites::Sprites,
    sprite_renderer: SpriteBatch,
    pluck: Sound,
    sound_service: SoundService,
}

impl game_util::Game for Game {
    type UserEvent = game_util::rusttype::Font<'static>;

    fn update(&mut self, _: &Window) -> GameloopCommand {
        self.ball_prev_pos = self.ball_pos;
        if self.mouse_in_window {
            self.ball_vel += (self.mouse_pos - self.ball_pos) * 0.01;
        }
        self.ball_vel *= 0.95;
        self.ball_pos += self.ball_vel;

        let time = Instant::now() - self.start;
        self.counter += 1.0 / 60.0;
        self.drift = self.counter - time.as_secs_f64();
        GameloopCommand::Continue
    }

    fn render(&mut self, _: &Window, alpha: f64, smooth_delta: f64) {
        let lsize = self.psize.to_logical::<f64>(self.dpi);
        self.text.dpi = self.dpi as f32;
        self.text.screen_size = (lsize.width as f32, lsize.height as f32);

        self.sprite_renderer.draw(
            &self.sprites.ball,
            self.ball_prev_pos.lerp(self.ball_pos, alpha as f32),
            [255; 4],
        );

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
            Alignment::Left,
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
            Alignment::Left,
            [0, 0, 0, 255],
            28.0,
            0,
        );
        self.text
            .draw_text("16px", 10.0, 10.0, Alignment::Left, [0, 0, 0, 255], 16.0, 0);

        self.text.draw_text(
            &unsafe { self.gl.get_parameter_string(glow::VERSION) },
            100.0,
            10.0,
            Alignment::Left,
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

        self.sprite_renderer.render(Transform3D::ortho(
            0.0,
            self.psize.width as f32,
            0.0,
            self.psize.height as f32,
            -1.0,
            1.0,
        ));

        self.text.render();
    }

    fn event(&mut self, _: &Window, event: WindowEvent) -> GameloopCommand {
        match event {
            WindowEvent::CloseRequested => return GameloopCommand::Exit,
            WindowEvent::Resized(new_size) => {
                self.psize = new_size;
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => self.dpi = scale_factor,
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_pos = point2(
                    position.x as f32,
                    self.psize.height as f32 - position.y as f32,
                )
            }
            WindowEvent::CursorLeft { .. } => self.mouse_in_window = false,
            WindowEvent::CursorEntered { .. } => self.mouse_in_window = true,
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state: ElementState::Pressed,
                ..
            } => self.sound_service.play(&self.pluck),
            _ => {}
        }
        GameloopCommand::Continue
    }

    fn user_event(&mut self, _: &Window, font: Self::UserEvent) -> GameloopCommand {
        self.text.add_fallback_font(0, font);
        GameloopCommand::Continue
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen(start))]
pub fn main() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    game_util::launch(
        WindowBuilder::new(),
        60.0,
        true,
        |window, gl, proxy, executor| {
            let dpi = window.scale_factor();
            let psize = window.inner_size();
            async move {
                executor.spawn(async move {
                    proxy
                        .send_event(
                            Font::try_from_vec(
                                game_util::load_binary("res/WenQuanYiMicroHei.ttf")
                                    .await
                                    .unwrap(),
                            )
                            .unwrap(),
                        )
                        .ok();
                });

                let (noto_sans, (sprites, sprite_tex), pluck) = game_util::futures::join!(
                    async {
                        Font::try_from_vec(
                            game_util::load_binary("res/NotoSans-Regular.ttf")
                                .await
                                .unwrap(),
                        )
                        .unwrap()
                    },
                    async { sprites::Sprites::load(&gl, "res/generated").await.unwrap() },
                    async { Sound::load("res/pluck.ogg").await.unwrap() }
                );

                let center = point2(psize.width as f32, psize.height as f32) / 2.0;
                Game {
                    psize,
                    dpi,
                    drift: 0.0,
                    counter: 0.0,
                    ball_pos: center,
                    ball_prev_pos: center,
                    ball_vel: vec2(0.0, 0.0),
                    mouse_pos: center,
                    mouse_in_window: false,
                    start: Instant::now(),
                    text: {
                        let mut t = TextRenderer::new(&gl).unwrap();
                        t.add_style(Some(noto_sans));
                        t
                    },
                    sprites,
                    sprite_renderer: SpriteBatch::new(
                        &gl,
                        game_util::sprite::sprite_shader(&gl),
                        sprite_tex,
                    )
                    .unwrap(),
                    gl,
                    pluck,
                    sound_service: SoundService::new(&executor),
                }
            }
        },
    );
}

include!(concat!(env!("OUT_DIR"), "/sprites.rs"));
