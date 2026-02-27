//! Ghost personality targeting calculations - Blinky, Pinky, Inky, Clyde.

use super::targeting::find_nearest_node;
use super::GhostType;
use crate::map::builder::Map;
use crate::map::direction::Direction;
use crate::systems::NodeId;
use glam::Vec2;

const TILE_SIZE: f32 = 8.0;

/// Context provided for chase mode targeting calculations
pub struct TargetingContext {
    /// Pac-Man's current node
    pub pacman_node: NodeId,
    /// Pac-Man's current direction
    pub pacman_direction: Direction,
    /// Pac-Man's pixel position (for distance calculations)
    pub pacman_position: Vec2,
    /// Blinky's pixel position (needed for Inky's vector doubling calculation)
    pub blinky_position: Vec2,
    /// This ghost's current node
    pub self_node: NodeId,
    /// This ghost's pixel position
    pub self_position: Vec2,
}

/// Calculate chase target for a specific ghost personality
pub fn calculate_chase_target(ghost_type: GhostType, ctx: &TargetingContext, map: &Map) -> NodeId {
    match ghost_type {
        GhostType::Blinky => blinky_chase_target(ctx),
        GhostType::Pinky => pinky_chase_target(ctx, map),
        GhostType::Inky => inky_chase_target(ctx, map),
        GhostType::Clyde => clyde_chase_target(ctx, map),
    }
}

/// Blinky: "Shadow" - Directly targets Pac-Man
fn blinky_chase_target(ctx: &TargetingContext) -> NodeId {
    ctx.pacman_node
}

/// Pinky: "Speedy" / "Ambush" - Targets 4 tiles ahead of Pac-Man
fn pinky_chase_target(ctx: &TargetingContext, map: &Map) -> NodeId {
    // Target 4 tiles ahead of Pac-Man
    // NOTE: Original has overflow bug when Pac-Man faces up (also offsets left)
    // We implement the bug for authenticity
    let offset = match ctx.pacman_direction {
        Direction::Up => Vec2::new(-4.0, -4.0), // Bug: also goes left
        Direction::Down => Vec2::new(0.0, 4.0),
        Direction::Left => Vec2::new(-4.0, 0.0),
        Direction::Right => Vec2::new(4.0, 0.0),
    } * TILE_SIZE;

    let target_pos = ctx.pacman_position + offset;
    find_nearest_node(map, target_pos)
}

/// Inky: "Bashful" / "Fickle" - Uses Blinky + Pac-Man vector doubling
fn inky_chase_target(ctx: &TargetingContext, map: &Map) -> NodeId {
    // 1. Get position 2 tiles ahead of Pac-Man (with up-direction bug)
    let offset = match ctx.pacman_direction {
        Direction::Up => Vec2::new(-2.0, -2.0), // Bug: also goes left
        Direction::Down => Vec2::new(0.0, 2.0),
        Direction::Left => Vec2::new(-2.0, 0.0),
        Direction::Right => Vec2::new(2.0, 0.0),
    } * TILE_SIZE;
    let intermediate = ctx.pacman_position + offset;

    // 2. Draw vector from Blinky to intermediate, then double it
    let vector = intermediate - ctx.blinky_position;
    let target_pos = ctx.blinky_position + vector * 2.0;

    find_nearest_node(map, target_pos)
}

/// Clyde: "Pokey" / "Feigning ignorance" - Chases when far, scatters when close
fn clyde_chase_target(ctx: &TargetingContext, _map: &Map) -> NodeId {
    // If distance to Pac-Man >= 8 tiles, target Pac-Man
    // Otherwise, would target scatter corner (handled by caller checking distance)
    let distance_sq = ctx.self_position.distance_squared(ctx.pacman_position);
    let threshold = 8.0 * TILE_SIZE;

    if distance_sq >= threshold * threshold {
        ctx.pacman_node
    } else {
        // Signal to use scatter target instead
        // The movement system will check this
        ctx.self_node
    }
}
