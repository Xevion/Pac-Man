//! The Entity-Component-System (ECS) module.
//!
//! This module contains all the ECS-related logic, including components, systems,
//! and resources.

use bevy_ecs::{bundle::Bundle, component::Component, resource::Resource};
use glam::Vec2;

use crate::{
    entity::{direction::Direction, graph::Graph, traversal},
    error::{EntityError, GameResult},
    texture::{
        animated::AnimatedTexture,
        directional::DirectionalAnimatedTexture,
        sprite::{AtlasTile, Sprite},
    },
};

/// A tag component for entities that are controlled by the player.
#[derive(Default, Component)]
pub struct PlayerControlled;

/// A component for entities that have a sprite, with a layer for ordering.
///
/// This is intended to be modified by other entities allowing animation.
#[derive(Component)]
pub struct Renderable {
    pub sprite: AtlasTile,
    pub layer: u8,
}

/// A component for entities that have a directional animated texture.
#[derive(Component)]
pub struct DirectionalAnimated {
    pub textures: [Option<AnimatedTexture>; 4],
    pub stopped_textures: [Option<AnimatedTexture>; 4],
}

/// A unique identifier for a node, represented by its index in the graph's storage.
pub type NodeId = usize;

/// Represents the current position of an entity traversing the graph.
///
/// This enum allows for precise tracking of whether an entity is exactly at a node
/// or moving along an edge between two nodes.
#[derive(Component, Debug, Copy, Clone, PartialEq)]
pub enum Position {
    /// The traverser is located exactly at a node.
    AtNode(NodeId),
    /// The traverser is on an edge between two nodes.
    BetweenNodes {
        from: NodeId,
        to: NodeId,
        /// The floating-point distance traversed along the edge from the `from` node.
        traversed: f32,
    },
}

impl Position {
    /// Calculates the current pixel position in the game world.
    ///
    /// Converts the graph position to screen coordinates, accounting for
    /// the board offset and centering the sprite.
    pub fn get_pixel_pos(&self, graph: &Graph) -> GameResult<Vec2> {
        let pos = match self {
            Position::AtNode(node_id) => {
                let node = graph.get_node(*node_id).ok_or(EntityError::NodeNotFound(*node_id))?;
                node.position
            }
            Position::BetweenNodes { from, to, traversed } => {
                let from_node = graph.get_node(*from).ok_or(EntityError::NodeNotFound(*from))?;
                let to_node = graph.get_node(*to).ok_or(EntityError::NodeNotFound(*to))?;
                let edge = graph
                    .find_edge(*from, *to)
                    .ok_or(EntityError::EdgeNotFound { from: *from, to: *to })?;
                from_node.position + (to_node.position - from_node.position) * (traversed / edge.distance)
            }
        };

        Ok(Vec2::new(
            pos.x + crate::constants::BOARD_PIXEL_OFFSET.x as f32,
            pos.y + crate::constants::BOARD_PIXEL_OFFSET.y as f32,
        ))
    }
}

impl Default for Position {
    fn default() -> Self {
        Position::AtNode(0)
    }
}

#[allow(dead_code)]
impl Position {
    /// Returns `true` if the position is exactly at a node.
    pub fn is_at_node(&self) -> bool {
        matches!(self, Position::AtNode(_))
    }

    /// Returns the `NodeId` of the current or most recently departed node.
    #[allow(clippy::wrong_self_convention)]
    pub fn from_node_id(&self) -> NodeId {
        match self {
            Position::AtNode(id) => *id,
            Position::BetweenNodes { from, .. } => *from,
        }
    }

    /// Returns the `NodeId` of the destination node, if currently on an edge.
    #[allow(clippy::wrong_self_convention)]
    pub fn to_node_id(&self) -> Option<NodeId> {
        match self {
            Position::AtNode(_) => None,
            Position::BetweenNodes { to, .. } => Some(*to),
        }
    }

    /// Returns `true` if the traverser is stopped at a node.
    pub fn is_stopped(&self) -> bool {
        matches!(self, Position::AtNode(_))
    }
}

/// A component for entities that have a velocity, with a direction and speed.
#[derive(Default, Component)]
pub struct Velocity {
    pub direction: Direction,
    pub speed: Option<f32>,
}

#[derive(Bundle)]
pub struct PlayerBundle {
    pub player: PlayerControlled,
    pub position: Position,
    pub velocity: Velocity,
    pub sprite: Renderable,
    pub directional_animated: DirectionalAnimated,
}

#[derive(Resource)]
pub struct GlobalState {
    pub exit: bool,
}

#[derive(Resource)]
pub struct DeltaTime(pub f32);

pub mod interact;
pub mod render;
