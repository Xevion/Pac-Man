#![windows_subsystem = "windows"]

use crate::{app::App, constants::LOOP_TIME};
use tracing::info;
use tracing_error::ErrorLayer;
use tracing_subscriber::layer::SubscriberExt;

mod app;
mod asset;
mod audio;
mod constants;

mod entity;
mod error;
mod game;
mod helpers;
mod input;
mod map;
mod platform;
mod texture;

/// The main entry point of the application.
///
/// This function initializes SDL, the window, the game state, and then enters
/// the main game loop.
pub fn main() {
    // Setup tracing
    let subscriber = tracing_subscriber::fmt()
        .with_ansi(cfg!(not(target_os = "emscripten")))
        .with_max_level(tracing::Level::DEBUG)
        .finish()
        .with(ErrorLayer::default());

    tracing::subscriber::set_global_default(subscriber).expect("Could not set global default");

    let mut app = App::new().expect("Could not create app");

    info!("Starting game loop ({:?})", LOOP_TIME);

    loop {
        if !app.run() {
            break;
        }
    }
}
