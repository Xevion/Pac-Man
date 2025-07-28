//! This module defines the game map and provides functions for interacting with it.

use crate::constants::{MapTile, BOARD_CELL_SIZE, BOARD_PIXEL_OFFSET, BOARD_PIXEL_SIZE, CELL_SIZE};
use crate::entity::direction::DIRECTIONS;
use crate::texture::sprite::{AtlasTile, SpriteAtlas};
use glam::{IVec2, UVec2, Vec2};
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Canvas, RenderTarget};
use std::collections::{HashMap, VecDeque};
use tracing::info;

use crate::entity::graph::{Graph, Node, NodeId};
use crate::texture::text::TextTexture;

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
        let mut map = [[MapTile::Empty; BOARD_CELL_SIZE.y as usize]; BOARD_CELL_SIZE.x as usize];
        let mut house_door = [None; 2];
        for (y, line) in raw_board.iter().enumerate().take(BOARD_CELL_SIZE.y as usize) {
            for (x, character) in line.chars().enumerate().take(BOARD_CELL_SIZE.x as usize) {
                let tile = match character {
                    '#' => MapTile::Wall,
                    '.' => MapTile::Pellet,
                    'o' => MapTile::PowerPellet,
                    ' ' => MapTile::Empty,
                    'T' => MapTile::Tunnel,
                    c @ '0'..='4' => MapTile::StartingPosition(c.to_digit(10).unwrap() as u8),
                    '=' => {
                        if house_door[0].is_none() {
                            house_door[0] = Some(IVec2::new(x as i32, y as i32));
                        } else {
                            house_door[1] = Some(IVec2::new(x as i32, y as i32));
                        }
                        MapTile::Wall
                    }
                    _ => panic!("Unknown character in board: {character}"),
                };
                map[x][y] = tile;
            }
        }

        if house_door.iter().filter(|x| x.is_some()).count() != 2 {
            panic!("House door must have exactly 2 positions");
        }

        let mut graph = Self::generate_graph(&map);

        let house_door_node_id = {
            let offset = Vec2::splat(CELL_SIZE as f32 / 2.0);

            let position_a = house_door[0].unwrap().as_vec2() * Vec2::splat(CELL_SIZE as f32) + offset;
            let position_b = house_door[1].unwrap().as_vec2() * Vec2::splat(CELL_SIZE as f32) + offset;
            info!("Position A: {position_a}, Position B: {position_b}");
            let position = position_a.lerp(position_b, 0.5);

            graph.add_node(Node { position })
        };
        info!("House door node id: {house_door_node_id}");

        // Connect the house door node to nearby nodes
        Self::connect_house_door(&mut graph, house_door_node_id, &map);

        Map { current: map, graph }
    }

    /// Generates a navigation graph from the given map layout.
    ///
    /// This function performs a breadth-first search (BFS) starting from Pac-Man's
    /// initial position to identify all walkable tiles and create a connected graph.
    /// Nodes are placed at the center of each walkable tile, and edges are created
    /// between adjacent walkable tiles.
    fn generate_graph(map: &[[MapTile; BOARD_CELL_SIZE.y as usize]; BOARD_CELL_SIZE.x as usize]) -> Graph {
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

        let mut queue = VecDeque::new();
        queue.push_back(start_pos);

        let pos = Vec2::new(
            (start_pos.x * CELL_SIZE as i32) as f32,
            (start_pos.y * CELL_SIZE as i32) as f32,
        ) + cell_offset;
        let node_id = graph.add_node(Node { position: pos });
        grid_to_node.insert(start_pos, node_id);

        while let Some(source_position) = queue.pop_front() {
            for &dir in DIRECTIONS.iter() {
                let new_position = source_position + dir.to_ivec2();

                if new_position.x < 0
                    || new_position.x >= BOARD_CELL_SIZE.x as i32
                    || new_position.y < 0
                    || new_position.y >= BOARD_CELL_SIZE.y as i32
                {
                    continue;
                }

                if grid_to_node.contains_key(&new_position) {
                    continue;
                }

                if matches!(
                    map[new_position.x as usize][new_position.y as usize],
                    MapTile::Pellet | MapTile::PowerPellet | MapTile::Empty | MapTile::Tunnel | MapTile::StartingPosition(_)
                ) {
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

                    graph
                        .connect(*source_node_id, new_node_id, None, dir)
                        .expect("Failed to add edge");
                }
            }
        }

        // While most nodes are already connected to their neighbors, some may not be
        for (grid_pos, &node_id) in &grid_to_node {
            for dir in DIRECTIONS {
                // If the node doesn't have an edge in this direction, look for a neighbor in that direction
                if graph.adjacency_list[node_id].get(dir).is_none() {
                    let neighbor = grid_pos + dir.to_ivec2();
                    // If the neighbor exists, connect the node to it
                    if let Some(&neighbor_id) = grid_to_node.get(&neighbor) {
                        graph.connect(node_id, neighbor_id, None, dir).expect("Failed to add edge");
                    }
                }
            }
        }

        graph
    }

    /// Connects the house door node to nearby walkable nodes in the graph.
    ///
    /// This function finds nodes within a reasonable distance of the house door
    /// and creates bidirectional connections to them.
    fn connect_house_door(
        graph: &mut Graph,
        house_door_node_id: NodeId,
        _map: &[[MapTile; BOARD_CELL_SIZE.y as usize]; BOARD_CELL_SIZE.x as usize],
    ) {
        let house_position = graph.get_node(house_door_node_id).unwrap().position;
        let connection_distance = CELL_SIZE as f32 * 1.5; // Connect to nodes within 1.5 cells

        // Find all nodes that should be connected to the house door
        for node_id in 0..graph.node_count() {
            if node_id == house_door_node_id {
                continue; // Skip the house door node itself
            }

            let node_position = graph.get_node(node_id).unwrap().position;
            let distance = house_position.distance(node_position);

            if distance <= connection_distance {
                // Determine the direction from house door to this node
                let direction = Self::direction_from_to(house_position, node_position);

                // Add bidirectional connection
                if let Err(e) = graph.add_edge(house_door_node_id, node_id, None, direction) {
                    info!("Failed to connect house door to node {}: {}", node_id, e);
                }

                // Add reverse connection
                let reverse_direction = direction.opposite();
                if let Err(e) = graph.add_edge(node_id, house_door_node_id, None, reverse_direction) {
                    info!("Failed to connect node {} to house door: {}", node_id, e);
                }
            }
        }
    }

    /// Determines the primary direction from one position to another.
    ///
    /// This is a simplified direction calculation that prioritizes the axis
    /// with the larger difference.
    fn direction_from_to(from: Vec2, to: Vec2) -> crate::entity::direction::Direction {
        let diff = to - from;
        let abs_x = diff.x.abs();
        let abs_y = diff.y.abs();

        if abs_x > abs_y {
            if diff.x > 0.0 {
                crate::entity::direction::Direction::Right
            } else {
                crate::entity::direction::Direction::Left
            }
        } else {
            if diff.y > 0.0 {
                crate::entity::direction::Direction::Down
            } else {
                crate::entity::direction::Direction::Up
            }
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
