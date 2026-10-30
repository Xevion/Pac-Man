//! Centralized error types for the Pac-Man game.
//!
//! This module defines all error types used throughout the application,
//! providing a consistent error handling approach.

use std::fmt::Display;
use std::io;

use bevy_ecs::event::Event;
use snafu::{Location, Snafu};
use tracing::subscriber::SetGlobalDefaultError;

/// Main error type for the Pac-Man game.
///
/// This is the primary error type that should be used in public APIs.
/// It can represent any error that can occur during game operation.
#[derive(Debug, Snafu, Event)]
#[snafu(module, context(suffix(false)), visibility(pub(crate)))]
pub enum GameError {
    #[snafu(transparent)]
    Asset { source: AssetError },

    #[snafu(transparent)]
    Platform { source: PlatformError },

    #[snafu(transparent)]
    MapParse { source: ParseError },

    #[snafu(transparent)]
    Map { source: MapError },

    #[snafu(transparent)]
    Texture { source: TextureError },

    #[snafu(transparent)]
    Entity { source: EntityError },

    #[snafu(display("SDL {operation} failed: {message} ({location})"))]
    Sdl {
        operation: &'static str,
        message: String,
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display("Invalid state: {message} ({location})"))]
    InvalidState {
        message: String,
        #[snafu(implicit)]
        location: Location,
    },
}

/// Errors related to loading game assets.
#[derive(Debug, Snafu)]
#[snafu(module, context(suffix(false)), visibility(pub(crate)))]
pub enum AssetError {
    #[snafu(display("IO error reading asset {path} ({location})"))]
    Io {
        path: String,
        source: io::Error,
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display("Asset not found: {path} ({location})"))]
    NotFound {
        path: String,
        #[snafu(implicit)]
        location: Location,
    },
}

/// Platform-specific errors.
#[derive(Debug, Snafu)]
#[snafu(module, context(suffix(false)), visibility(pub(crate)))]
pub enum PlatformError {
    #[snafu(display("Console initialization failed: {message} ({location})"))]
    ConsoleInit {
        message: String,
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display("Failed to create log file {path} ({location})"))]
    LogFile {
        path: &'static str,
        source: io::Error,
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display("Failed to set tracing subscriber ({location})"))]
    Subscriber {
        source: SetGlobalDefaultError,
        #[snafu(implicit)]
        location: Location,
    },
}

/// Error type for map parsing operations.
#[derive(Debug, Snafu)]
#[snafu(module, context(suffix(false)), visibility(pub(crate)))]
pub enum ParseError {
    #[snafu(display("Unknown character in board: {character} ({location})"))]
    UnknownCharacter {
        character: char,
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display("House door must have exactly 2 positions, found {count} ({location})"))]
    InvalidHouseDoorCount {
        count: usize,
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display("Map parsing failed: {message} ({location})"))]
    ParseFailed {
        message: String,
        #[snafu(implicit)]
        location: Location,
    },
}

/// Errors related to texture operations.
#[derive(Debug, Snafu)]
#[snafu(module, context(suffix(false)), visibility(pub(crate)))]
pub enum TextureError {
    #[snafu(display("Failed to load texture: {message} ({location})"))]
    LoadFailed {
        message: String,
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display("Texture not found in atlas: {name} ({location})"))]
    AtlasTileNotFound {
        name: String,
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display("Invalid texture format: {message} ({location})"))]
    InvalidFormat {
        message: String,
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display("Rendering failed: {message} ({location})"))]
    RenderFailed {
        message: String,
        #[snafu(implicit)]
        location: Location,
    },
}

/// Errors related to entity operations.
#[derive(Debug, Snafu)]
#[snafu(module, context(suffix(false)), visibility(pub(crate)))]
pub enum EntityError {
    #[snafu(display("Node not found in graph: {node} ({location})"))]
    NodeNotFound {
        node: usize,
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display("Edge not found: from {from} to {to} ({location})"))]
    EdgeNotFound {
        from: usize,
        to: usize,
        #[snafu(implicit)]
        location: Location,
    },
}

/// Errors related to audio initialization and playback.
#[derive(Debug, Snafu)]
#[snafu(module, context(suffix(false)), visibility(pub(crate)))]
pub enum AudioError {
    #[snafu(display("Failed to initialize audio: {message} ({location})"))]
    InitFailed {
        message: String,
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display("Failed to load sound: {message} ({location})"))]
    LoadFailed {
        message: String,
        #[snafu(implicit)]
        location: Location,
    },
}

/// Errors related to map operations.
#[derive(Debug, Snafu)]
#[snafu(module, context(suffix(false)), visibility(pub(crate)))]
pub enum MapError {
    #[snafu(display("Node not found: {node} ({location})"))]
    NodeNotFound {
        node: usize,
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display("Invalid map configuration: {message} ({location})"))]
    InvalidConfig {
        message: String,
        #[snafu(implicit)]
        location: Location,
    },
}

/// Result type for game operations.
pub type GameResult<T> = Result<T, GameError>;

/// Converts the error of an SDL call into a [`GameError::Sdl`] naming the operation.
pub trait SdlResultExt<T> {
    /// Wraps an `Err` as [`GameError::Sdl`] located at the caller.
    #[track_caller]
    fn sdl(self, operation: &'static str) -> GameResult<T>;
}

impl<T, E: Display> SdlResultExt<T> for Result<T, E> {
    #[track_caller]
    fn sdl(self, operation: &'static str) -> GameResult<T> {
        match self {
            Ok(value) => Ok(value),
            Err(error) => Err(game_error::Sdl {
                operation,
                message: error.to_string(),
            }
            .build()),
        }
    }
}
