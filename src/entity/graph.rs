use glam::Vec2;
use smallvec::SmallVec;

use super::direction::Direction;

/// A unique identifier for a node, represented by its index in the graph's storage.
pub type NodeId = usize;

/// Represents a directed edge from one node to another with a given weight (e.g., distance).
#[derive(Debug, Clone, Copy)]
pub struct Edge {
    pub target: NodeId,
    pub distance: f32,
    pub direction: Direction,
}

#[derive(Debug)]
pub struct Node {
    pub position: Vec2,
}

/// A generic, arena-based graph.
/// The graph owns all node data and connection information.
pub struct Graph {
    nodes: Vec<Node>,
    adjacency_list: Vec<SmallVec<[Edge; 4]>>,
}

impl Graph {
    /// Creates a new, empty graph.
    pub fn new() -> Self {
        Graph {
            nodes: Vec::new(),
            adjacency_list: Vec::new(),
        }
    }

    /// Adds a new node with the given data to the graph and returns its ID.
    pub fn add_node(&mut self, data: Node) -> NodeId {
        let id = self.nodes.len();
        self.nodes.push(data);
        self.adjacency_list.push(SmallVec::new());
        id
    }

    /// Adds a directed edge between two nodes.
    pub fn add_edge(
        &mut self,
        from: NodeId,
        to: NodeId,
        distance: Option<f32>,
        direction: Direction,
    ) -> Result<(), &'static str> {
        let edge = Edge {
            target: to,
            distance: match distance {
                Some(distance) => {
                    if distance <= 0.0 {
                        return Err("Edge distance must be positive.");
                    }
                    distance
                }
                None => {
                    // If no distance is provided, calculate it based on the positions of the nodes
                    let from_pos = self.nodes[from].position;
                    let to_pos = self.nodes[to].position;
                    from_pos.distance(to_pos)
                }
            },
            direction,
        };

        if from >= self.adjacency_list.len() {
            return Err("From node does not exist.");
        }

        let adjacency_list = &mut self.adjacency_list[from];

        // Check if the edge already exists in this direction or to the same target
        if let Some(err) = adjacency_list.iter().find_map(|e| {
            if e.direction == direction {
                Some(Err("Edge already exists in this direction."))
            } else if e.target == to {
                Some(Err("Edge already exists."))
            } else {
                None
            }
        }) {
            return err;
        }

        adjacency_list.push(edge);

        Ok(())
    }

    /// Retrieves an immutable reference to a node's data.
    pub fn get_node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(id)
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Finds a specific edge from a source node to a target node.
    pub fn find_edge(&self, from: NodeId, to: NodeId) -> Option<&Edge> {
        self.adjacency_list.get(from)?.iter().find(|edge| edge.target == to)
    }

    pub fn find_edge_in_direction(&self, from: NodeId, direction: Direction) -> Option<&Edge> {
        self.adjacency_list.get(from)?.iter().find(|edge| edge.direction == direction)
    }
}

// Default implementation for creating an empty graph.
impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}

// --- Traversal State and Logic ---

/// Represents the traverser's current position within the graph.
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

impl Position {
    pub fn is_at_node(&self) -> bool {
        matches!(self, Position::AtNode(_))
    }

    pub fn from_node_id(&self) -> NodeId {
        match self {
            Position::AtNode(id) => *id,
            Position::BetweenNodes { from, .. } => *from,
        }
    }

    pub fn to_node_id(&self) -> Option<NodeId> {
        match self {
            Position::AtNode(_) => None,
            Position::BetweenNodes { to, .. } => Some(*to),
        }
    }

    pub fn is_stopped(&self) -> bool {
        matches!(self, Position::AtNode(_))
    }
}

/// Manages a traversal session over a graph.
/// It holds a reference to the graph and the current position state.
pub struct Traverser {
    pub position: Position,
    pub direction: Direction,
    pub next_direction: Option<(Direction, u8)>,
}

impl Traverser {
    /// Creates a new traverser starting at the given node ID.
    pub fn new(graph: &Graph, start_node: NodeId, initial_direction: Direction) -> Self {
        let mut traverser = Traverser {
            position: Position::AtNode(start_node),
            direction: initial_direction,
            next_direction: Some((initial_direction, 1)),
        };

        // This will kickstart the traverser into motion
        traverser.advance(graph, 0.0);

        traverser
    }

    pub fn set_next_direction(&mut self, new_direction: Direction) {
        if self.direction != new_direction {
            self.next_direction = Some((new_direction, 30));
        }
    }

    pub fn advance(&mut self, graph: &Graph, distance: f32) {
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
                        // Start moving in that direction
                        self.position = Position::BetweenNodes {
                            from: node_id,
                            to: edge.target,
                            traversed: distance.max(0.0),
                        };
                        self.direction = next_direction;
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

                    // If we didn't move, try to continue in the current direction
                    if !moved {
                        if let Some(edge) = graph.find_edge_in_direction(to, self.direction) {
                            self.position = Position::BetweenNodes {
                                from: to,
                                to: edge.target,
                                traversed: overflow,
                            };
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
