//! This module defines the game map and provides functions for interacting with it.

use crate::constants::{MapTile, BOARD_CELL_SIZE, BOARD_PIXEL_OFFSET, BOARD_PIXEL_SIZE, CELL_SIZE};
use crate::entity::direction::DIRECTIONS;
use crate::texture::sprite::{AtlasTile, SpriteAtlas};
use glam::{IVec2, UVec2, Vec2};
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Canvas, RenderTarget};
use smallvec::SmallVec;
use std::collections::{HashMap, VecDeque};
use tracing::info;

use crate::entity::graph::{Graph, Node};
use crate::texture::text::TextTexture;

/// The game map.
///
/// The map is represented as a 2D array of `MapTile`s. It also stores a copy of
/// the original map, which can be used to reset the map to its initial state.
pub struct Map {
    /// The current state of the map.
    current: [[MapTile; BOARD_CELL_SIZE.y as usize]; BOARD_CELL_SIZE.x as usize],
    /// The node map for entity movement.
    pub graph: Graph,
}

impl Map {
    /// Creates a new `Map` instance from a raw board layout.
    ///
    /// # Arguments
    ///
    /// * `raw_board` - A 2D array of characters representing the board layout.
    pub fn new(raw_board: [&str; BOARD_CELL_SIZE.y as usize]) -> Map {
        let mut map = [[MapTile::Empty; BOARD_CELL_SIZE.y as usize]; BOARD_CELL_SIZE.x as usize];
        let mut house_door = SmallVec::<[IVec2; 2]>::new();
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
                        house_door.push(IVec2::new(x as i32, y as i32));
                        MapTile::Wall
                    }
                    _ => panic!("Unknown character in board: {character}"),
                };
                map[x][y] = tile;
            }
        }

        if house_door.len() != 2 {
            panic!("House door must have exactly 2 positions");
        }

        let mut graph = Self::create_graph(&map);

        let house_door_node_id = {
            let offset = Vec2::splat(CELL_SIZE as f32 / 2.0);

            let position_a = house_door[0].as_vec2() * Vec2::splat(CELL_SIZE as f32) + offset;
            let position_b = house_door[1].as_vec2() * Vec2::splat(CELL_SIZE as f32) + offset;
            info!("Position A: {position_a}, Position B: {position_b}");
            let position = position_a.lerp(position_b, 0.5);

            graph.add_node(Node { position })
        };
        info!("House door node id: {house_door_node_id}");

        Map { current: map, graph }
    }

    fn create_graph(map: &[[MapTile; BOARD_CELL_SIZE.y as usize]; BOARD_CELL_SIZE.x as usize]) -> Graph {
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

        while let Some(grid_pos) = queue.pop_front() {
            for &dir in DIRECTIONS.iter() {
                let neighbor = grid_pos + dir.to_ivec2();

                if neighbor.x < 0
                    || neighbor.x >= BOARD_CELL_SIZE.x as i32
                    || neighbor.y < 0
                    || neighbor.y >= BOARD_CELL_SIZE.y as i32
                {
                    continue;
                }

                if grid_to_node.contains_key(&neighbor) {
                    continue;
                }

                if matches!(
                    map[neighbor.x as usize][neighbor.y as usize],
                    MapTile::Pellet | MapTile::PowerPellet | MapTile::Empty | MapTile::Tunnel | MapTile::StartingPosition(_)
                ) {
                    let pos =
                        Vec2::new((neighbor.x * CELL_SIZE as i32) as f32, (neighbor.y * CELL_SIZE as i32) as f32) + cell_offset;
                    let node_id = graph.add_node(Node { position: pos });
                    grid_to_node.insert(neighbor, node_id);
                    queue.push_back(neighbor);
                }
            }
        }

        for (grid_pos, &node_id) in &grid_to_node {
            for &dir in DIRECTIONS.iter() {
                let neighbor = grid_pos + dir.to_ivec2();

                if let Some(&neighbor_id) = grid_to_node.get(&neighbor) {
                    graph.add_edge(node_id, neighbor_id, None, dir).expect("Failed to add edge");
                }
            }
        }
        graph
    }

    /// Finds the starting position for a given entity ID.
    ///
    /// # Arguments
    ///
    /// * `entity_id` - The entity ID (0 for Pac-Man, 1-4 for ghosts)
    ///
    /// # Returns
    ///
    /// The starting position as UVec2, or None if not found
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

    /// Renders the map to the given canvas using the provided map texture.
    pub fn render<T: RenderTarget>(&self, canvas: &mut Canvas<T>, atlas: &mut SpriteAtlas, map_texture: &mut AtlasTile) {
        let dest = Rect::new(
            BOARD_PIXEL_OFFSET.x as i32,
            BOARD_PIXEL_OFFSET.y as i32,
            BOARD_PIXEL_SIZE.x,
            BOARD_PIXEL_SIZE.y,
        );
        let _ = map_texture.render(canvas, atlas, dest);
    }

    pub fn debug_render_nodes<T: RenderTarget>(&self, canvas: &mut Canvas<T>, atlas: &mut SpriteAtlas, text: &mut TextTexture) {
        for i in 0..self.graph.node_count() {
            let node = self.graph.get_node(i).unwrap();
            let pos = node.position + BOARD_PIXEL_OFFSET.as_vec2();

            // Draw connections
            // TODO: fix this
            // canvas.set_draw_color(Color::BLUE);

            // for neighbor in node.neighbors() {
            //     let end_pos = neighbor.get(&self.node_map).position + BOARD_PIXEL_OFFSET.as_vec2();
            //     canvas
            //         .draw_line((pos.x as i32, pos.y as i32), (end_pos.x as i32, end_pos.y as i32))
            //         .unwrap();
            // }

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
