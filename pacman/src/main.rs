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

/// The main entry point of the application.
///
/// This function initializes SDL, the window, the game state, and then enters
/// the main game loop.
pub fn main() {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let force_console = args.iter().any(|arg| arg == "--console" || arg == "-c");

    // On Emscripten, this connects the subscriber to the browser console
    platform::init_console(force_console).expect("Could not initialize console");

    let mut app = App::new().expect("Could not create app");

    info!(loop_time = ?LOOP_TIME, "Starting game loop");

    loop {
        if !app.run() {
            break;
        }
    }
}
