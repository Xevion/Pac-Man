// Note: This disables the console window on Windows. We manually re-attach to the parent terminal or process later on.
#![windows_subsystem = "windows"]

use crate::{app::App, constants::LOOP_TIME};
use tracing::{debug, info, warn};

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
    let platform = platform::get_platform();
    if platform.requires_console() {
        // Setup buffered tracing subscriber that will buffer logs until console is ready
        let switchable_writer = platform::tracing_buffer::setup_switchable_subscriber();

        // Initialize platform-specific console
        platform.init_console().expect("Could not initialize console");

        // Now that console is initialized, flush buffered logs and switch to direct output
        debug!("Switching to direct logging mode and flushing buffer...");
        if let Err(error) = switchable_writer.switch_to_direct_mode() {
            warn!("Failed to flush buffered logs to console: {error:?}");
        }
    }

    let mut app = App::new().expect("Could not create app");

    info!(loop_time = ?LOOP_TIME, "Starting game loop");

    loop {
        if !app.run() {
            break;
        }
    }
}
