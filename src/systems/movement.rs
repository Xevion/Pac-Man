use crate::error::{EntityError, GameResult};
use crate::map::direction::Direction;
use crate::map::graph::Graph;
use bevy_ecs::component::Component;
use glam::Vec2;

/// Zero-based index identifying a specific node in the navigation graph.
///
/// Nodes represent discrete movement targets in the maze. The index directly corresponds to the node's position in the
/// graph's internal storage arrays.
pub type NodeId = usize;

/// A component that represents the speed and cardinal direction of an entity.
/// Speed is static, only applied when the entity has an edge to traverse.
/// Direction is dynamic, but is controlled externally.
#[derive(Component, Debug, Copy, Clone, PartialEq)]
pub struct Velocity {
    pub speed: f32,
    pub direction: Direction,
}

/// A component that represents a direction change that is only remembered for a period of time.
/// This is used to allow entities to change direction before they reach their current target node (which consumes their buffered direction).
#[derive(Component, Debug, Copy, Clone, PartialEq)]
pub enum BufferedDirection {
    None,
    Some { direction: Direction, remaining_time: f32 },
}

/// Entity position state that handles both stationary entities and moving entities.
///
/// Supports precise positioning during movement between discrete navigation nodes.
/// When moving, entities smoothly interpolate along edges while tracking exact distance remaining to the target node.
#[derive(Component, Debug, Copy, Clone, PartialEq)]
pub enum Position {
    /// Entity is stationary at a specific graph node.
    Stopped { node: NodeId },
    /// Entity is traveling between two nodes.
    Moving {
        from: NodeId,
        to: NodeId,
        /// Distance remaining to reach the target node.
        remaining_distance: f32,
    },
}

impl Position {
    /// Calculates the current pixel position in the game world.
    ///
    /// Converts the graph position to screen coordinates, accounting for
    /// the board offset and centering the sprite.
    ///
    /// # Errors
    ///
    /// Returns an `EntityError` if the node or edge is not found.
    pub fn get_pixel_position(&self, graph: &Graph) -> GameResult<Vec2> {
        let pos = match &self {
            Position::Stopped { node } => {
                // Entity is stationary at a node
                let node = graph.get_node(*node).ok_or(EntityError::NodeNotFound(*node))?;
                node.position
            }
            Position::Moving {
                from,
                to,
                remaining_distance,
            } => {
                // Entity is traveling between nodes
                let from_node = graph.get_node(*from).ok_or(EntityError::NodeNotFound(*from))?;
                let to_node = graph.get_node(*to).ok_or(EntityError::NodeNotFound(*to))?;
                let edge = graph
                    .find_edge(*from, *to)
                    .ok_or(EntityError::EdgeNotFound { from: *from, to: *to })?;

                // For zero-distance edges (tunnels), progress >= 1.0 means we're at the target
                if edge.distance == 0.0 {
                    to_node.position
                } else {
                    // Interpolate position based on progress
                    let progress = 1.0 - (*remaining_distance / edge.distance);
                    from_node.position.lerp(to_node.position, progress)
                }
            }
        };

        Ok(Vec2::new(
            pos.x + crate::constants::BOARD_PIXEL_OFFSET.x as f32,
            pos.y + crate::constants::BOARD_PIXEL_OFFSET.y as f32,
        ))
    }

    /// Advances movement progress by the specified distance with overflow handling.
    ///
    /// For moving entities, decreases the remaining distance to the target node.
    /// If the distance would overshoot the target, the entity transitions to
    /// `Stopped` state and returns the excess distance for chaining movement
    /// to the next edge in the same frame.
    ///
    /// # Arguments
    ///
    /// * `distance` - Distance to travel this frame (typically speed × delta_time)
    ///
    /// # Returns
    ///
    /// `Some(overflow)` if the target was reached with distance remaining,
    /// `None` if still moving or already stopped.
    pub fn tick(&mut self, distance: f32) -> Option<f32> {
        if distance <= 0.0 || self.is_at_node() {
            return None;
        }

        match self {
            Position::Moving {
                to, remaining_distance, ..
            } => {
                // If the remaining distance is less than or equal the distance, we'll reach the target
                if *remaining_distance <= distance {
                    let overflow: Option<f32> = if *remaining_distance != distance {
                        Some(distance - *remaining_distance)
                    } else {
                        None
                    };
                    *self = Position::Stopped { node: *to };

                    return overflow;
                }

                *remaining_distance -= distance;

                None
            }
            _ => unreachable!(),
        }
    }

    /// Returns `true` if the position is exactly at a node (not traveling).
    pub fn is_at_node(&self) -> bool {
        matches!(self, Position::Stopped { .. })
    }

    /// Returns the `NodeId` of the current node (source of travel if moving).
    pub fn current_node(&self) -> NodeId {
        match self {
            Position::Stopped { node } => *node,
            Position::Moving { from, .. } => *from,
        }
    }
}
