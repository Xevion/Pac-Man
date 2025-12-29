#![cfg_attr(all(not(use_console), target_os = "windows"), windows_subsystem = "windows")]
#![cfg_attr(all(use_console, target_os = "windows"), windows_subsystem = "console")]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![cfg_attr(coverage_nightly, coverage(off))]

use std::env;

use crate::{app::App, constants::LOOP_TIME};
use tracing::info;

// These modules are excluded from coverage.
#[cfg_attr(coverage_nightly, coverage(off))]
mod app;
#[cfg_attr(coverage_nightly, coverage(off))]
mod audio;
#[cfg_attr(coverage_nightly, coverage(off))]
mod error;
#[cfg_attr(coverage_nightly, coverage(off))]
mod events;
#[cfg_attr(coverage_nightly, coverage(off))]
mod formatter;
#[cfg_attr(coverage_nightly, coverage(off))]
mod platform;

mod asset;
mod constants;
mod game;
mod map;
mod systems;
mod texture;

// Emscripten-specific: static storage for the App instance
// Required because emscripten_set_main_loop_arg needs a persistent pointer
#[cfg(target_os = "emscripten")]
static mut APP: Option<App> = None;

/// Called from JavaScript when the user interacts with the page.
/// Transitions the game from WaitingForInteraction to Starting state.
#[cfg(target_os = "emscripten")]
#[no_mangle]
pub extern "C" fn start_game() {
    unsafe {
        if let Some(ref mut app) = APP {
            app.game.start();
        }
    }
}

/// Called from JavaScript when navigating away from the game page.
/// Stops the Emscripten main loop and halts all audio.
#[cfg(target_os = "emscripten")]
#[no_mangle]
pub extern "C" fn stop_game() {
    tracing::info!("Stopping game loop and halting audio");
    unsafe {
        platform::emscripten_cancel_main_loop();
        sdl2::mixer::Channel::all().halt();
    }
}

/// Called from JavaScript to restart the game after navigating back.
/// Creates a fresh App instance with the new canvas and starts the main loop.
#[cfg(target_os = "emscripten")]
#[no_mangle]
pub extern "C" fn restart_game() {
    use std::ptr;

    tracing::info!("Restarting game with fresh App instance");

    unsafe {
        // Drop old App to clean up resources
        APP = None;

        // Reinitialize audio subsystem for fresh state
        sdl2::mixer::close_audio();

        // Create fresh App with new canvas
        match App::new() {
            Ok(app) => {
                APP = Some(app);
                tracing::info!("Game restarted successfully");

                // Signal ready and start the main loop
                platform::run_script("if (window.pacmanReady) window.pacmanReady()");
                platform::emscripten_set_main_loop_arg(main_loop_callback, ptr::null_mut(), 0, 1);
            }
            Err(e) => {
                tracing::error!("Failed to restart game: {}", e);
            }
        }
    }
}

/// Emscripten main loop callback - runs once per frame
#[cfg(target_os = "emscripten")]
unsafe extern "C" fn main_loop_callback(_arg: *mut std::ffi::c_void) {
    if let Some(ref mut app) = APP {
        let _ = app.run();
    }
}

/// The main entry point of the application.
///
/// This function initializes SDL, the window, the game state, and then enters
/// the main game loop.
pub fn main() {
    // Parse command line arguments (only on desktop - Emscripten ignores force_console)
    #[cfg(not(target_os = "emscripten"))]
    let force_console = {
        let args: Vec<String> = env::args().collect();
        args.iter().any(|arg| arg == "--console" || arg == "-c")
    };
    #[cfg(target_os = "emscripten")]
    let force_console = false;

    // On Emscripten, this connects the subscriber to the browser console
    platform::init_console(force_console).expect("Could not initialize console");

    let app = App::new().expect("Could not create app");

    info!(loop_time = ?LOOP_TIME, "Starting game loop");

    #[cfg(target_os = "emscripten")]
    {
        use std::ptr;

        // Store app in static for callback access
        unsafe {
            APP = Some(app);
        }

        // Signal to JavaScript that the game is ready for interaction
        platform::run_script("if (window.pacmanReady) window.pacmanReady()");

        // Use emscripten_set_main_loop_arg for browser-friendly loop
        // fps=0 means use requestAnimationFrame for optimal performance
        // simulate_infinite_loop=1 means this call won't return
        unsafe {
            platform::emscripten_set_main_loop_arg(main_loop_callback, ptr::null_mut(), 0, 1);
        }
    }

    #[cfg(not(target_os = "emscripten"))]
    {
        let mut app = app;
        loop {
            if !app.run() {
                break;
            }
        }
    }
}
