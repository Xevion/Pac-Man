#![windows_subsystem = "windows"]

use crate::{app::App, constants::LOOP_TIME};
use tracing::info;

mod app;
mod asset;
mod audio;
mod constants;

mod error;
mod events;
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
    // Setup buffered tracing subscriber that will buffer logs until console is ready
    let switchable_writer = platform::tracing_buffer::setup_switchable_subscriber();

    // Log early to show buffering is working
    tracing::debug!("Tracing subscriber initialized with buffering - logs will be buffered until console is ready");

    // Initialize platform-specific console
    tracing::debug!("Starting console initialization...");
    platform::get_platform().init_console().expect("Could not initialize console");
    tracing::debug!("Console initialization completed");

    // Now that console is initialized, flush buffered logs and switch to direct output
    tracing::debug!("Switching to direct logging mode and flushing buffer...");
    if let Err(e) = switchable_writer.switch_to_direct_mode() {
        tracing::warn!("Failed to flush buffered logs to console: {}", e);
    }

    let mut app = App::new().expect("Could not create app");

    info!("Starting game loop ({:?})", LOOP_TIME);

    loop {
        if !app.run() {
            break;
        }
    }
}
