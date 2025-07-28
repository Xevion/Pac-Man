//! Map construction and building functionality.

use crate::constants::{MapTile, BOARD_CELL_SIZE, CELL_SIZE};
use crate::entity::direction::{Direction, DIRECTIONS};
use crate::entity::graph::{Graph, Node, NodeId};
use crate::map::parser::MapTileParser;
use crate::map::render::MapRenderer;
use crate::texture::sprite::{AtlasTile, SpriteAtlas};
use glam::{IVec2, UVec2, Vec2};
use sdl2::render::{Canvas, RenderTarget};
use std::collections::{HashMap, VecDeque};
use tracing::debug;

/// The game map, responsible for holding the tile-based layout and the navigation graph.
///
/// The map is represented as a 2D array of `MapTile`s. It also stores a navigation
/// `Graph` that entities like Pac-Man and ghosts use for movement. The graph is
/// generated from the walkable tiles of the map.
pub struct Map {
    /// The current state of the map.
    current: [[MapTile; BOARD_CELL_SIZE.y as usize]; BOARD_CELL_SIZE.x as usize],
    /// The node map for entity movement.
    pub graph: Graph,
    /// A mapping from grid positions to node IDs.
    pub grid_to_node: HashMap<IVec2, NodeId>,
}

impl Map {
    /// Creates a new `Map` instance from a raw board layout.
    ///
    /// This constructor initializes the map tiles based on the provided character layout
    /// and then generates a navigation graph from the walkable areas.
    ///
    /// # Panics
    ///
    /// This function will panic if the board layout contains unknown characters or if
    /// the house door is not defined by exactly two '=' characters.
    pub fn new(raw_board: [&str; BOARD_CELL_SIZE.y as usize]) -> Map {
        let parsed_map = MapTileParser::parse_board(raw_board).expect("Failed to parse board layout");

        let map = parsed_map.tiles;
        let house_door = parsed_map.house_door;
        let tunnel_ends = parsed_map.tunnel_ends;

        let mut graph = Graph::new();
        let mut grid_to_node = HashMap::new();

        let cell_offset = Vec2::splat(CELL_SIZE as f32 / 2.0);

        // Find a starting point for the graph generation, preferably Pac-Man's position.
        let start_pos = (0..BOARD_CELL_SIZE.y)
            .flat_map(|y| (0..BOARD_CELL_SIZE.x).map(move |x| IVec2::new(x as i32, y as i32)))
            .find(|&p| matches!(map[p.x as usize][p.y as usize], MapTile::StartingPosition(0)))
            .unwrap_or_else(|| {
                // Fallback to any valid walkable tile if Pac-Man's start is not found
                (0..BOARD_CELL_SIZE.y)
                    .flat_map(|y| (0..BOARD_CELL_SIZE.x).map(move |x| IVec2::new(x as i32, y as i32)))
                    .find(|&p| {
                        matches!(
                            map[p.x as usize][p.y as usize],
                            MapTile::Pellet
                                | MapTile::PowerPellet
                                | MapTile::Empty
                                | MapTile::Tunnel
                                | MapTile::StartingPosition(_)
                        )
                    })
                    .expect("No valid starting position found on map for graph generation")
            });

        // Add the starting position to the graph/queue
        let mut queue = VecDeque::new();
        queue.push_back(start_pos);
        let pos = Vec2::new(
            (start_pos.x * CELL_SIZE as i32) as f32,
            (start_pos.y * CELL_SIZE as i32) as f32,
        ) + cell_offset;
        let node_id = graph.add_node(Node { position: pos });
        grid_to_node.insert(start_pos, node_id);

        // Iterate over the queue, adding nodes to the graph and connecting them to their neighbors
        while let Some(source_position) = queue.pop_front() {
            for &dir in DIRECTIONS.iter() {
                let new_position = source_position + dir.as_ivec2();

                // Skip if the new position is out of bounds
                if new_position.x < 0
                    || new_position.x >= BOARD_CELL_SIZE.x as i32
                    || new_position.y < 0
                    || new_position.y >= BOARD_CELL_SIZE.y as i32
                {
                    continue;
                }

                // Skip if the new position is already in the graph
                if grid_to_node.contains_key(&new_position) {
                    continue;
                }

                // Skip if the new position is not a walkable tile
                if matches!(
                    map[new_position.x as usize][new_position.y as usize],
                    MapTile::Pellet | MapTile::PowerPellet | MapTile::Empty | MapTile::Tunnel | MapTile::StartingPosition(_)
                ) {
                    // Add the new position to the graph/queue
                    let pos = Vec2::new(
                        (new_position.x * CELL_SIZE as i32) as f32,
                        (new_position.y * CELL_SIZE as i32) as f32,
                    ) + cell_offset;
                    let new_node_id = graph.add_node(Node { position: pos });
                    grid_to_node.insert(new_position, new_node_id);
                    queue.push_back(new_position);

                    // Connect the new node to the source node
                    let source_node_id = grid_to_node
                        .get(&source_position)
                        .unwrap_or_else(|| panic!("Source node not found for {source_position}"));

                    // Connect the new node to the source node
                    graph
                        .connect(*source_node_id, new_node_id, false, None, dir)
                        .expect("Failed to add edge");
                }
            }
        }

        // While most nodes are already connected to their neighbors, some may not be, so we need to connect them
        for (grid_pos, &node_id) in &grid_to_node {
            for dir in DIRECTIONS {
                // If the node doesn't have an edge in this direction, look for a neighbor in that direction
                if graph.adjacency_list[node_id].get(dir).is_none() {
                    let neighbor = grid_pos + dir.as_ivec2();
                    // If the neighbor exists, connect the node to it
                    if let Some(&neighbor_id) = grid_to_node.get(&neighbor) {
                        graph
                            .connect(node_id, neighbor_id, false, None, dir)
                            .expect("Failed to add edge");
                    }
                }
            }
        }

        // Build house structure
        Self::build_house(&mut graph, &grid_to_node, &house_door);

        // Build tunnel connections
        Self::build_tunnels(&mut graph, &grid_to_node, &tunnel_ends);

        Map {
            current: map,
            grid_to_node,
            graph,
        }
    }

    /// Finds the starting position for a given entity ID.
    ///
    /// # Arguments
    ///
    /// * `entity_id` - The entity ID (0 for Pac-Man, 1-4 for ghosts)
    ///
    /// # Returns
    ///
    /// The starting position as a grid coordinate (`UVec2`), or `None` if not found.
    pub fn find_starting_position(&self, entity_id: u8) -> Option<UVec2> {
        for (x, col) in self.current.iter().enumerate().take(BOARD_CELL_SIZE.x as usize) {
            for (y, &cell) in col.iter().enumerate().take(BOARD_CELL_SIZE.y as usize) {
                if let MapTile::StartingPosition(id) = cell {
                    if id == entity_id {
                        return Some(UVec2::new(x as u32, y as u32));
                    }
                }
            }
        }
        None
    }

    /// Renders the map to the given canvas.
    ///
    /// This function draws the static map texture to the screen at the correct
    /// position and scale.
    pub fn render<T: RenderTarget>(&self, canvas: &mut Canvas<T>, atlas: &mut SpriteAtlas, map_texture: &mut AtlasTile) {
        MapRenderer::render_map(canvas, atlas, map_texture);
    }

    /// Renders a debug visualization of the navigation graph.
    ///
    /// This function is intended for development and debugging purposes. It draws the
    /// nodes and edges of the graph on top of the map, allowing for visual
    /// inspection of the navigation paths.
    pub fn debug_render_nodes<T: RenderTarget>(&self, canvas: &mut Canvas<T>) {
        MapRenderer::debug_render_nodes(&self.graph, canvas);
    }

    /// Builds the house structure in the graph.
    fn build_house(graph: &mut Graph, grid_to_node: &HashMap<IVec2, NodeId>, house_door: &[Option<IVec2>; 2]) {
        // Calculate the position of the house entrance node
        let (house_entrance_node_id, house_entrance_node_position) = {
            // Translate the grid positions to the actual node ids
            let left_node = grid_to_node
                .get(&(house_door[0].expect("First house door position not acquired") + Direction::Left.as_ivec2()))
                .expect("Left house door node  not found");
            let right_node = grid_to_node
                .get(&(house_door[1].expect("Second house door position not acquired") + Direction::Right.as_ivec2()))
                .expect("Right house door node  not found");

            // Calculate the position of the house node
            let (node_id, node_position) = {
                let left_pos = graph.get_node(*left_node).unwrap().position;
                let right_pos = graph.get_node(*right_node).unwrap().position;
                let house_node = graph.add_node(Node {
                    position: left_pos.lerp(right_pos, 0.5),
                });
                (house_node, left_pos.lerp(right_pos, 0.5))
            };

            // Connect the house door to the left and right nodes
            graph
                .connect(node_id, *left_node, true, None, Direction::Left)
                .expect("Failed to connect house door to left node");
            graph
                .connect(node_id, *right_node, true, None, Direction::Right)
                .expect("Failed to connect house door to right node");

            (node_id, node_position)
        };

        // A helper function to help create the various 'lines' of nodes within the house
        let create_house_line = |graph: &mut Graph, center_pos: Vec2| -> (NodeId, NodeId) {
            // Place the nodes at, above, and below the center position
            let center_node_id = graph.add_node(Node { position: center_pos });
            let top_node_id = graph.add_node(Node {
                position: center_pos + (Direction::Up.as_ivec2() * (CELL_SIZE as i32 / 2)).as_vec2(),
            });
            let bottom_node_id = graph.add_node(Node {
                position: center_pos + (Direction::Down.as_ivec2() * (CELL_SIZE as i32 / 2)).as_vec2(),
            });

            // Connect the center node to the top and bottom nodes
            graph
                .connect(center_node_id, top_node_id, false, None, Direction::Up)
                .expect("Failed to connect house line to left node");
            graph
                .connect(center_node_id, bottom_node_id, false, None, Direction::Down)
                .expect("Failed to connect house line to right node");

            (center_node_id, top_node_id)
        };

        // Calculate the position of the center line's center node
        let center_line_center_position =
            house_entrance_node_position + (Direction::Down.as_ivec2() * (3 * CELL_SIZE as i32)).as_vec2();

        // Create the center line
        let (center_center_node_id, center_top_node_id) = create_house_line(graph, center_line_center_position);

        // Connect the house entrance to the top line
        graph
            .connect(house_entrance_node_id, center_top_node_id, false, None, Direction::Down)
            .expect("Failed to connect house entrance to top line");

        // Create the left line
        let (left_center_node_id, _) = create_house_line(
            graph,
            center_line_center_position + (Direction::Left.as_ivec2() * (CELL_SIZE as i32 * 2)).as_vec2(),
        );

        // Create the right line
        let (right_center_node_id, _) = create_house_line(
            graph,
            center_line_center_position + (Direction::Right.as_ivec2() * (CELL_SIZE as i32 * 2)).as_vec2(),
        );

        debug!("Left center node id: {left_center_node_id}");

        // Connect the center line to the left and right lines
        graph
            .connect(center_center_node_id, left_center_node_id, false, None, Direction::Left)
            .expect("Failed to connect house entrance to left top line");

        graph
            .connect(center_center_node_id, right_center_node_id, false, None, Direction::Right)
            .expect("Failed to connect house entrance to right top line");

        debug!("House entrance node id: {house_entrance_node_id}");
    }

    /// Builds the tunnel connections in the graph.
    fn build_tunnels(graph: &mut Graph, grid_to_node: &HashMap<IVec2, NodeId>, tunnel_ends: &[Option<IVec2>; 2]) {
        // Create the hidden tunnel nodes
        let left_tunnel_hidden_node_id = {
            let left_tunnel_entrance_node_id = grid_to_node[&tunnel_ends[0].expect("Left tunnel end not found")];
            let left_tunnel_entrance_node = graph
                .get_node(left_tunnel_entrance_node_id)
                .expect("Left tunnel entrance node not found");

            graph
                .connect_node(
                    left_tunnel_entrance_node_id,
                    Direction::Left,
                    Node {
                        position: left_tunnel_entrance_node.position
                            + (Direction::Left.as_ivec2() * (CELL_SIZE as i32 * 2)).as_vec2(),
                    },
                )
                .expect("Failed to connect left tunnel entrance to left tunnel hidden node")
        };

        // Create the right tunnel nodes
        let right_tunnel_hidden_node_id = {
            let right_tunnel_entrance_node_id = grid_to_node[&tunnel_ends[1].expect("Right tunnel end not found")];
            let right_tunnel_entrance_node = graph
                .get_node(right_tunnel_entrance_node_id)
                .expect("Right tunnel entrance node not found");

            graph
                .connect_node(
                    right_tunnel_entrance_node_id,
                    Direction::Right,
                    Node {
                        position: right_tunnel_entrance_node.position
                            + (Direction::Right.as_ivec2() * (CELL_SIZE as i32 * 2)).as_vec2(),
                    },
                )
                .expect("Failed to connect right tunnel entrance to right tunnel hidden node")
        };

        // Connect the left tunnel hidden node to the right tunnel hidden node
        graph
            .connect(
                left_tunnel_hidden_node_id,
                right_tunnel_hidden_node_id,
                false,
                Some(0.0),
                Direction::Left,
            )
            .expect("Failed to connect left tunnel hidden node to right tunnel hidden node");
    }
}
