//! Pac-Man entity implementation.
//!
//! This module contains the main player character logic, including movement,
//! animation, and rendering. Pac-Man moves through the game graph using
//! a traverser and displays directional animated textures.

use crate::entity::direction::Direction;
use crate::entity::graph::{Edge, EdgePermissions, Graph, NodeId};
use crate::entity::r#trait::Entity;
use crate::entity::traversal::Traverser;
use crate::texture::animated::AnimatedTexture;
use crate::texture::directional::DirectionalAnimatedTexture;
use crate::texture::sprite::SpriteAtlas;
use sdl2::keyboard::Keycode;

use crate::error::{GameError, GameResult, TextureError};

/// Determines if Pac-Man can traverse a given edge.
///
/// Pac-Man can only move through edges that allow all entities.
fn can_pacman_traverse(edge: Edge) -> bool {
    matches!(edge.permissions, EdgePermissions::All)
}

/// The main player character entity.
///
/// Pac-Man moves through the game world using a graph-based navigation system
/// and displays directional animated sprites based on movement state.
pub struct Pacman {
    /// Handles movement through the game graph
    pub traverser: Traverser,
    /// Manages directional animated textures for different movement states
    texture: DirectionalAnimatedTexture,
}

impl Entity for Pacman {
    fn traverser(&self) -> &Traverser {
        &self.traverser
    }

    fn traverser_mut(&mut self) -> &mut Traverser {
        &mut self.traverser
    }

    fn texture(&self) -> &DirectionalAnimatedTexture {
        &self.texture
    }

    fn texture_mut(&mut self) -> &mut DirectionalAnimatedTexture {
        &mut self.texture
    }

    fn speed(&self) -> f32 {
        1.125
    }

    fn can_traverse(&self, edge: Edge) -> bool {
        can_pacman_traverse(edge)
    }

    fn tick(&mut self, dt: f32, graph: &Graph) {
        if let Err(e) = self.traverser.advance(graph, dt * 60.0 * 1.125, &can_pacman_traverse) {
            eprintln!("Pac-Man movement error: {}", e);
        }
        self.texture.tick(dt);
    }
}

impl Pacman {
    /// Creates a new Pac-Man instance at the specified starting node.
    ///
    /// Sets up animated textures for all four directions with moving and stopped states.
    /// The moving animation cycles through open mouth, closed mouth, and full sprites.
    pub fn new(graph: &Graph, start_node: NodeId, atlas: &SpriteAtlas) -> GameResult<Self> {
        let mut textures = [None, None, None, None];
        let mut stopped_textures = [None, None, None, None];

        for direction in Direction::DIRECTIONS {
            let moving_prefix = match direction {
                Direction::Up => "pacman/up",
                Direction::Down => "pacman/down",
                Direction::Left => "pacman/left",
                Direction::Right => "pacman/right",
            };
            let moving_tiles = vec![
                SpriteAtlas::get_tile(atlas, &format!("{moving_prefix}_a.png"))
                    .ok_or_else(|| GameError::Texture(TextureError::AtlasTileNotFound(format!("{moving_prefix}_a.png"))))?,
                SpriteAtlas::get_tile(atlas, &format!("{moving_prefix}_b.png"))
                    .ok_or_else(|| GameError::Texture(TextureError::AtlasTileNotFound(format!("{moving_prefix}_b.png"))))?,
                SpriteAtlas::get_tile(atlas, "pacman/full.png")
                    .ok_or_else(|| GameError::Texture(TextureError::AtlasTileNotFound("pacman/full.png".to_string())))?,
            ];

            let stopped_tiles = vec![SpriteAtlas::get_tile(atlas, &format!("{moving_prefix}_b.png"))
                .ok_or_else(|| GameError::Texture(TextureError::AtlasTileNotFound(format!("{moving_prefix}_b.png"))))?];

            textures[direction.as_usize()] =
                Some(AnimatedTexture::new(moving_tiles, 0.08).map_err(|e| GameError::Texture(TextureError::Animated(e)))?);
            stopped_textures[direction.as_usize()] =
                Some(AnimatedTexture::new(stopped_tiles, 0.1).map_err(|e| GameError::Texture(TextureError::Animated(e)))?);
        }

        Ok(Self {
            traverser: Traverser::new(graph, start_node, Direction::Left, &can_pacman_traverse),
            texture: DirectionalAnimatedTexture::new(textures, stopped_textures),
        })
    }

    /// Handles keyboard input to change Pac-Man's direction.
    ///
    /// Maps arrow keys to directions and queues the direction change
    /// for the next valid intersection.
    pub fn handle_key(&mut self, keycode: Keycode) {
        let direction = match keycode {
            Keycode::Up => Some(Direction::Up),
            Keycode::Down => Some(Direction::Down),
            Keycode::Left => Some(Direction::Left),
            Keycode::Right => Some(Direction::Right),
            _ => None,
        };

        if let Some(direction) = direction {
            self.traverser.set_next_direction(direction);
        }
    }
}
