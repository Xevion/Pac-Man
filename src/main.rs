#![windows_subsystem = "windows"]

use crate::{app::App, constants::LOOP_TIME};
use tracing::info;
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

mod app;
mod asset;
mod audio;
mod constants;
#[cfg(target_os = "emscripten")]
mod emscripten;
mod entity;
mod game;
mod map;
mod texture;

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
