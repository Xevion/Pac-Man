// Note: This disables the console window on Windows. We manually re-attach to the parent terminal or process later on.
#![windows_subsystem = "windows"]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use crate::{app::App, constants::LOOP_TIME};
use tracing::info;

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
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn main() {
    // On Windows, this connects output streams to the console dynamically
    // On Emscripten, this connects the subscriber to the browser console
    platform::init_console().expect("Could not initialize console");

    let mut app = App::new().expect("Could not create app");

    info!(loop_time = ?LOOP_TIME, "Starting game loop");

    loop {
        if !app.run() {
            break;
        }
    }
}
