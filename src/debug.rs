//! Debug rendering utilities for Pac-Man.
use crate::{
    constants::{MapTile, BOARD_HEIGHT, BOARD_WIDTH},
    ghosts::blinky::Blinky,
    map::Map,
};
use glam::{IVec2, UVec2};
use sdl2::{pixels::Color, render::Canvas, video::Window};

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum DebugMode {
    None,
    Grid,
    Pathfinding,
    ValidPositions,
}

pub struct DebugRenderer;

impl DebugRenderer {
    pub fn draw_cell(canvas: &mut Canvas<Window>, _map: &Map, cell: UVec2, color: Color) {
        let position = Map::cell_to_pixel(cell);
        canvas.set_draw_color(color);
        canvas
            .draw_rect(sdl2::rect::Rect::new(position.x, position.y, 24, 24))
            .expect("Could not draw rectangle");
    }

    pub fn draw_debug_grid(canvas: &mut Canvas<Window>, map: &Map, pacman_cell: UVec2) {
        for x in 0..BOARD_WIDTH {
            for y in 0..BOARD_HEIGHT {
                let tile = map.get_tile(IVec2::new(x as i32, y as i32)).unwrap_or(MapTile::Empty);
                let cell = UVec2::new(x, y);
                let mut color = None;
                if cell == pacman_cell {
                    Self::draw_cell(canvas, map, cell, Color::CYAN);
                } else {
                    color = match tile {
                        MapTile::Empty => None,
                        MapTile::Wall => Some(Color::BLUE),
                        MapTile::Pellet => Some(Color::RED),
                        MapTile::PowerPellet => Some(Color::MAGENTA),
                        MapTile::StartingPosition(_) => Some(Color::GREEN),
                        MapTile::Tunnel => Some(Color::CYAN),
                    };
                }
                if let Some(color) = color {
                    Self::draw_cell(canvas, map, cell, color);
                }
            }
        }
    }

    pub fn draw_next_cell(canvas: &mut Canvas<Window>, map: &Map, next_cell: UVec2) {
        Self::draw_cell(canvas, map, next_cell, Color::YELLOW);
    }

    pub fn draw_valid_positions(canvas: &mut Canvas<Window>, map: &mut Map) {
        let valid_positions_vec = map.get_valid_playable_positions().clone();
        for &pos in &valid_positions_vec {
            Self::draw_cell(canvas, map, pos, Color::RGB(255, 140, 0));
        }
    }

    pub fn draw_pathfinding(canvas: &mut Canvas<Window>, blinky: &Blinky, map: &Map) {
        let target = blinky.get_target_tile();
        if let Some((path, _)) = blinky.get_path_to_target(target.as_uvec2()) {
            for pos in &path {
                Self::draw_cell(canvas, map, *pos, Color::YELLOW);
            }
        }
    }
}
