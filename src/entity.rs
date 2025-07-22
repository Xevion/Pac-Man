//! This module defines the `Entity` trait, which is implemented by all game
//! objects that can be moved and rendered.

/// A trait for game objects that can be moved and rendered.
pub trait Entity {
    /// Returns true if the entity is colliding with the other entity.
    fn is_colliding(&self, other: &dyn Entity) -> bool;
    /// Returns the absolute position of the entity, in pixels.
    fn position(&self) -> (i32, i32);
    /// Returns the cell position of the entity, in grid coordinates.
    fn cell_position(&self) -> (u32, u32);
    /// Returns the position of the entity within its current cell, in pixels.
    fn internal_position(&self) -> (u32, u32);
    /// Ticks the entity, which updates its state and position.
    fn tick(&mut self);
}
