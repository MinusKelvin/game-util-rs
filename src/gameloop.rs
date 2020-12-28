use winit::event_loop::{ EventLoop, ControlFlow };
use winit::event::{ WindowEvent, Event, StartCause };
use winit::window::WindowId;
use instant::{ Instant, Duration };

pub trait Game {
    type UserEvent;

    fn update(&mut self) -> GameloopCommand;
    fn render(&mut self, alpha: f64, smooth_delta: f64);
    fn event(&mut self, event: WindowEvent, window: WindowId) -> GameloopCommand;
    fn user_event(&mut self, event: Self::UserEvent) -> GameloopCommand;

    fn begin_frame(&mut self) {}
}

pub enum GameloopCommand {
    Continue,
    Exit,
    Pause,
    UnPause,
    ChangeUps(f64)
}

/// Variable UPS interpolation gameloop.
/// 
/// Skips up to `ups / 12` frames when FPS is less than UPS. Does interpolation when FPS is greater
/// than UPS. Smoothes frametimes over 10 frames.
/// 
/// If `lockstep` is true, then when FPS is close to UPS (within 2 Hz or 1 millisecond, whichever
/// is shorter), this will switch to being a lockstep gameloop. This results in more responsive
/// gameplay at the cost of slight drift over time.
pub fn gameloop<G: Game + 'static>(
    el: EventLoop<G::UserEvent>,
    mut game: G,
    mut ups: f64,
    lockstep: bool
) -> ! {
    let mut prev_time = Instant::now();
    let mut frametimes = [Duration::new(0, 16_666_666); 10];
    let mut alpha = 0.0;
    let mut paused = false;
    let mut low_framerate = false;

    el.run(move |event, _, flow| match event {
        Event::NewEvents(StartCause::Poll) => {
            let now = Instant::now();
            frametimes[0] = now - prev_time;
            frametimes.rotate_left(1);
            prev_time = now;

            game.begin_frame();
        }
        Event::WindowEvent { event, window_id } => {
            let command = game.event(event, window_id);
            if process_command(command, &mut paused, &mut ups, &mut alpha) {
                *flow = ControlFlow::Exit;
            }
        }
        Event::MainEventsCleared => {
            let frametime = frametimes.iter().sum::<Duration>() / 10;
            let frametime = frametime.as_nanos() as f64 / 1_000_000_000.0;
    
            if !paused {
                let (lockstep_low, lockstep_high) = lockstep_tolerance(ups);
    
                let high_framerate = frametime < lockstep_low;
                low_framerate = frametime > lockstep_high;
    
                if low_framerate || high_framerate || !lockstep {
                    alpha += frametime * ups;
                } else {
                    alpha = 2.0;
                }
    
                let mut updates = 0;
                while alpha > 1.0 && !paused {
                    updates += 1;
                    if updates as f64 > ups / 12.0 {
                        alpha = alpha.min(2.0);
                    }
                    alpha -= 1.0;
                    if process_command(game.update(), &mut paused, &mut ups, &mut alpha) {
                        *flow = ControlFlow::Exit;
                        return
                    }
                }
            }
    
            game.render(if low_framerate { 1.0 } else { alpha }, frametime);
        }
        Event::UserEvent(e) => {
            if process_command(game.user_event(e), &mut paused, &mut ups, &mut alpha) {
                *flow = ControlFlow::Exit
            }
        }
        _ => {}
    })
}

fn process_command(c: GameloopCommand, paused: &mut bool, ups: &mut f64, alpha: &mut f64) -> bool {
    match c {
        GameloopCommand::Pause => {
            *paused = true;
            false
        }
        GameloopCommand::UnPause => {
            *paused = false;
            false
        }
        GameloopCommand::ChangeUps(new_ups) => {
            *alpha *= new_ups / *ups;
            *ups = new_ups;
            false
        }
        GameloopCommand::Exit => true,
        GameloopCommand::Continue => false
    }
}

fn lockstep_tolerance(ups: f64) -> (f64, f64) {
    let ms_lower_bound = 1.0/ups - 0.001;
    let hz_lower_bound = 1.0/(ups + 2.0);

    let ms_upper_bound = 1.0/ups + 0.001;
    let hz_upper_bound = 1.0/(ups - 2.0);

    (ms_lower_bound.max(hz_lower_bound), ms_upper_bound.min(hz_upper_bound))
}