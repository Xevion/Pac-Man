use super::direction::Direction;
use super::graph::{Edge, Graph, NodeId};

/// Represents the current position of an entity traversing the graph.
///
/// This enum allows for precise tracking of whether an entity is exactly at a node
/// or moving along an edge between two nodes.
#[derive(Debug, PartialEq, Clone, Copy)]
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
    /// Creates a new traverser starting at the given node ID.
    ///
    /// The traverser will immediately attempt to start moving in the initial direction.
    pub fn new<F>(graph: &Graph, start_node: NodeId, initial_direction: Direction, can_traverse: &F) -> Self
    where
        F: Fn(Edge) -> bool,
    {
        let mut traverser = Traverser {
            position: Position::AtNode(start_node),
            direction: initial_direction,
            next_direction: Some((initial_direction, 1)),
        };

        // This will kickstart the traverser into motion
        traverser.advance(graph, 0.0, can_traverse);

        traverser
    }

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
    pub fn advance<F>(&mut self, graph: &Graph, distance: f32, can_traverse: &F)
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
                        }
                    }

                    self.next_direction = None; // Consume the buffered direction regardless of whether we started moving with it
                }
            }
            Position::BetweenNodes { from, to, traversed } => {
                // There is no point in any of the next logic if we don't travel at all
                if distance <= 0.0 {
                    return;
                }

                let edge = graph
                    .find_edge(from, to)
                    .expect("Inconsistent state: Traverser is on a non-existent edge.");

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
    }
}
