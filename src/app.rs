use std::time::{Duration, Instant};

use anyhow::{anyhow, Result};
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::render::{Canvas, ScaleMode, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};
use sdl2::EventPump;
use tracing::{error, event};

use crate::constants::{CANVAS_SIZE, LOOP_TIME, SCALE};
use crate::game::Game;

#[cfg(target_os = "emscripten")]
use crate::emscripten;

#[cfg(not(target_os = "emscripten"))]
fn sleep(value: Duration) {
    spin_sleep::sleep(value);
}

#[cfg(target_os = "emscripten")]
fn sleep(value: Duration) {
    emscripten::emscripten::sleep(value.as_millis() as u32);
}

pub struct App<'a> {
    game: Game,
    canvas: Canvas<Window>,
    event_pump: EventPump,
    backbuffer: Texture<'a>,
    paused: bool,
    last_tick: Instant,
}

impl App<'_> {
    pub fn new() -> Result<Self> {
        let sdl_context = sdl2::init().map_err(|e| anyhow!(e))?;
        let video_subsystem = sdl_context.video().map_err(|e| anyhow!(e))?;
        let audio_subsystem = sdl_context.audio().map_err(|e| anyhow!(e))?;
        let ttf_context = sdl2::ttf::init().map_err(|e| anyhow!(e.to_string()))?;

        let window = video_subsystem
            .window(
                "Pac-Man",
                (CANVAS_SIZE.x as f32 * SCALE).round() as u32,
                (CANVAS_SIZE.y as f32 * SCALE).round() as u32,
            )
            .resizable()
            .position_centered()
            .build()?;

        let mut canvas = window.into_canvas().build()?;
        canvas.set_logical_size(CANVAS_SIZE.x, CANVAS_SIZE.y)?;

        let texture_creator_static: &'static TextureCreator<WindowContext> = Box::leak(Box::new(canvas.texture_creator()));

        let mut game = Game::new(texture_creator_static, &ttf_context, &audio_subsystem);
        game.audio.set_mute(cfg!(debug_assertions));

        let mut backbuffer = texture_creator_static.create_texture_target(None, CANVAS_SIZE.x, CANVAS_SIZE.y)?;
        backbuffer.set_scale_mode(ScaleMode::Nearest);

        let event_pump = sdl_context.event_pump().map_err(|e| anyhow!(e))?;

        // Initial draw
        game.draw(&mut canvas, &mut backbuffer)?;
        game.present_backbuffer(&mut canvas, &backbuffer)?;

        Ok(Self {
            game,
            canvas,
            event_pump,
            backbuffer,
            paused: false,
            last_tick: Instant::now(),
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
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape) | Some(Keycode::Q),
                        ..
                    } => {
                        event!(tracing::Level::INFO, "Exit requested. Exiting...");
                        return false;
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::P),
                        ..
                    } => {
                        self.paused = !self.paused;
                        event!(tracing::Level::INFO, "{}", if self.paused { "Paused" } else { "Unpaused" });
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::Space),
                        ..
                    } => {
                        self.game.debug_mode = !self.game.debug_mode;
                    }
                    Event::KeyDown { keycode, .. } => {
                        self.game.keyboard_event(keycode.unwrap());
                    }
                    _ => {}
                }
            }

            let dt = self.last_tick.elapsed().as_secs_f32();
            self.last_tick = Instant::now();

            if !self.paused {
                self.game.tick(dt);
                if let Err(e) = self.game.draw(&mut self.canvas, &mut self.backbuffer) {
                    error!("Failed to draw game: {e}");
                }
                if let Err(e) = self.game.present_backbuffer(&mut self.canvas, &self.backbuffer) {
                    error!("Failed to present backbuffer: {e}");
                }
            }

            if start.elapsed() < LOOP_TIME {
                let time = LOOP_TIME.saturating_sub(start.elapsed());
                if time != Duration::ZERO {
                    sleep(time);
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
