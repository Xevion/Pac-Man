//! Debug rendering utilities for Pac-Man.
use crate::{
    constants::{MapTile, BOARD_HEIGHT, BOARD_WIDTH},
    ghosts::blinky::Blinky,
    map::Map,
};
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
    pub fn draw_cell(canvas: &mut Canvas<Window>, _map: &Map, cell: (u32, u32), color: Color) {
        let position = Map::cell_to_pixel(cell);
        canvas.set_draw_color(color);
        canvas
            .draw_rect(sdl2::rect::Rect::new(position.0, position.1, 24, 24))
            .expect("Could not draw rectangle");
    }

    pub fn draw_debug_grid(canvas: &mut Canvas<Window>, map: &Map, pacman_cell: (u32, u32)) {
        for x in 0..BOARD_WIDTH {
            for y in 0..BOARD_HEIGHT {
                let tile = map.get_tile((x as i32, y as i32)).unwrap_or(MapTile::Empty);
                let mut color = None;
                if (x, y) == pacman_cell {
                    Self::draw_cell(canvas, map, (x, y), Color::CYAN);
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
                    Self::draw_cell(canvas, map, (x, y), color);
                }
            }
        }
    }

    pub fn draw_next_cell(canvas: &mut Canvas<Window>, map: &Map, next_cell: (u32, u32)) {
        Self::draw_cell(canvas, map, next_cell, Color::YELLOW);
    }

    pub fn draw_valid_positions(canvas: &mut Canvas<Window>, map: &mut Map) {
        let valid_positions_vec = map.get_valid_playable_positions().clone();
        for &pos in &valid_positions_vec {
            Self::draw_cell(canvas, map, (pos.x, pos.y), Color::RGB(255, 140, 0));
            // ORANGE
        }
    }

    pub fn draw_pathfinding(canvas: &mut Canvas<Window>, blinky: &Blinky, map: &Map) {
        if let Some((path, _)) = blinky.get_path_to_target({
            let (tx, ty) = blinky.get_target_tile();
            (tx as u32, ty as u32)
        }) {
            for &(x, y) in &path {
                Self::draw_cell(canvas, map, (x, y), Color::YELLOW);
            }
        }
    }
}
