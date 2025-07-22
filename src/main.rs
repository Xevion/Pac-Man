#![windows_subsystem = "windows"]

use crate::constants::{WINDOW_HEIGHT, WINDOW_WIDTH};
use crate::game::Game;
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use std::time::{Duration, Instant};
use tracing::event;
use tracing_error::ErrorLayer;
use tracing_subscriber::layer::SubscriberExt;

#[cfg(windows)]
use winapi::{
    shared::{ntdef::NULL, windef::HWND},
    um::{
        fileapi::{CreateFileA, OPEN_EXISTING},
        handleapi::INVALID_HANDLE_VALUE,
        processenv::SetStdHandle,
        winbase::{STD_ERROR_HANDLE, STD_OUTPUT_HANDLE},
        wincon::{AttachConsole, GetConsoleWindow},
        winnt::{FILE_SHARE_READ, FILE_SHARE_WRITE, GENERIC_READ, GENERIC_WRITE},
    },
};

#[cfg(windows)]
unsafe fn attach_console() {
    if GetConsoleWindow() != std::ptr::null_mut() as HWND {
        return;
    }

    if AttachConsole(winapi::um::wincon::ATTACH_PARENT_PROCESS) != 0 {
        let handle = CreateFileA(
            "CONOUT$\0".as_ptr() as *const i8,
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

mod animation;
mod audio;
mod constants;
mod direction;
mod entity;
mod game;
mod helper;
mod map;
mod modulation;
mod pacman;

pub fn main() {
    #[cfg(windows)]
    unsafe {
        attach_console();
    }

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();
    let ttf_context = sdl2::ttf::init().unwrap();

    // Setup tracing
    let subscriber = tracing_subscriber::fmt()
        .with_ansi(cfg!(not(target_os = "emscripten")))
        .with_max_level(tracing::Level::DEBUG)
        .finish()
        .with(ErrorLayer::default());

    tracing::subscriber::set_global_default(subscriber).expect("Could not set global default");

    let window = video_subsystem
        .window("Pac-Man", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .build()
        .expect("Could not initialize window");

    let mut canvas = window
        .into_canvas()
        .build()
        .expect("Could not build canvas");

    canvas
        .set_logical_size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .expect("Could not set logical size");

    let texture_creator = canvas.texture_creator();
    let mut game = Game::new(
        &mut canvas,
        &texture_creator,
        &ttf_context,
        &audio_subsystem,
    );

    let mut event_pump = sdl_context
        .event_pump()
        .expect("Could not get SDL EventPump");

    // Initial draw and tick
    game.draw();
    game.tick();

    let loop_time = Duration::from_secs(1) / 60;
    let mut tick_no = 0u32;

    // The start of a period of time over which we average the frame time.
    let mut last_averaging_time = Instant::now();
    let mut sleep_time = Duration::ZERO;
    let mut paused = false;
    let mut shown = false;

    event!(
        tracing::Level::INFO,
        "Starting game loop ({:.3}ms)",
        loop_time.as_secs_f32() * 1000.0
    );
    let mut main_loop = || {
        let start = Instant::now();

        // TODO: Fix key repeat delay issues by using VecDeque for instant key repeat
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
                    event!(
                        tracing::Level::INFO,
                        "{}",
                        if paused { "Paused" } else { "Unpaused" }
                    );
                }
                Event::KeyDown { keycode, .. } => {
                    game.keyboard_event(keycode.unwrap());
                }
                _ => {}
            }
        }

        // TODO: Proper pausing implementation that does not interfere with statistic gathering
        if !paused {
            // game.audio_demo_tick();
            game.tick();
            game.draw();
        }

        if start.elapsed() < loop_time {
            let time = loop_time.saturating_sub(start.elapsed());
            if time != Duration::ZERO {
                #[cfg(not(target_os = "emscripten"))]
                {
                    spin_sleep::sleep(time);
                }
                #[cfg(target_os = "emscripten")]
                {
                    std::thread::sleep(time);
                }
            }
            sleep_time += time;
        } else {
            event!(
                tracing::Level::WARN,
                "Game loop behind schedule by: {:?}",
                start.elapsed() - loop_time
            );
        }

        tick_no += 1;

        const PERIOD: u32 = 60 * 60;
        let tick_mod = tick_no % PERIOD;
        if tick_mod % PERIOD == 0 {
            let average_fps = PERIOD as f32 / last_averaging_time.elapsed().as_secs_f32();
            let average_sleep = sleep_time / PERIOD;
            let average_process = loop_time - average_sleep;

            sleep_time = Duration::ZERO;
            last_averaging_time = Instant::now();
        }

        true
    };

    loop {
        if !main_loop() {
            break;
        }
    }
}
