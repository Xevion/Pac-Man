use crate::constants::{WINDOW_HEIGHT, WINDOW_WIDTH};
use crate::game::Game;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use spin_sleep::sleep;
use std::time::{Duration, Instant};
use tracing::event;
use tracing_error::ErrorLayer;
use tracing_subscriber::layer::SubscriberExt;

mod animation;
mod constants;
mod direction;
mod entity;
mod game;
mod map;
mod modulation;
mod pacman;

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

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
        .accelerated()
        .build()
        .expect("Could not build canvas");

    canvas
        .set_logical_size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .expect("Could not set logical size");

    let texture_creator = canvas.texture_creator();
    let mut game = Game::new(&mut canvas, &texture_creator);

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
            game.tick();
            game.draw();
        }

        if start.elapsed() < loop_time {
            let time = loop_time - start.elapsed();
            sleep(time);
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

            event!(
                tracing::Level::DEBUG,
                "Timing Averages [fps={}] [sleep={:?}] [process={:?}]",
                average_fps,
                average_sleep,
                average_process
            );

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
