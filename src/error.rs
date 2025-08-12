//! Centralized error types for the Pac-Man game.
//!
//! This module defines all error types used throughout the application,
//! providing a consistent error handling approach.

use thiserror::Error;

/// Main error type for the Pac-Man game.
///
/// This is the primary error type that should be used in public APIs.
/// It can represent any error that can occur during game operation.
#[derive(Error, Debug)]
pub enum GameError {
    #[error("Asset error: {0}")]
    Asset(#[from] crate::asset::AssetError),

    #[error("Platform error: {0}")]
    Platform(#[from] crate::platform::PlatformError),

    #[error("Map parsing error: {0}")]
    MapParse(#[from] crate::map::parser::ParseError),

    #[error("Map error: {0}")]
    Map(#[from] MapError),

    #[error("Texture error: {0}")]
    Texture(#[from] TextureError),

    #[error("Entity error: {0}")]
    Entity(#[from] EntityError),

    #[error("Game state error: {0}")]
    GameState(#[from] GameStateError),

    #[error("SDL error: {0}")]
    Sdl(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Configuration error: {0}")]
    Config(String),
}

/// Errors related to texture operations.
#[derive(Error, Debug)]
pub enum TextureError {
    #[error("Animated texture error: {0}")]
    Animated(#[from] crate::texture::animated::AnimatedTextureError),

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
#[derive(Error, Debug)]
pub enum EntityError {
    #[error("Node not found in graph: {0}")]
    NodeNotFound(usize),

    #[error("Edge not found: from {from} to {to}")]
    EdgeNotFound { from: usize, to: usize },

    #[error("Invalid movement: {0}")]
    InvalidMovement(String),

    #[error("Pathfinding failed: {0}")]
    PathfindingFailed(String),
}

/// Errors related to game state operations.
#[derive(Error, Debug)]
pub enum GameStateError {}

/// Errors related to map operations.
#[derive(Error, Debug)]
pub enum MapError {
    #[error("Node not found: {0}")]
    NodeNotFound(usize),

    #[error("Invalid map configuration: {0}")]
    InvalidConfig(String),
}

/// Result type for game operations.
pub type GameResult<T> = Result<T, GameError>;

/// Helper trait for converting other error types to GameError.
pub trait IntoGameError<T> {
    #[allow(dead_code)]
    fn into_game_error(self) -> GameResult<T>;
}

impl<T, E> IntoGameError<T> for Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn into_game_error(self) -> GameResult<T> {
        self.map_err(|e| GameError::InvalidState(e.to_string()))
    }
}

/// Helper trait for converting Option to GameResult with a custom error.
pub trait OptionExt<T> {
    #[allow(dead_code)]
    fn ok_or_game_error<F>(self, f: F) -> GameResult<T>
    where
        F: FnOnce() -> GameError;
}

impl<T> OptionExt<T> for Option<T> {
    fn ok_or_game_error<F>(self, f: F) -> GameResult<T>
    where
        F: FnOnce() -> GameError,
    {
        self.ok_or_else(f)
    }
}

/// Helper trait for converting Result to GameResult with context.
pub trait ResultExt<T, E> {
    #[allow(dead_code)]
    fn with_context<F>(self, f: F) -> GameResult<T>
    where
        F: FnOnce(&E) -> GameError;
}

impl<T, E> ResultExt<T, E> for Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn with_context<F>(self, f: F) -> GameResult<T>
    where
        F: FnOnce(&E) -> GameError,
    {
        self.map_err(|e| f(&e))
    }
}
