#![windows_subsystem = "windows"]

use crate::constants::{BOARD_PIXEL_SIZE, SCALE};
use crate::game::Game;
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use std::time::{Duration, Instant};
use tracing::event;
use tracing_error::ErrorLayer;
use tracing_subscriber::layer::SubscriberExt;

#[cfg(windows)]
use winapi::{
    shared::ntdef::NULL,
    um::{
        fileapi::{CreateFileA, OPEN_EXISTING},
        handleapi::INVALID_HANDLE_VALUE,
        processenv::SetStdHandle,
        winbase::{STD_ERROR_HANDLE, STD_OUTPUT_HANDLE},
        wincon::{AttachConsole, GetConsoleWindow},
        winnt::{FILE_SHARE_READ, FILE_SHARE_WRITE, GENERIC_READ, GENERIC_WRITE},
    },
};

/// Attaches the process to the parent console on Windows.
///
/// This allows the application to print to the console when run from a terminal,
/// which is useful for debugging purposes. If the application is not run from a
/// terminal, this function does nothing.
#[cfg(windows)]
unsafe fn attach_console() {
    if !std::ptr::eq(GetConsoleWindow(), std::ptr::null_mut()) {
        return;
    }

    if AttachConsole(winapi::um::wincon::ATTACH_PARENT_PROCESS) != 0 {
        let handle = CreateFileA(
            c"CONOUT$".as_ptr(),
            GENERIC_READ | GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            std::ptr::null_mut(),
            OPEN_EXISTING,
            0,
            NULL,
        );

        if handle != INVALID_HANDLE_VALUE {
            SetStdHandle(STD_OUTPUT_HANDLE, handle);
            SetStdHandle(STD_ERROR_HANDLE, handle);
        }
    }
    // Do NOT call AllocConsole here - we don't want a console when launched from Explorer
}

mod asset;
mod audio;
mod constants;
mod debug;
#[cfg(target_os = "emscripten")]
mod emscripten;
mod entity;
mod game;
mod helper;
mod map;
mod texture;

#[cfg(not(target_os = "emscripten"))]
fn sleep(value: Duration) {
    spin_sleep::sleep(value);
}

#[cfg(target_os = "emscripten")]
fn sleep(value: Duration) {
    emscripten::emscripten::sleep(value.as_millis() as u32);
}

/// The main entry point of the application.
///
/// This function initializes SDL, the window, the game state, and then enters
/// the main game loop.
pub fn main() {
    // Attaches the console on Windows for debugging purposes.
    #[cfg(windows)]
    unsafe {
        attach_console();
    }

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();
    let ttf_context = sdl2::ttf::init().unwrap();

    // Set nearest-neighbor scaling for pixelated rendering
    sdl2::hint::set("SDL_RENDER_SCALE_QUALITY", "nearest");

    // Setup tracing
    let subscriber = tracing_subscriber::fmt()
        .with_ansi(cfg!(not(target_os = "emscripten")))
        .with_max_level(tracing::Level::DEBUG)
        .finish()
        .with(ErrorLayer::default());

    tracing::subscriber::set_global_default(subscriber).expect("Could not set global default");

    let window = video_subsystem
        .window(
            "Pac-Man",
            (BOARD_PIXEL_SIZE.x as f32 * SCALE).round() as u32,
            (BOARD_PIXEL_SIZE.y as f32 * SCALE).round() as u32,
        )
        .resizable()
        .position_centered()
        .build()
        .expect("Could not initialize window");

    let mut canvas = window.into_canvas().build().expect("Could not build canvas");

    canvas
        .set_logical_size(BOARD_PIXEL_SIZE.x, BOARD_PIXEL_SIZE.y)
        .expect("Could not set logical size");

    let texture_creator = canvas.texture_creator();
    let texture_creator_static: &'static sdl2::render::TextureCreator<sdl2::video::WindowContext> =
        Box::leak(Box::new(texture_creator));
    let mut game = Game::new(texture_creator_static, &ttf_context, &audio_subsystem);
    game.audio.set_mute(cfg!(debug_assertions));

    // Create a backbuffer texture for drawing
    let mut backbuffer = texture_creator_static
        .create_texture_target(None, BOARD_PIXEL_SIZE.x, BOARD_PIXEL_SIZE.y)
        .expect("Could not create backbuffer texture");

    let mut event_pump = sdl_context.event_pump().expect("Could not get SDL EventPump");

    // Initial draw and tick
    if let Err(e) = game.draw(&mut canvas, &mut backbuffer) {
        eprintln!("Initial draw failed: {}", e);
    }
    if let Err(e) = game.present_backbuffer(&mut canvas, &backbuffer) {
        eprintln!("Initial present failed: {}", e);
    }
    game.tick();

    // The target time for each frame of the game loop (60 FPS).
    let loop_time = Duration::from_secs(1) / 60;

    let mut paused = false;
    // Whether the window is currently shown.
    let mut shown = false;

    // FPS tracking
    let mut frame_times_1s = Vec::new();
    let mut frame_times_10s = Vec::new();
    let mut last_frame_time = Instant::now();

    event!(tracing::Level::INFO, "Starting game loop ({:?})", loop_time);
    let mut main_loop = || {
        let start = Instant::now();
        let current_frame_time = Instant::now();
        let frame_duration = current_frame_time.duration_since(last_frame_time);
        last_frame_time = current_frame_time;

        // Update FPS tracking
        frame_times_1s.push(frame_duration);
        frame_times_10s.push(frame_duration);

        // Keep only last 1 second of data (assuming 60 FPS = ~60 frames)
        while frame_times_1s.len() > 60 {
            frame_times_1s.remove(0);
        }

        // Keep only last 10 seconds of data
        while frame_times_10s.len() > 600 {
            frame_times_10s.remove(0);
        }

        // Calculate FPS averages
        let fps_1s = if !frame_times_1s.is_empty() {
            let total_time: Duration = frame_times_1s.iter().sum();
            if total_time > Duration::ZERO {
                frame_times_1s.len() as f64 / total_time.as_secs_f64()
            } else {
                0.0
            }
        } else {
            0.0
        };

        let fps_10s = if !frame_times_10s.is_empty() {
            let total_time: Duration = frame_times_10s.iter().sum();
            if total_time > Duration::ZERO {
                frame_times_10s.len() as f64 / total_time.as_secs_f64()
            } else {
                0.0
            }
        } else {
            0.0
        };

        // TODO: Fix key repeat delay issues by using a queue for keyboard events.
        // This would allow for instant key repeat without being affected by the
        // main loop's tick rate.
        for event in event_pump.poll_iter() {
            match event {
                Event::Window { win_event, .. } => match win_event {
                    WindowEvent::Hidden => {
                        event!(tracing::Level::DEBUG, "Window hidden");
                        shown = false;
                    }
                    WindowEvent::Shown => {
                        event!(tracing::Level::DEBUG, "Window shown");
                        shown = true;
                    }
                    _ => {}
                },
                // Handle quitting keys or window close
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
                    paused = !paused;
                    event!(tracing::Level::INFO, "{}", if paused { "Paused" } else { "Unpaused" });
                }
                Event::KeyDown { keycode, .. } => {
                    game.keyboard_event(keycode.unwrap());
                }
                _ => {}
            }
        }

        // TODO: Implement a proper pausing mechanism that does not interfere with
        // statistic gathering and other background tasks.
        if !paused {
            game.tick();
            if let Err(e) = game.draw(&mut canvas, &mut backbuffer) {
                eprintln!("Failed to draw game: {}", e);
            }
            if let Err(e) = game.present_backbuffer(&mut canvas, &backbuffer) {
                eprintln!("Failed to present backbuffer: {}", e);
            }
        }

        // Update game with FPS data
        game.update_fps(fps_1s, fps_10s);

        if start.elapsed() < loop_time {
            let time = loop_time.saturating_sub(start.elapsed());
            if time != Duration::ZERO {
                sleep(time);
            }
        } else {
            event!(
                tracing::Level::WARN,
                "Game loop behind schedule by: {:?}",
                start.elapsed() - loop_time
            );
        }

        true
    };

    loop {
        if !main_loop() {
            break;
        }
    }
}
