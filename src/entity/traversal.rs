use tracing::error;

use crate::ecs::{NodeId, Position};
use crate::error::GameResult;

use super::direction::Direction;
use super::graph::{Edge, Graph};

/// Manages an entity's movement through the graph.
///
/// A `Traverser` encapsulates the state of an entity's position and direction,
/// providing a way to advance along the graph's paths based on a given distance.
/// It also handles direction changes, buffering the next intended direction.
pub struct Traverser {
    /// The current position of the traverser in the graph.
    pub position: Position,
    /// The current direction of movement.
    pub direction: Direction,
    /// Buffered direction change with remaining frame count for timing.
    ///
    /// The `u8` value represents the number of frames remaining before
    /// the buffered direction expires. This allows for responsive controls
    /// by storing direction changes for a limited time.
    pub next_direction: Option<(Direction, u8)>,
}

impl Traverser {
    /// Sets the next direction for the traverser to take.
    ///
    /// The direction is buffered and will be applied at the next opportunity,
    /// typically when the traverser reaches a new node. This allows for responsive
    /// controls, as the new direction is stored for a limited time.
    pub fn set_next_direction(&mut self, new_direction: Direction) {
        if self.direction != new_direction {
            self.next_direction = Some((new_direction, 30));
        }
    }

    /// Advances the traverser along the graph by a specified distance.
    ///
    /// This method updates the traverser's position based on its current state
    /// and the distance to travel.
    ///
    /// - If at a node, it checks for a buffered direction to start moving.
    /// - If between nodes, it moves along the current edge.
    /// - If it reaches a node, it attempts to transition to a new edge based on
    ///   the buffered direction or by continuing straight.
    /// - If no valid move is possible, it stops at the node.
    ///
    /// Returns an error if the movement is invalid (e.g., trying to move in an impossible direction).
    pub fn advance<F>(&mut self, graph: &Graph, distance: f32, can_traverse: &F) -> GameResult<()>
    where
        F: Fn(Edge) -> bool,
    {
        // Decrement the remaining frames for the next direction
        if let Some((direction, remaining)) = self.next_direction {
            if remaining > 0 {
                self.next_direction = Some((direction, remaining - 1));
            } else {
                self.next_direction = None;
            }
        }

        match self.position {
            Position::AtNode(node_id) => {
                // We're not moving, but a buffered direction is available.
                if let Some((next_direction, _)) = self.next_direction {
                    if let Some(edge) = graph.find_edge_in_direction(node_id, next_direction) {
                        if can_traverse(edge) {
                            // Start moving in that direction
                            self.position = Position::BetweenNodes {
                                from: node_id,
                                to: edge.target,
                                traversed: distance.max(0.0),
                            };
                            self.direction = next_direction;
                        } else {
                            return Err(crate::error::GameError::Entity(crate::error::EntityError::InvalidMovement(
                                format!(
                                    "Cannot traverse edge from {} to {} in direction {:?}",
                                    node_id, edge.target, next_direction
                                ),
                            )));
                        }
                    } else {
                        return Err(crate::error::GameError::Entity(crate::error::EntityError::InvalidMovement(
                            format!("No edge found in direction {:?} from node {}", next_direction, node_id),
                        )));
                    }

                    self.next_direction = None; // Consume the buffered direction regardless of whether we started moving with it
                }
            }
            Position::BetweenNodes { from, to, traversed } => {
                // There is no point in any of the next logic if we don't travel at all
                if distance <= 0.0 {
                    return Ok(());
                }

                let edge = graph.find_edge(from, to).ok_or_else(|| {
                    crate::error::GameError::Entity(crate::error::EntityError::InvalidMovement(format!(
                        "Inconsistent state: Traverser is on a non-existent edge from {} to {}.",
                        from, to
                    )))
                })?;

                let new_traversed = traversed + distance;

                if new_traversed < edge.distance {
                    // Still on the same edge, just update the distance.
                    self.position = Position::BetweenNodes {
                        from,
                        to,
                        traversed: new_traversed,
                    };
                } else {
                    let overflow = new_traversed - edge.distance;
                    let mut moved = false;

                    // If we buffered a direction, try to find an edge in that direction
                    if let Some((next_dir, _)) = self.next_direction {
                        if let Some(edge) = graph.find_edge_in_direction(to, next_dir) {
                            if can_traverse(edge) {
                                self.position = Position::BetweenNodes {
                                    from: to,
                                    to: edge.target,
                                    traversed: overflow,
                                };

                                self.direction = next_dir; // Remember our new direction
                                self.next_direction = None; // Consume the buffered direction
                                moved = true;
                            }
                        }
                    }

                    // If we didn't move, try to continue in the current direction
                    if !moved {
                        if let Some(edge) = graph.find_edge_in_direction(to, self.direction) {
                            if can_traverse(edge) {
                                self.position = Position::BetweenNodes {
                                    from: to,
                                    to: edge.target,
                                    traversed: overflow,
                                };
                            } else {
                                self.position = Position::AtNode(to);
                                self.next_direction = None;
                            }
                        } else {
                            self.position = Position::AtNode(to);
                            self.next_direction = None;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
