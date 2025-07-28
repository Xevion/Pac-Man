//! This module defines the game map and provides functions for interacting with it.

use crate::constants::{MapTile, BOARD_CELL_SIZE, BOARD_PIXEL_OFFSET, BOARD_PIXEL_SIZE, CELL_SIZE};
use crate::entity::direction::{Direction, DIRECTIONS};
use crate::texture::sprite::{AtlasTile, SpriteAtlas};
use glam::{IVec2, UVec2, Vec2};
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Canvas, RenderTarget};
use std::collections::{HashMap, VecDeque};
use tracing::debug;

use crate::entity::graph::{Graph, Node, NodeId};
use crate::texture::text::TextTexture;

/// Error type for map parsing operations.
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Unknown character in board: {0}")]
    UnknownCharacter(char),
    #[error("House door must have exactly 2 positions, found {0}")]
    InvalidHouseDoorCount(usize),
}

/// Represents the parsed data from a raw board layout.
#[derive(Debug)]
pub struct ParsedMap {
    /// The parsed tile layout.
    pub tiles: [[MapTile; BOARD_CELL_SIZE.y as usize]; BOARD_CELL_SIZE.x as usize],
    /// The positions of the house door tiles.
    pub house_door: [Option<IVec2>; 2],
    /// The positions of the tunnel end tiles.
    pub tunnel_ends: [Option<IVec2>; 2],
}

/// Parser for converting raw board layouts into structured map data.
pub struct MapTileParser;

impl MapTileParser {
    /// Parses a single character into a map tile.
    ///
    /// # Arguments
    ///
    /// * `c` - The character to parse
    /// * `_x` - The x coordinate of the character (unused but kept for API consistency)
    /// * `_y` - The y coordinate of the character (unused but kept for API consistency)
    ///
    /// # Returns
    ///
    /// The parsed map tile, or an error if the character is unknown.
    pub fn parse_character(c: char) -> Result<MapTile, ParseError> {
        match c {
            '#' => Ok(MapTile::Wall),
            '.' => Ok(MapTile::Pellet),
            'o' => Ok(MapTile::PowerPellet),
            ' ' => Ok(MapTile::Empty),
            'T' => Ok(MapTile::Tunnel),
            c @ '0'..='4' => Ok(MapTile::StartingPosition(c.to_digit(10).unwrap() as u8)),
            '=' => Ok(MapTile::Wall), // House door is represented as a wall tile
            _ => Err(ParseError::UnknownCharacter(c)),
        }
    }

    /// Parses a raw board layout into structured map data.
    ///
    /// # Arguments
    ///
    /// * `raw_board` - The raw board layout as an array of strings
    ///
    /// # Returns
    ///
    /// The parsed map data, or an error if parsing fails.
    ///
    /// # Errors
    ///
    /// Returns an error if the board contains unknown characters or if the house door
    /// is not properly defined by exactly two '=' characters.
    pub fn parse_board(raw_board: [&str; BOARD_CELL_SIZE.y as usize]) -> Result<ParsedMap, ParseError> {
        let mut tiles = [[MapTile::Empty; BOARD_CELL_SIZE.y as usize]; BOARD_CELL_SIZE.x as usize];
        let mut house_door = [None; 2];
        let mut tunnel_ends = [None; 2];

        for (y, line) in raw_board.iter().enumerate().take(BOARD_CELL_SIZE.y as usize) {
            for (x, character) in line.chars().enumerate().take(BOARD_CELL_SIZE.x as usize) {
                let tile = Self::parse_character(character)?;

                // Track special positions
                match tile {
                    MapTile::Tunnel => {
                        if tunnel_ends[0].is_none() {
                            tunnel_ends[0] = Some(IVec2::new(x as i32, y as i32));
                        } else {
                            tunnel_ends[1] = Some(IVec2::new(x as i32, y as i32));
                        }
                    }
                    MapTile::Wall if character == '=' => {
                        if house_door[0].is_none() {
                            house_door[0] = Some(IVec2::new(x as i32, y as i32));
                        } else {
                            house_door[1] = Some(IVec2::new(x as i32, y as i32));
                        }
                    }
                    _ => {}
                }

                tiles[x][y] = tile;
            }
        }

        // Validate house door configuration
        let house_door_count = house_door.iter().filter(|x| x.is_some()).count();
        if house_door_count != 2 {
            return Err(ParseError::InvalidHouseDoorCount(house_door_count));
        }

        Ok(ParsedMap {
            tiles,
            house_door,
            tunnel_ends,
        })
    }
}

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
                let new_position = source_position + dir.to_ivec2();

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
                        .expect(&format!("Source node not found for {source_position}"));

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
                    let neighbor = grid_pos + dir.to_ivec2();
                    // If the neighbor exists, connect the node to it
                    if let Some(&neighbor_id) = grid_to_node.get(&neighbor) {
                        graph
                            .connect(node_id, neighbor_id, false, None, dir)
                            .expect("Failed to add edge");
                    }
                }
            }
        }

        // Calculate the position of the house entrance node
        let (house_entrance_node_id, house_entrance_node_position) = {
            // Translate the grid positions to the actual node ids
            let left_node = grid_to_node
                .get(&(house_door[0].expect("First house door position not acquired") + Direction::Left.to_ivec2()))
                .expect("Left house door node  not found");
            let right_node = grid_to_node
                .get(&(house_door[1].expect("Second house door position not acquired") + Direction::Right.to_ivec2()))
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
                position: center_pos + (Direction::Up.to_ivec2() * (CELL_SIZE as i32 / 2)).as_vec2(),
            });
            let bottom_node_id = graph.add_node(Node {
                position: center_pos + (Direction::Down.to_ivec2() * (CELL_SIZE as i32 / 2)).as_vec2(),
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
            house_entrance_node_position + (Direction::Down.to_ivec2() * (3 * CELL_SIZE as i32)).as_vec2();

        // Create the center line
        let (center_center_node_id, center_top_node_id) = create_house_line(&mut graph, center_line_center_position);

        // Connect the house entrance to the top line
        graph
            .connect(house_entrance_node_id, center_top_node_id, false, None, Direction::Down)
            .expect("Failed to connect house entrance to top line");

        // Create the left line
        let (left_center_node_id, _) = create_house_line(
            &mut graph,
            center_line_center_position + (Direction::Left.to_ivec2() * (CELL_SIZE as i32 * 2)).as_vec2(),
        );

        // Create the right line
        let (right_center_node_id, _) = create_house_line(
            &mut graph,
            center_line_center_position + (Direction::Right.to_ivec2() * (CELL_SIZE as i32 * 2)).as_vec2(),
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
                            + (Direction::Left.to_ivec2() * (CELL_SIZE as i32 * 2)).as_vec2(),
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
                            + (Direction::Right.to_ivec2() * (CELL_SIZE as i32 * 2)).as_vec2(),
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
        let dest = Rect::new(
            BOARD_PIXEL_OFFSET.x as i32,
            BOARD_PIXEL_OFFSET.y as i32,
            BOARD_PIXEL_SIZE.x,
            BOARD_PIXEL_SIZE.y,
        );
        let _ = map_texture.render(canvas, atlas, dest);
    }

    /// Renders a debug visualization of the navigation graph.
    ///
    /// This function is intended for development and debugging purposes. It draws the
    /// nodes and edges of the graph on top of the map, allowing for visual
    /// inspection of the navigation paths.
    pub fn debug_render_nodes<T: RenderTarget>(&self, canvas: &mut Canvas<T>, atlas: &mut SpriteAtlas, text: &mut TextTexture) {
        for i in 0..self.graph.node_count() {
            let node = self.graph.get_node(i).unwrap();
            let pos = node.position + BOARD_PIXEL_OFFSET.as_vec2();

            // Draw connections
            canvas.set_draw_color(Color::BLUE);

            for edge in self.graph.adjacency_list[i].edges() {
                let end_pos = self.graph.get_node(edge.target).unwrap().position + BOARD_PIXEL_OFFSET.as_vec2();
                canvas
                    .draw_line((pos.x as i32, pos.y as i32), (end_pos.x as i32, end_pos.y as i32))
                    .unwrap();
            }

            // Draw node
            // let color = if pacman.position.from_node_idx() == i.into() {
            //     Color::GREEN
            // } else if let Some(to_idx) = pacman.position.to_node_idx() {
            //     if to_idx == i.into() {
            //         Color::CYAN
            //     } else {
            //         Color::RED
            //     }
            // } else {
            //     Color::RED
            // };
            canvas.set_draw_color(Color::GREEN);
            canvas
                .fill_rect(Rect::new(0, 0, 3, 3).centered_on(Point::new(pos.x as i32, pos.y as i32)))
                .unwrap();

            // Draw node index
            // text.render(canvas, atlas, &i.to_string(), pos.as_uvec2()).unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::RAW_BOARD;

    #[test]
    fn test_parse_character() {
        assert!(matches!(MapTileParser::parse_character('#').unwrap(), MapTile::Wall));
        assert!(matches!(MapTileParser::parse_character('.').unwrap(), MapTile::Pellet));
        assert!(matches!(MapTileParser::parse_character('o').unwrap(), MapTile::PowerPellet));
        assert!(matches!(MapTileParser::parse_character(' ').unwrap(), MapTile::Empty));
        assert!(matches!(MapTileParser::parse_character('T').unwrap(), MapTile::Tunnel));
        assert!(matches!(
            MapTileParser::parse_character('0').unwrap(),
            MapTile::StartingPosition(0)
        ));
        assert!(matches!(
            MapTileParser::parse_character('4').unwrap(),
            MapTile::StartingPosition(4)
        ));
        assert!(matches!(MapTileParser::parse_character('=').unwrap(), MapTile::Wall));

        // Test invalid character
        assert!(MapTileParser::parse_character('X').is_err());
    }

    #[test]
    fn test_parse_board() {
        let result = MapTileParser::parse_board(RAW_BOARD);
        assert!(result.is_ok());

        let parsed = result.unwrap();

        // Verify we have tiles
        assert_eq!(parsed.tiles.len(), BOARD_CELL_SIZE.x as usize);
        assert_eq!(parsed.tiles[0].len(), BOARD_CELL_SIZE.y as usize);

        // Verify we found house door positions
        assert!(parsed.house_door[0].is_some());
        assert!(parsed.house_door[1].is_some());

        // Verify we found tunnel ends
        assert!(parsed.tunnel_ends[0].is_some());
        assert!(parsed.tunnel_ends[1].is_some());
    }

    #[test]
    fn test_parse_board_invalid_character() {
        let mut invalid_board = RAW_BOARD.clone();
        invalid_board[0] = "###########################X";

        let result = MapTileParser::parse_board(invalid_board);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ParseError::UnknownCharacter('X')));
    }
}
