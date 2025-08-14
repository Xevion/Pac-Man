use std::time::{Duration, Instant};

use glam::Vec2;
use sdl2::event::{Event, WindowEvent};
use sdl2::render::{Canvas, ScaleMode, Texture, TextureCreator};
use sdl2::ttf::Sdl2TtfContext;
use sdl2::video::{Window, WindowContext};
use sdl2::{AudioSubsystem, EventPump, Sdl, VideoSubsystem};
use tracing::{error, event};

use crate::error::{GameError, GameResult};

use crate::constants::{CANVAS_SIZE, LOOP_TIME, SCALE};
use crate::game::Game;
use crate::input::commands::GameCommand;
use crate::input::InputSystem;
use crate::platform::get_platform;

pub struct App {
    game: Game,
    input_system: InputSystem,
    canvas: Canvas<Window>,
    event_pump: &'static mut EventPump,
    backbuffer: Texture<'static>,
    paused: bool,
    last_tick: Instant,
    cursor_pos: Vec2,
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

        let mut canvas = window
            .into_canvas()
            .accelerated()
            .present_vsync()
            .build()
            .map_err(|e| GameError::Sdl(e.to_string()))?;

        canvas
            .set_logical_size(CANVAS_SIZE.x, CANVAS_SIZE.y)
            .map_err(|e| GameError::Sdl(e.to_string()))?;

        let texture_creator: &'static TextureCreator<WindowContext> = Box::leak(Box::new(canvas.texture_creator()));

        let mut game = Game::new(texture_creator)?;
        // game.audio.set_mute(cfg!(debug_assertions));

        let mut backbuffer = texture_creator
            .create_texture_target(None, CANVAS_SIZE.x, CANVAS_SIZE.y)
            .map_err(|e| GameError::Sdl(e.to_string()))?;
        backbuffer.set_scale_mode(ScaleMode::Nearest);

        // Initial draw
        game.draw(&mut canvas, &mut backbuffer)
            .map_err(|e| GameError::Sdl(e.to_string()))?;
        game.present_backbuffer(&mut canvas, &backbuffer, glam::Vec2::ZERO)
            .map_err(|e| GameError::Sdl(e.to_string()))?;

        Ok(App {
            game,
            input_system: InputSystem::new(),
            canvas,
            event_pump,
            backbuffer,
            paused: false,
            last_tick: Instant::now(),
            cursor_pos: Vec2::ZERO,
        })
    }

    pub fn run(&mut self) -> bool {
        {
            let start = Instant::now();

            for event in self.event_pump.poll_iter() {
                match event {
                    Event::Window { win_event, .. } => match win_event {
                        WindowEvent::Hidden => {
                            event!(tracing::Level::DEBUG, "Window hidden");
                        }
                        WindowEvent::Shown => {
                            event!(tracing::Level::DEBUG, "Window shown");
                        }
                        _ => {}
                    },
                    // It doesn't really make sense to have this available in the browser
                    #[cfg(not(target_os = "emscripten"))]
                    Event::Quit { .. } => {
                        event!(tracing::Level::INFO, "Exit requested. Exiting...");
                        return false;
                    }
                    Event::MouseMotion { x, y, .. } => {
                        // Convert window coordinates to logical coordinates
                        self.cursor_pos = Vec2::new(x as f32, y as f32);
                    }
                    _ => {}
                }

                let commands = self.input_system.handle_event(&event);
                for command in commands {
                    match command {
                        GameCommand::Exit => {
                            event!(tracing::Level::INFO, "Exit requested. Exiting...");
                            return false;
                        }
                        GameCommand::TogglePause => {
                            self.paused = !self.paused;
                            event!(tracing::Level::INFO, "{}", if self.paused { "Paused" } else { "Unpaused" });
                        }
                        _ => self.game.post_event(command.into()),
                    }
                }
            }

            let dt = self.last_tick.elapsed().as_secs_f32();
            self.last_tick = Instant::now();

            if !self.paused {
                self.game.tick(dt);
                if let Err(e) = self.game.draw(&mut self.canvas, &mut self.backbuffer) {
                    error!("Failed to draw game: {}", e);
                }
                if let Err(e) = self
                    .game
                    .present_backbuffer(&mut self.canvas, &self.backbuffer, self.cursor_pos)
                {
                    error!("Failed to present backbuffer: {}", e);
                }
            }

            if start.elapsed() < LOOP_TIME {
                let time = LOOP_TIME.saturating_sub(start.elapsed());
                if time != Duration::ZERO {
                    get_platform().sleep(time);
                }
            } else {
                event!(
                    tracing::Level::WARN,
                    "Game loop behind schedule by: {:?}",
                    start.elapsed() - LOOP_TIME
                );
            }

            true
        }
    }
}
