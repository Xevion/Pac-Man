//! Entity trait for common movement and rendering functionality.
//!
//! This module defines a trait that captures the shared behavior between
//! different game entities like Ghosts and Pac-Man, including movement,
//! rendering, and position calculations.

use glam::Vec2;
use sdl2::render::{Canvas, RenderTarget};

use crate::entity::direction::Direction;
use crate::entity::graph::{Edge, Graph, NodeId};
use crate::entity::traversal::{Position, Traverser};
use crate::error::{EntityError, GameError, GameResult, TextureError};
use crate::texture::directional::DirectionalAnimatedTexture;
use crate::texture::sprite::SpriteAtlas;

/// Trait defining common functionality for game entities that move through the graph.
///
/// This trait provides a unified interface for entities that:
/// - Move through the game graph using a traverser
/// - Render using directional animated textures
/// - Have position calculations and movement speed
#[allow(dead_code)]
pub trait Entity {
    /// Returns a reference to the entity's traverser for movement control.
    fn traverser(&self) -> &Traverser;

    /// Returns a mutable reference to the entity's traverser for movement control.
    fn traverser_mut(&mut self) -> &mut Traverser;

    /// Returns a reference to the entity's directional animated texture.
    fn texture(&self) -> &DirectionalAnimatedTexture;

    /// Returns a mutable reference to the entity's directional animated texture.
    fn texture_mut(&mut self) -> &mut DirectionalAnimatedTexture;

    /// Returns the movement speed multiplier for this entity.
    fn speed(&self) -> f32;

    /// Determines if this entity can traverse a given edge.
    fn can_traverse(&self, edge: Edge) -> bool;

    /// Updates the entity's position and animation state.
    ///
    /// This method advances movement through the graph and updates texture animation.
    fn tick(&mut self, dt: f32, graph: &Graph);

    /// Calculates the current pixel position in the game world.
    ///
    /// Converts the graph position to screen coordinates, accounting for
    /// the board offset and centering the sprite.
    fn get_pixel_pos(&self, graph: &Graph) -> GameResult<Vec2> {
        let pos = match self.traverser().position {
            Position::AtNode(node_id) => {
                let node = graph.get_node(node_id).ok_or(EntityError::NodeNotFound(node_id))?;
                node.position
            }
            Position::BetweenNodes { from, to, traversed } => {
                let from_node = graph.get_node(from).ok_or(EntityError::NodeNotFound(from))?;
                let to_node = graph.get_node(to).ok_or(EntityError::NodeNotFound(to))?;
                let edge = graph.find_edge(from, to).ok_or(EntityError::EdgeNotFound { from, to })?;
                from_node.position + (to_node.position - from_node.position) * (traversed / edge.distance)
            }
        };

        Ok(Vec2::new(
            pos.x + crate::constants::BOARD_PIXEL_OFFSET.x as f32,
            pos.y + crate::constants::BOARD_PIXEL_OFFSET.y as f32,
        ))
    }

    /// Returns the current node ID that the entity is at or moving towards.
    ///
    /// If the entity is at a node, returns that node ID.
    /// If the entity is between nodes, returns the node it's moving towards.
    fn current_node_id(&self) -> NodeId {
        match self.traverser().position {
            Position::AtNode(node_id) => node_id,
            Position::BetweenNodes { to, .. } => to,
        }
    }

    /// Sets the next direction for the entity to take.
    ///
    /// The direction is buffered and will be applied at the next opportunity,
    /// typically when the entity reaches a new node.
    fn set_next_direction(&mut self, direction: Direction) {
        self.traverser_mut().set_next_direction(direction);
    }

    /// Renders the entity at its current position.
    ///
    /// Draws the appropriate directional sprite based on the entity's
    /// current movement state and direction.
    fn render<T: RenderTarget>(&self, canvas: &mut Canvas<T>, atlas: &mut SpriteAtlas, graph: &Graph) -> GameResult<()> {
        let pixel_pos = self.get_pixel_pos(graph)?;
        let dest = crate::helpers::centered_with_size(
            glam::IVec2::new(pixel_pos.x as i32, pixel_pos.y as i32),
            glam::UVec2::new(16, 16),
        );

        if self.traverser().position.is_stopped() {
            self.texture()
                .render_stopped(canvas, atlas, dest, self.traverser().direction)
                .map_err(|e| GameError::Texture(TextureError::RenderFailed(e.to_string())))?;
        } else {
            self.texture()
                .render(canvas, atlas, dest, self.traverser().direction)
                .map_err(|e| GameError::Texture(TextureError::RenderFailed(e.to_string())))?;
        }

        Ok(())
    }
}
