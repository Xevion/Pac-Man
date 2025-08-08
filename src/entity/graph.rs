use glam::Vec2;

use super::direction::Direction;

/// A unique identifier for a node, represented by its index in the graph's storage.
pub type NodeId = usize;

/// Defines who can traverse a given edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EdgePermissions {
    /// Anyone can use this edge.
    #[default]
    All,
    /// Only ghosts can use this edge.
    GhostsOnly,
}

/// Represents a directed edge from one node to another with a given weight (e.g., distance).
#[derive(Debug, Clone, Copy)]
pub struct Edge {
    /// The destination node of this edge.
    pub target: NodeId,
    /// The length of the edge.
    pub distance: f32,
    /// The cardinal direction of this edge.
    pub direction: Direction,
    /// Defines who is allowed to traverse this edge.
    pub permissions: EdgePermissions,
}

/// Represents a node in the graph, defined by its position.
#[derive(Debug)]
pub struct Node {
    /// The 2D coordinates of the node.
    pub position: Vec2,
}

/// Represents the four possible directions from a node in the graph.
///
/// Each field contains an optional edge leading in that direction.
/// This structure is used to represent the adjacency list for each node,
/// providing O(1) access to edges in any cardinal direction.
#[derive(Debug, Default)]
pub struct Intersection {
    /// Edge leading upward from this node, if it exists.
    pub up: Option<Edge>,
    /// Edge leading downward from this node, if it exists.
    pub down: Option<Edge>,
    /// Edge leading leftward from this node, if it exists.
    pub left: Option<Edge>,
    /// Edge leading rightward from this node, if it exists.
    pub right: Option<Edge>,
}

impl Intersection {
    /// Returns an iterator over all edges from this intersection.
    ///
    /// This iterator yields only the edges that exist (non-None values).
    pub fn edges(&self) -> impl Iterator<Item = Edge> {
        [self.up, self.down, self.left, self.right].into_iter().flatten()
    }

    /// Retrieves the edge in the specified direction, if it exists.
    pub fn get(&self, direction: Direction) -> Option<Edge> {
        match direction {
            Direction::Up => self.up,
            Direction::Down => self.down,
            Direction::Left => self.left,
            Direction::Right => self.right,
        }
    }

    /// Sets the edge in the specified direction.
    ///
    /// This will overwrite any existing edge in that direction.
    pub fn set(&mut self, direction: Direction, edge: Edge) {
        match direction {
            Direction::Up => self.up = Some(edge),
            Direction::Down => self.down = Some(edge),
            Direction::Left => self.left = Some(edge),
            Direction::Right => self.right = Some(edge),
        }
    }
}

/// A directed graph structure using an adjacency list representation.
///
/// Nodes are stored in a vector, and their indices serve as their `NodeId`.
/// This design provides fast, O(1) lookups for node data. Edges are stored
/// in an adjacency list, where each node has a list of outgoing edges.
pub struct Graph {
    nodes: Vec<Node>,
    pub adjacency_list: Vec<Intersection>,
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
        self.adjacency_list.push(Intersection::default());
        id
    }

    /// Connects a new node to the graph and adds an edge between the existing node and the new node.
    pub fn connect_node(&mut self, from: NodeId, direction: Direction, new_node: Node) -> Result<NodeId, &'static str> {
        let to = self.add_node(new_node);
        self.connect(from, to, false, None, direction)?;
        Ok(to)
    }

    /// Connects two existing nodes with an edge.
    pub fn connect(
        &mut self,
        from: NodeId,
        to: NodeId,
        replace: bool,
        distance: Option<f32>,
        direction: Direction,
    ) -> Result<(), &'static str> {
        if from >= self.adjacency_list.len() {
            return Err("From node does not exist.");
        }
        if to >= self.adjacency_list.len() {
            return Err("To node does not exist.");
        }

        let edge_a = self.add_edge(from, to, replace, distance, direction, EdgePermissions::default());
        let edge_b = self.add_edge(to, from, replace, distance, direction.opposite(), EdgePermissions::default());

        if edge_a.is_err() && edge_b.is_err() {
            return Err("Failed to connect nodes in both directions.");
        }

        Ok(())
    }

    /// Adds a directed edge between two nodes.
    ///
    /// If `distance` is `None`, it will be calculated automatically based on the
    /// Euclidean distance between the two nodes.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The `from` node does not exist
    /// - An edge already exists in the specified direction
    /// - An edge already exists to the target node
    /// - The provided distance is not positive
    pub fn add_edge(
        &mut self,
        from: NodeId,
        to: NodeId,
        replace: bool,
        distance: Option<f32>,
        direction: Direction,
        permissions: EdgePermissions,
    ) -> Result<(), &'static str> {
        let edge = Edge {
            target: to,
            distance: match distance {
                Some(distance) => {
                    if distance < 0.0 {
                        return Err("Edge distance must be on-negative.");
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
            permissions,
        };

        if from >= self.adjacency_list.len() {
            return Err("From node does not exist.");
        }

        let adjacency_list = &mut self.adjacency_list[from];

        // Check if the edge already exists in this direction or to the same target
        if let Some(err) = adjacency_list.edges().find_map(|e| {
            // If we're not replacing the edge, we don't want to replace an edge that already exists in this direction
            if !replace && e.direction == direction {
                Some(Err("Edge already exists in this direction."))
            } else if e.target == to {
                Some(Err("Edge already exists."))
            } else {
                None
            }
        }) {
            return err;
        }

        adjacency_list.set(direction, edge);

        Ok(())
    }

    /// Retrieves an immutable reference to a node's data.
    pub fn get_node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(id)
    }

    /// Returns the total number of nodes in the graph.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Finds a specific edge from a source node to a target node.
    pub fn find_edge(&self, from: NodeId, to: NodeId) -> Option<Edge> {
        self.adjacency_list.get(from)?.edges().find(|edge| edge.target == to)
    }

    /// Finds an edge originating from a given node that follows a specific direction.
    pub fn find_edge_in_direction(&self, from: NodeId, direction: Direction) -> Option<Edge> {
        self.adjacency_list.get(from)?.get(direction)
    }
}

// Default implementation for creating an empty graph.
impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}

// --- Traversal State and Logic ---

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
