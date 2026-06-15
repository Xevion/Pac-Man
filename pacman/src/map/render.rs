//! Map rendering functionality.

use crate::constants::CELL_SIZE;
use crate::map::layout::TILE_MAP;
use crate::texture::sprite::{AtlasTile, SpriteAtlas};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, RenderTarget};

/// Handles rendering operations for the map.
pub struct MapRenderer;

impl MapRenderer {
    /// Renders the map to the given canvas.
    ///
    /// This function draws the static map texture to the screen at the correct
    /// position and scale.
    pub fn render_map<T: RenderTarget>(canvas: &mut Canvas<T>, atlas: &mut SpriteAtlas, map_tiles: &[AtlasTile]) {
        for (y, row) in TILE_MAP.iter().enumerate() {
            for (x, &tile_index) in row.iter().enumerate() {
                let mut tile = map_tiles[tile_index];
                tile.color = Some(Color::RGB(0x20, 0x20, 0xf9));
                let dest = Rect::new(
                    (x * CELL_SIZE as usize) as i32,
                    (y * CELL_SIZE as usize) as i32,
                    CELL_SIZE,
                    CELL_SIZE,
                );

                if let Err(e) = tile.render(canvas, atlas, dest) {
                    tracing::error!("Failed to render map tile: {}", e);
                }
            }
        }
    }
}
