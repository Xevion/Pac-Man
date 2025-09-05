use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::error::{GameError, GameResult};

use crate::constants::{CANVAS_SIZE, LOOP_TIME, SCALE};
use crate::formatter;
use crate::game::Game;
use crate::platform;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::RendererInfo;
use sdl2::{AudioSubsystem, Sdl};
use tracing::debug;

/// Main application wrapper that manages SDL initialization, window lifecycle, and the game loop.
pub struct App {
    pub game: Game,
    last_tick: Instant,
    focused: bool,
    // Keep SDL alive for the app lifetime so subsystems (audio) are not shut down
    _sdl_context: Sdl,
    _audio_subsystem: AudioSubsystem,
}

impl App {
    /// Initializes SDL subsystems, creates the game window, and sets up the game state.
    ///
    /// # Errors
    ///
    /// Returns `GameError::Sdl` if any SDL initialization step fails, or propagates
    /// errors from `Game::new()` during game state setup.
    pub fn new() -> GameResult<Self> {
        let sdl_context = sdl2::init().map_err(|e| GameError::Sdl(e.to_string()))?;
        let ttf_context = sdl2::ttf::init().map_err(|e| GameError::Sdl(e.to_string()))?;
        let video_subsystem = sdl_context.video().map_err(|e| GameError::Sdl(e.to_string()))?;
        let audio_subsystem = sdl_context.audio().map_err(|e| GameError::Sdl(e.to_string()))?;
        let event_pump = sdl_context.event_pump().map_err(|e| GameError::Sdl(e.to_string()))?;

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

        #[derive(Debug)]
        struct DriverDetail {
            info: RendererInfo,
            index: usize,
        }

        let drivers: HashMap<&'static str, DriverDetail> = sdl2::render::drivers()
            .enumerate()
            .map(|(index, d)| (d.name, DriverDetail { info: d, index }))
            .collect::<HashMap<_, _>>();

        let get_driver =
            |name: &'static str| -> Option<u32> { drivers.get(name.to_lowercase().as_str()).map(|d| d.index as u32) };

        {
            let mut names = drivers.keys().collect::<Vec<_>>();
            names.sort_by_key(|k| get_driver(k));
            debug!("Drivers: {names:?}")
        }

        // Count the number of times each pixel format is supported by each driver
        let pixel_format_counts: HashMap<PixelFormatEnum, usize> = drivers
            .values()
            .flat_map(|d| d.info.texture_formats.iter())
            .fold(HashMap::new(), |mut counts, format| {
                *counts.entry(*format).or_insert(0) += 1;
                counts
            });

        debug!("Pixel format counts: {pixel_format_counts:?}");

        let index = get_driver("direct3d");
        debug!("Driver index: {index:?}");

        let mut canvas = window
            .into_canvas()
            .accelerated()
            // .index(index)
            .build()
            .map_err(|e| GameError::Sdl(e.to_string()))?;

        canvas
            .set_logical_size(CANVAS_SIZE.x, CANVAS_SIZE.y)
            .map_err(|e| GameError::Sdl(e.to_string()))?;
        debug!("Renderer: {:?}", canvas.info());

        let texture_creator = canvas.texture_creator();

        let game = Game::new(canvas, ttf_context, texture_creator, event_pump)?;

        Ok(App {
            game,
            focused: true,
            last_tick: Instant::now(),
            _sdl_context: sdl_context,
            _audio_subsystem: audio_subsystem,
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

            let dt = self.last_tick.elapsed().as_secs_f32();
            self.last_tick = start;

            // Increment the global tick counter for tracing
            formatter::increment_tick();

            let exit = self.game.tick(dt);

            if exit {
                return false;
            }

            // Sleep if we still have time left
            if start.elapsed() < LOOP_TIME {
                let time = LOOP_TIME.saturating_sub(start.elapsed());
                if time != Duration::ZERO {
                    platform::sleep(time, self.focused);
                }
            }

            true
        }
    }
}
