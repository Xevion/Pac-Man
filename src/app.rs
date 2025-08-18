use std::time::{Duration, Instant};

use glam::Vec2;
use sdl2::render::TextureCreator;
use sdl2::ttf::Sdl2TtfContext;
use sdl2::video::WindowContext;
use sdl2::{AudioSubsystem, EventPump, Sdl, VideoSubsystem};

use crate::error::{GameError, GameResult};

use crate::constants::{CANVAS_SIZE, LOOP_TIME, SCALE};
use crate::game::Game;
use crate::platform::get_platform;

/// Main application wrapper that manages SDL initialization, window lifecycle, and the game loop.
///
/// Handles platform-specific setup, maintains consistent frame timing, and delegates
/// game logic to the contained `Game` instance. The app manages focus state to
/// optimize CPU usage when the window loses focus.
pub struct App {
    pub game: Game,
    last_tick: Instant,
    focused: bool,
    _cursor_pos: Vec2,
}

impl App {
    /// Initializes SDL subsystems, creates the game window, and sets up the game state.
    ///
    /// Performs comprehensive initialization including video/audio subsystems, platform-specific
    /// console setup, window creation with proper scaling, and canvas configuration. All SDL
    /// resources are leaked to maintain 'static lifetimes required by the game architecture.
    ///
    /// # Errors
    ///
    /// Returns `GameError::Sdl` if any SDL initialization step fails, or propagates
    /// errors from `Game::new()` during game state setup.
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

    /// Executes a single frame of the game loop with consistent timing and optional sleep.
    ///
    /// Calculates delta time since the last frame, runs game logic via `game.tick()`,
    /// and implements frame rate limiting by sleeping for remaining time if the frame
    /// completed faster than the target `LOOP_TIME`. Sleep behavior varies based on
    /// window focus to conserve CPU when the game is not active.
    ///
    /// # Returns
    ///
    /// `true` if the game should continue running, `false` if the game requested exit.
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
