use glam::Vec2;

use crate::systems::components::NodeId;

use super::direction::Direction;

use bitflags::bitflags;

bitflags! {
    /// Defines who can traverse a given edge using flags for fast checking.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct TraversalFlags: u8 {
        const PACMAN = 1 << 0;
        const GHOST = 1 << 1;

        /// Convenience flag for edges that all entities can use
        const ALL = Self::PACMAN.bits() | Self::GHOST.bits();
    }
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
    pub traversal_flags: TraversalFlags,
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
    pub fn add_connected(&mut self, from: NodeId, direction: Direction, new_node: Node) -> Result<NodeId, &'static str> {
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

        let edge_a = self.add_edge(from, to, replace, distance, direction, TraversalFlags::ALL);
        let edge_b = self.add_edge(to, from, replace, distance, direction.opposite(), TraversalFlags::ALL);

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
        traversal_flags: TraversalFlags,
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
            traversal_flags,
        };

        if from >= self.adjacency_list.len() {
            return Err("From node does not exist.");
        }

        let adjacency_list = &mut self.adjacency_list[from];

        // Check if the edge already exists in this direction or to the same target
        if let Some(err) = adjacency_list.edges().find_map(|e| {
            if !replace {
                // If we're not replacing the edge, we don't want to replace an edge that already exists in this direction
                if e.direction == direction {
                    return Some(Err("Edge already exists in this direction."));
                } else if e.target == to {
                    return Some(Err("Edge already exists."));
                }
            }
            None
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
