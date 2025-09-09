//! Centralized error types for the Pac-Man game.
//!
//! This module defines all error types used throughout the application,
//! providing a consistent error handling approach.

use std::io;

use bevy_ecs::event::Event;

/// Main error type for the Pac-Man game.
///
/// This is the primary error type that should be used in public APIs.
/// It can represent any error that can occur during game operation.
#[derive(thiserror::Error, Debug, Event)]
pub enum GameError {
    #[error("Asset error: {0}")]
    Asset(#[from] AssetError),

    #[error("Platform error: {0}")]
    Platform(#[from] PlatformError),

    #[error("Map parsing error: {0}")]
    MapParse(#[from] ParseError),

    #[error("Map error: {0}")]
    Map(#[from] MapError),

    #[error("Texture error: {0}")]
    Texture(#[from] TextureError),

    #[error("Entity error: {0}")]
    Entity(#[from] EntityError),

    #[error("SDL error: {0}")]
    Sdl(String),

    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Invalid state: {0}")]
    InvalidState(String),
}

#[derive(thiserror::Error, Debug)]
pub enum AssetError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    // This error is only possible on Emscripten, as the assets are loaded from a 'filesystem' of sorts (while on Desktop, they are included in the binary at compile time)
    #[allow(dead_code)]
    #[error("Asset not found: {0}")]
    NotFound(String),
}

/// Platform-specific errors.
#[derive(thiserror::Error, Debug)]
pub enum PlatformError {
    #[error("Console initialization failed: {0}")]
    #[cfg(any(windows, target_os = "emscripten"))]
    ConsoleInit(String),
}

/// Error type for map parsing operations.
#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("Unknown character in board: {0}")]
    UnknownCharacter(char),
    #[error("House door must have exactly 2 positions, found {0}")]
    InvalidHouseDoorCount(usize),
    #[error("Map parsing failed: {0}")]
    ParseFailed(String),
}

/// Errors related to texture operations.
#[derive(thiserror::Error, Debug)]
pub enum TextureError {
    #[error("Failed to load texture: {0}")]
    LoadFailed(String),

    #[error("Texture not found in atlas: {0}")]
    AtlasTileNotFound(String),

    #[error("Invalid texture format: {0}")]
    InvalidFormat(String),

    #[error("Rendering failed: {0}")]
    RenderFailed(String),
}

/// Errors related to entity operations.
#[derive(thiserror::Error, Debug)]
pub enum EntityError {
    #[error("Node not found in graph: {0}")]
    NodeNotFound(usize),

    #[error("Edge not found: from {from} to {to}")]
    EdgeNotFound { from: usize, to: usize },
}

/// Errors related to map operations.
#[derive(thiserror::Error, Debug)]
pub enum MapError {
    #[error("Node not found: {0}")]
    NodeNotFound(usize),

    #[error("Invalid map configuration: {0}")]
    InvalidConfig(String),
}

/// Result type for game operations.
pub type GameResult<T> = Result<T, GameError>;
