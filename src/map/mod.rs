//! This module defines the game map and provides functions for interacting with it.

pub mod builder;
pub mod layout;
pub mod parser;
pub mod render;

// Re-export main types for convenience
pub use builder::Map;
