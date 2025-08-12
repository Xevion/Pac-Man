//! Map construction and building functionality.

use crate::constants::{MapTile, BOARD_CELL_SIZE, CELL_SIZE};
use crate::entity::direction::Direction;
use crate::entity::graph::{EdgePermissions, Graph, Node, NodeId};
use crate::map::parser::MapTileParser;
use crate::map::render::MapRenderer;
use crate::texture::sprite::{AtlasTile, SpriteAtlas};
use glam::{IVec2, UVec2, Vec2};
use sdl2::render::{Canvas, RenderTarget};
use std::collections::{HashMap, VecDeque};
use tracing::debug;

use crate::error::{GameResult, MapError};

/// The starting positions of the entities in the game.
#[allow(dead_code)]
pub struct NodePositions {
    pub pacman: NodeId,
    pub blinky: NodeId,
    pub pinky: NodeId,
    pub inky: NodeId,
    pub clyde: NodeId,
}

/// The main map structure containing the game board and navigation graph.
pub struct Map {
    /// The current state of the map.
    #[allow(dead_code)]
    current: [[MapTile; BOARD_CELL_SIZE.y as usize]; BOARD_CELL_SIZE.x as usize],
    /// The node map for entity movement.
    pub graph: Graph,
    /// A mapping from grid positions to node IDs.
    pub grid_to_node: HashMap<IVec2, NodeId>,
    /// A mapping of the starting positions of the entities.
    #[allow(dead_code)]
    pub start_positions: NodePositions,
    /// Pac-Man's starting position.
    pacman_start: Option<IVec2>,
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
    pub fn new(raw_board: [&str; BOARD_CELL_SIZE.y as usize]) -> GameResult<Map> {
        let parsed_map = MapTileParser::parse_board(raw_board)?;

        let map = parsed_map.tiles;
        let house_door = parsed_map.house_door;
        let tunnel_ends = parsed_map.tunnel_ends;
        let pacman_start = parsed_map.pacman_start;

        let mut graph = Graph::new();
        let mut grid_to_node = HashMap::new();

        let cell_offset = Vec2::splat(CELL_SIZE as f32 / 2.0);

        // Find a starting point for the graph generation, preferably Pac-Man's position.
        let start_pos =
            pacman_start.ok_or_else(|| MapError::InvalidConfig("Pac-Man's starting position not found".to_string()))?;

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
            for dir in Direction::DIRECTIONS {
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
                    MapTile::Pellet | MapTile::PowerPellet | MapTile::Empty | MapTile::Tunnel
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
                        .map_err(|e| MapError::InvalidConfig(format!("Failed to add edge: {e}")))?;
                }
            }
        }

        // While most nodes are already connected to their neighbors, some may not be, so we need to connect them
        for (grid_pos, &node_id) in &grid_to_node {
            for dir in Direction::DIRECTIONS {
                // If the node doesn't have an edge in this direction, look for a neighbor in that direction
                if graph.adjacency_list[node_id].get(dir).is_none() {
                    let neighbor = grid_pos + dir.as_ivec2();
                    // If the neighbor exists, connect the node to it
                    if let Some(&neighbor_id) = grid_to_node.get(&neighbor) {
                        graph
                            .connect(node_id, neighbor_id, false, None, dir)
                            .map_err(|e| MapError::InvalidConfig(format!("Failed to add edge: {e}")))?;
                    }
                }
            }
        }

        // Build house structure
        let (house_entrance_node_id, left_center_node_id, center_center_node_id, right_center_node_id) =
            Self::build_house(&mut graph, &grid_to_node, &house_door)?;

        let start_positions = NodePositions {
            pacman: grid_to_node[&start_pos],
            blinky: house_entrance_node_id,
            pinky: left_center_node_id,
            inky: right_center_node_id,
            clyde: center_center_node_id,
        };

        // Build tunnel connections
        Self::build_tunnels(&mut graph, &grid_to_node, &tunnel_ends)?;

        Ok(Map {
            current: map,
            graph,
            grid_to_node,
            start_positions,
            pacman_start,
        })
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
        // For now, only Pac-Man (entity_id 0) is supported
        if entity_id == 0 {
            return self.pacman_start.map(|pos| UVec2::new(pos.x as u32, pos.y as u32));
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

    /// Renders a debug visualization with cursor-based highlighting.
    ///
    /// This function provides interactive debugging by highlighting the nearest node
    /// to the cursor, showing its ID, and highlighting its connections.
    pub fn debug_render_with_cursor<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        text_renderer: &mut crate::texture::text::TextTexture,
        atlas: &mut SpriteAtlas,
        cursor_pos: glam::Vec2,
    ) -> GameResult<()> {
        MapRenderer::debug_render_with_cursor(&self.graph, canvas, text_renderer, atlas, cursor_pos)
    }

    /// Builds the house structure in the graph.
    fn build_house(
        graph: &mut Graph,
        grid_to_node: &HashMap<IVec2, NodeId>,
        house_door: &[Option<IVec2>; 2],
    ) -> GameResult<(usize, usize, usize, usize)> {
        // Calculate the position of the house entrance node
        let (house_entrance_node_id, house_entrance_node_position) = {
            // Translate the grid positions to the actual node ids
            let left_node = grid_to_node
                .get(
                    &(house_door[0]
                        .ok_or_else(|| MapError::InvalidConfig("First house door position not acquired".to_string()))?
                        + Direction::Left.as_ivec2()),
                )
                .ok_or_else(|| MapError::InvalidConfig("Left house door node not found".to_string()))?;
            let right_node = grid_to_node
                .get(
                    &(house_door[1]
                        .ok_or_else(|| MapError::InvalidConfig("Second house door position not acquired".to_string()))?
                        + Direction::Right.as_ivec2()),
                )
                .ok_or_else(|| MapError::InvalidConfig("Right house door node not found".to_string()))?;

            // Calculate the position of the house node
            let (node_id, node_position) = {
                let left_pos = graph.get_node(*left_node).ok_or(MapError::NodeNotFound(*left_node))?.position;
                let right_pos = graph
                    .get_node(*right_node)
                    .ok_or(MapError::NodeNotFound(*right_node))?
                    .position;
                let house_node = graph.add_node(Node {
                    position: left_pos.lerp(right_pos, 0.5),
                });
                (house_node, left_pos.lerp(right_pos, 0.5))
            };

            // Connect the house door to the left and right nodes
            graph
                .connect(node_id, *left_node, true, None, Direction::Left)
                .map_err(|e| MapError::InvalidConfig(format!("Failed to connect house door to left node: {e}")))?;
            graph
                .connect(node_id, *right_node, true, None, Direction::Right)
                .map_err(|e| MapError::InvalidConfig(format!("Failed to connect house door to right node: {e}")))?;

            (node_id, node_position)
        };

        // A helper function to help create the various 'lines' of nodes within the house
        let create_house_line = |graph: &mut Graph, center_pos: Vec2| -> GameResult<(NodeId, NodeId)> {
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
                .map_err(|e| MapError::InvalidConfig(format!("Failed to connect house line to top node: {e}")))?;
            graph
                .connect(center_node_id, bottom_node_id, false, None, Direction::Down)
                .map_err(|e| MapError::InvalidConfig(format!("Failed to connect house line to bottom node: {e}")))?;

            Ok((center_node_id, top_node_id))
        };

        // Calculate the position of the center line's center node
        let center_line_center_position =
            house_entrance_node_position + (Direction::Down.as_ivec2() * (3 * CELL_SIZE as i32)).as_vec2();

        // Create the center line
        let (center_center_node_id, center_top_node_id) = create_house_line(graph, center_line_center_position)?;

        // Create a ghost-only, two-way connection for the house door.
        // This prevents Pac-Man from entering or exiting through the door.
        graph
            .add_edge(
                house_entrance_node_id,
                center_top_node_id,
                false,
                None,
                Direction::Down,
                EdgePermissions::GhostsOnly,
            )
            .map_err(|e| MapError::InvalidConfig(format!("Failed to create ghost-only entrance to house: {e}")))?;

        graph
            .add_edge(
                center_top_node_id,
                house_entrance_node_id,
                false,
                None,
                Direction::Up,
                EdgePermissions::GhostsOnly,
            )
            .map_err(|e| MapError::InvalidConfig(format!("Failed to create ghost-only exit from house: {e}")))?;

        // Create the left line
        let (left_center_node_id, _) = create_house_line(
            graph,
            center_line_center_position + (Direction::Left.as_ivec2() * (CELL_SIZE as i32 * 2)).as_vec2(),
        )?;

        // Create the right line
        let (right_center_node_id, _) = create_house_line(
            graph,
            center_line_center_position + (Direction::Right.as_ivec2() * (CELL_SIZE as i32 * 2)).as_vec2(),
        )?;

        debug!("Left center node id: {left_center_node_id}");

        // Connect the center line to the left and right lines
        graph
            .connect(center_center_node_id, left_center_node_id, false, None, Direction::Left)
            .map_err(|e| MapError::InvalidConfig(format!("Failed to connect house entrance to left top line: {e}")))?;

        graph
            .connect(center_center_node_id, right_center_node_id, false, None, Direction::Right)
            .map_err(|e| MapError::InvalidConfig(format!("Failed to connect house entrance to right top line: {e}")))?;

        debug!("House entrance node id: {house_entrance_node_id}");

        Ok((
            house_entrance_node_id,
            left_center_node_id,
            center_center_node_id,
            right_center_node_id,
        ))
    }

    /// Builds the tunnel connections in the graph.
    fn build_tunnels(
        graph: &mut Graph,
        grid_to_node: &HashMap<IVec2, NodeId>,
        tunnel_ends: &[Option<IVec2>; 2],
    ) -> GameResult<()> {
        // Create the hidden tunnel nodes
        let left_tunnel_hidden_node_id = {
            let left_tunnel_entrance_node_id =
                grid_to_node[&tunnel_ends[0].ok_or_else(|| MapError::InvalidConfig("Left tunnel end not found".to_string()))?];
            let left_tunnel_entrance_node = graph
                .get_node(left_tunnel_entrance_node_id)
                .ok_or_else(|| MapError::InvalidConfig("Left tunnel entrance node not found".to_string()))?;

            graph
                .add_connected(
                    left_tunnel_entrance_node_id,
                    Direction::Left,
                    Node {
                        position: left_tunnel_entrance_node.position
                            + (Direction::Left.as_ivec2() * (CELL_SIZE as i32 * 2)).as_vec2(),
                    },
                )
                .map_err(|e| {
                    MapError::InvalidConfig(format!(
                        "Failed to connect left tunnel entrance to left tunnel hidden node: {}",
                        e
                    ))
                })?
        };

        // Create the right tunnel nodes
        let right_tunnel_hidden_node_id = {
            let right_tunnel_entrance_node_id =
                grid_to_node[&tunnel_ends[1].ok_or_else(|| MapError::InvalidConfig("Right tunnel end not found".to_string()))?];
            let right_tunnel_entrance_node = graph
                .get_node(right_tunnel_entrance_node_id)
                .ok_or_else(|| MapError::InvalidConfig("Right tunnel entrance node not found".to_string()))?;

            graph
                .add_connected(
                    right_tunnel_entrance_node_id,
                    Direction::Right,
                    Node {
                        position: right_tunnel_entrance_node.position
                            + (Direction::Right.as_ivec2() * (CELL_SIZE as i32 * 2)).as_vec2(),
                    },
                )
                .map_err(|e| {
                    MapError::InvalidConfig(format!(
                        "Failed to connect right tunnel entrance to right tunnel hidden node: {}",
                        e
                    ))
                })?
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
            .map_err(|e| {
                MapError::InvalidConfig(format!(
                    "Failed to connect left tunnel hidden node to right tunnel hidden node: {}",
                    e
                ))
            })?;

        Ok(())
    }
}
