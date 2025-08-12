//! Ghost entity implementation.
//!
//! This module contains the ghost character logic, including movement,
//! animation, and rendering. Ghosts move through the game graph using
//! a traverser and display directional animated textures.

use pathfinding::prelude::dijkstra;
use rand::prelude::*;
use smallvec::SmallVec;
use tracing::error;

use crate::entity::{
    collision::Collidable,
    direction::Direction,
    graph::{Edge, EdgePermissions, Graph, NodeId},
    r#trait::Entity,
    traversal::Traverser,
};
use crate::texture::animated::AnimatedTexture;
use crate::texture::directional::DirectionalAnimatedTexture;
use crate::texture::sprite::SpriteAtlas;

use crate::error::{EntityError, GameError, GameResult, TextureError};

/// Determines if a ghost can traverse a given edge.
///
/// Ghosts can move through edges that allow all entities or ghost-only edges.
fn can_ghost_traverse(edge: Edge) -> bool {
    matches!(edge.permissions, EdgePermissions::All | EdgePermissions::GhostsOnly)
}

/// The four classic ghost types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GhostType {
    Blinky,
    Pinky,
    Inky,
    Clyde,
}

impl GhostType {
    /// Returns the ghost type name for atlas lookups.
    pub fn as_str(self) -> &'static str {
        match self {
            GhostType::Blinky => "blinky",
            GhostType::Pinky => "pinky",
            GhostType::Inky => "inky",
            GhostType::Clyde => "clyde",
        }
    }

    /// Returns the base movement speed for this ghost type.
    pub fn base_speed(self) -> f32 {
        match self {
            GhostType::Blinky => 1.0,
            GhostType::Pinky => 0.95,
            GhostType::Inky => 0.9,
            GhostType::Clyde => 0.85,
        }
    }
}

/// A ghost entity that roams the game world.
///
/// Ghosts move through the game world using a graph-based navigation system
/// and display directional animated sprites. They randomly choose directions
/// at each intersection.
pub struct Ghost {
    /// Handles movement through the game graph
    pub traverser: Traverser,
    /// The type of ghost (affects appearance and speed)
    pub ghost_type: GhostType,
    /// Manages directional animated textures for different movement states
    texture: DirectionalAnimatedTexture,
    /// Current movement speed
    speed: f32,
}

impl Entity for Ghost {
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
        self.speed
    }

    fn can_traverse(&self, edge: Edge) -> bool {
        can_ghost_traverse(edge)
    }

    fn tick(&mut self, dt: f32, graph: &Graph) {
        // Choose random direction when at a node
        if self.traverser.position.is_at_node() {
            self.choose_random_direction(graph);
        }

        if let Err(e) = self.traverser.advance(graph, dt * 60.0 * self.speed, &can_ghost_traverse) {
            error!("Ghost movement error: {}", e);
        }
        self.texture.tick(dt);
    }
}

impl Ghost {
    /// Creates a new ghost instance at the specified starting node.
    ///
    /// Sets up animated textures for all four directions with moving and stopped states.
    /// The moving animation cycles through two sprite variants.
    pub fn new(graph: &Graph, start_node: NodeId, ghost_type: GhostType, atlas: &SpriteAtlas) -> GameResult<Self> {
        let mut textures = [None, None, None, None];
        let mut stopped_textures = [None, None, None, None];

        for direction in Direction::DIRECTIONS {
            let moving_prefix = match direction {
                Direction::Up => "up",
                Direction::Down => "down",
                Direction::Left => "left",
                Direction::Right => "right",
            };
            let moving_tiles = vec![
                SpriteAtlas::get_tile(atlas, &format!("ghost/{}/{}_{}.png", ghost_type.as_str(), moving_prefix, "a"))
                    .ok_or_else(|| {
                        GameError::Texture(TextureError::AtlasTileNotFound(format!(
                            "ghost/{}/{}_{}.png",
                            ghost_type.as_str(),
                            moving_prefix,
                            "a"
                        )))
                    })?,
                SpriteAtlas::get_tile(atlas, &format!("ghost/{}/{}_{}.png", ghost_type.as_str(), moving_prefix, "b"))
                    .ok_or_else(|| {
                        GameError::Texture(TextureError::AtlasTileNotFound(format!(
                            "ghost/{}/{}_{}.png",
                            ghost_type.as_str(),
                            moving_prefix,
                            "b"
                        )))
                    })?,
            ];

            let stopped_tiles =
                vec![
                    SpriteAtlas::get_tile(atlas, &format!("ghost/{}/{}_{}.png", ghost_type.as_str(), moving_prefix, "a"))
                        .ok_or_else(|| {
                            GameError::Texture(TextureError::AtlasTileNotFound(format!(
                                "ghost/{}/{}_{}.png",
                                ghost_type.as_str(),
                                moving_prefix,
                                "a"
                            )))
                        })?,
                ];

            textures[direction.as_usize()] =
                Some(AnimatedTexture::new(moving_tiles, 0.2).map_err(|e| GameError::Texture(TextureError::Animated(e)))?);
            stopped_textures[direction.as_usize()] =
                Some(AnimatedTexture::new(stopped_tiles, 0.1).map_err(|e| GameError::Texture(TextureError::Animated(e)))?);
        }

        Ok(Self {
            traverser: Traverser::new(graph, start_node, Direction::Left, &can_ghost_traverse),
            ghost_type,
            texture: DirectionalAnimatedTexture::new(textures, stopped_textures),
            speed: ghost_type.base_speed(),
        })
    }

    /// Chooses a random available direction at the current intersection.
    fn choose_random_direction(&mut self, graph: &Graph) {
        let current_node = self.traverser.position.from_node_id();
        let intersection = &graph.adjacency_list[current_node];

        // Collect all available directions
        let mut available_directions = SmallVec::<[_; 4]>::new();
        for direction in Direction::DIRECTIONS {
            if let Some(edge) = intersection.get(direction) {
                if can_ghost_traverse(edge) {
                    available_directions.push(direction);
                }
            }
        }
        // Choose a random direction (avoid reversing unless necessary)
        if !available_directions.is_empty() {
            let mut rng = SmallRng::from_os_rng();

            // Filter out the opposite direction if possible, but allow it if we have limited options
            let opposite = self.traverser.direction.opposite();
            let filtered_directions: Vec<_> = available_directions
                .iter()
                .filter(|&&dir| dir != opposite || available_directions.len() <= 2)
                .collect();

            if let Some(&random_direction) = filtered_directions.choose(&mut rng) {
                self.traverser.set_next_direction(*random_direction);
            }
        }
    }

    /// Calculates the shortest path from the ghost's current position to a target node using Dijkstra's algorithm.
    ///
    /// Returns a vector of NodeIds representing the path, or an error if pathfinding fails.
    /// The path includes the current node and the target node.
    pub fn calculate_path_to_target(&self, graph: &Graph, target: NodeId) -> GameResult<Vec<NodeId>> {
        let start_node = self.traverser.position.from_node_id();

        // Use Dijkstra's algorithm to find the shortest path
        let result = dijkstra(
            &start_node,
            |&node_id| {
                // Get all edges from the current node
                graph.adjacency_list[node_id]
                    .edges()
                    .filter(|edge| can_ghost_traverse(*edge))
                    .map(|edge| (edge.target, (edge.distance * 100.0) as u32))
                    .collect::<Vec<_>>()
            },
            |&node_id| node_id == target,
        );

        result.map(|(path, _cost)| path).ok_or_else(|| {
            GameError::Entity(EntityError::PathfindingFailed(format!(
                "No path found from node {} to target {}",
                start_node, target
            )))
        })
    }

    /// Returns the ghost's color for debug rendering.
    pub fn debug_color(&self) -> sdl2::pixels::Color {
        match self.ghost_type {
            GhostType::Blinky => sdl2::pixels::Color::RGB(255, 0, 0),    // Red
            GhostType::Pinky => sdl2::pixels::Color::RGB(255, 182, 255), // Pink
            GhostType::Inky => sdl2::pixels::Color::RGB(0, 255, 255),    // Cyan
            GhostType::Clyde => sdl2::pixels::Color::RGB(255, 182, 85),  // Orange
        }
    }
}

impl Collidable for Ghost {
    fn position(&self) -> crate::entity::traversal::Position {
        self.traverser.position
    }
}
