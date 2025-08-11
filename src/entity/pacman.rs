//! Pac-Man entity implementation.
//!
//! This module contains the main player character logic, including movement,
//! animation, and rendering. Pac-Man moves through the game graph using
//! a traverser and displays directional animated textures.

use glam::{UVec2, Vec2};

use crate::constants::BOARD_PIXEL_OFFSET;
use crate::entity::direction::Direction;
use crate::entity::graph::{Edge, EdgePermissions, Graph, NodeId};
use crate::entity::traversal::{Position, Traverser};
use crate::helpers::centered_with_size;
use crate::texture::animated::AnimatedTexture;
use crate::texture::directional::DirectionalAnimatedTexture;
use crate::texture::sprite::SpriteAtlas;
use sdl2::keyboard::Keycode;
use sdl2::render::{Canvas, RenderTarget};

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

impl Pacman {
    /// Creates a new Pac-Man instance at the specified starting node.
    ///
    /// Sets up animated textures for all four directions with moving and stopped states.
    /// The moving animation cycles through open mouth, closed mouth, and full sprites.
    pub fn new(graph: &Graph, start_node: NodeId, atlas: &SpriteAtlas) -> Self {
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
                SpriteAtlas::get_tile(atlas, &format!("{moving_prefix}_a.png")).unwrap(),
                SpriteAtlas::get_tile(atlas, &format!("{moving_prefix}_b.png")).unwrap(),
                SpriteAtlas::get_tile(atlas, "pacman/full.png").unwrap(),
            ];

            let stopped_tiles = vec![SpriteAtlas::get_tile(atlas, &format!("{moving_prefix}_b.png")).unwrap()];

            textures[direction.as_usize()] = Some(AnimatedTexture::new(moving_tiles, 0.08).expect("Invalid frame duration"));
            stopped_textures[direction.as_usize()] =
                Some(AnimatedTexture::new(stopped_tiles, 0.1).expect("Invalid frame duration"));
        }

        Self {
            traverser: Traverser::new(graph, start_node, Direction::Left, &can_pacman_traverse),
            texture: DirectionalAnimatedTexture::new(textures, stopped_textures),
        }
    }

    /// Updates Pac-Man's position and animation state.
    ///
    /// Advances movement through the graph and updates texture animation.
    /// Movement speed is scaled by 60 FPS and a 1.125 multiplier.
    pub fn tick(&mut self, dt: f32, graph: &Graph) {
        self.traverser.advance(graph, dt * 60.0 * 1.125, &can_pacman_traverse);
        self.texture.tick(dt);
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

    /// Calculates the current pixel position in the game world.
    ///
    /// Interpolates between nodes when moving between them.
    fn get_pixel_pos(&self, graph: &Graph) -> Vec2 {
        match self.traverser.position {
            Position::AtNode(node_id) => graph.get_node(node_id).unwrap().position,
            Position::BetweenNodes { from, to, traversed } => {
                let from_pos = graph.get_node(from).unwrap().position;
                let to_pos = graph.get_node(to).unwrap().position;
                from_pos.lerp(to_pos, traversed / from_pos.distance(to_pos))
            }
        }
    }

    /// Renders Pac-Man to the canvas.
    ///
    /// Calculates screen position, determines if Pac-Man is stopped,
    /// and renders the appropriate directional texture.
    pub fn render<T: RenderTarget>(&self, canvas: &mut Canvas<T>, atlas: &mut SpriteAtlas, graph: &Graph) {
        let pixel_pos = self.get_pixel_pos(graph).round().as_ivec2() + BOARD_PIXEL_OFFSET.as_ivec2();
        let dest = centered_with_size(pixel_pos, UVec2::new(16, 16));
        let is_stopped = self.traverser.position.is_stopped();

        if is_stopped {
            self.texture
                .render_stopped(canvas, atlas, dest, self.traverser.direction)
                .unwrap();
        } else {
            self.texture.render(canvas, atlas, dest, self.traverser.direction).unwrap();
        }
    }
}
