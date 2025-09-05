// Note: This disables the console window on Windows. We manually re-attach to the parent terminal or process later on.
#![windows_subsystem = "windows"]

use crate::{app::App, constants::LOOP_TIME};
use tracing::info;

mod app;
mod asset;
mod audio;
mod constants;

mod error;
mod events;
mod formatter;
mod game;
mod map;
mod platform;
mod systems;
mod texture;

/// The main entry point of the application.
///
/// This function initializes SDL, the window, the game state, and then enters
/// the main game loop.
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
