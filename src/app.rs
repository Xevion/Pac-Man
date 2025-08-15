use std::time::{Duration, Instant};

use glam::Vec2;
use sdl2::render::TextureCreator;
use sdl2::ttf::Sdl2TtfContext;
use sdl2::video::WindowContext;
use sdl2::{AudioSubsystem, EventPump, Sdl, VideoSubsystem};
use tracing::{field, info, warn};

use crate::error::{GameError, GameResult};

use crate::constants::{CANVAS_SIZE, LOOP_TIME, SCALE};
use crate::game::Game;
use crate::platform::get_platform;
use crate::systems::profiling::SystemTimings;

pub struct App {
    pub game: Game,
    last_tick: Instant,
    focused: bool,
    _cursor_pos: Vec2,
}

impl App {
    pub fn new() -> GameResult<Self> {
        let sdl_context: &'static Sdl = Box::leak(Box::new(sdl2::init().map_err(|e| GameError::Sdl(e.to_string()))?));
        let video_subsystem: &'static VideoSubsystem =
            Box::leak(Box::new(sdl_context.video().map_err(|e| GameError::Sdl(e.to_string()))?));
        let _audio_subsystem: &'static AudioSubsystem =
            Box::leak(Box::new(sdl_context.audio().map_err(|e| GameError::Sdl(e.to_string()))?));
        let _ttf_context: &'static Sdl2TtfContext =
            Box::leak(Box::new(sdl2::ttf::init().map_err(|e| GameError::Sdl(e.to_string()))?));
        let event_pump: &'static mut EventPump =
            Box::leak(Box::new(sdl_context.event_pump().map_err(|e| GameError::Sdl(e.to_string()))?));

        // Initialize platform-specific console
        get_platform().init_console()?;

        let window = video_subsystem
            .window(
                "Pac-Man",
                (CANVAS_SIZE.x as f32 * SCALE).round() as u32,
                (CANVAS_SIZE.y as f32 * SCALE).round() as u32,
            )
            .resizable()
            .position_centered()
            .build()
            .map_err(|e| GameError::Sdl(e.to_string()))?;

        let canvas = Box::leak(Box::new(
            window
                .into_canvas()
                .accelerated()
                .present_vsync()
                .build()
                .map_err(|e| GameError::Sdl(e.to_string()))?,
        ));

        canvas
            .set_logical_size(CANVAS_SIZE.x, CANVAS_SIZE.y)
            .map_err(|e| GameError::Sdl(e.to_string()))?;

        let texture_creator: &'static mut TextureCreator<WindowContext> = Box::leak(Box::new(canvas.texture_creator()));

        let game = Game::new(canvas, texture_creator, event_pump)?;
        // game.audio.set_mute(cfg!(debug_assertions));

        Ok(App {
            game,
            focused: true,
            last_tick: Instant::now(),
            _cursor_pos: Vec2::ZERO,
        })
    }

    pub fn run(&mut self) -> bool {
        {
            let start = Instant::now();

            // for event in self
            //     .game
            //     .world
            //     .get_non_send_resource_mut::<&'static mut EventPump>()
            //     .unwrap()
            //     .poll_iter()
            // {
            //     match event {
            //         Event::Window { win_event, .. } => match win_event {
            //             WindowEvent::FocusGained => {
            //                 self.focused = true;
            //             }
            //             WindowEvent::FocusLost => {
            //                 self.focused = false;
            //             }
            //             _ => {}
            //         },
            //         Event::MouseMotion { x, y, .. } => {
            //             // Convert window coordinates to logical coordinates
            //             self.cursor_pos = Vec2::new(x as f32, y as f32);
            //         }
            //         _ => {}
            //     }
            // }

            let dt = self.last_tick.elapsed().as_secs_f32();
            self.last_tick = Instant::now();

            let exit = self.game.tick(dt);

            if exit {
                return false;
            }

            // Show timings if the loop took more than 25% of the loop time
            let show_timings = start.elapsed() > (LOOP_TIME / 4);
            if show_timings || true {
                if let Some(timings) = self.game.world.get_resource::<SystemTimings>() {
                    let mut timings = timings.timings.lock();
                    let total = timings.values().sum::<Duration>();
                    info!("Total: {:?}, Timings: {:?}", total, field::debug(&timings));
                    timings.clear();
                }
            }

            // Sleep if we still have time left
            if start.elapsed() < LOOP_TIME {
                let time = LOOP_TIME.saturating_sub(start.elapsed());
                if time != Duration::ZERO {
                    get_platform().sleep(time, self.focused);
                }
            }

            true
        }
    }
}
