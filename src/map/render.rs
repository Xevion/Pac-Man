//! Map rendering functionality.

use crate::texture::sprite::{AtlasTile, SpriteAtlas};
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Canvas, RenderTarget};

/// Handles rendering operations for the map.
pub struct MapRenderer;

impl MapRenderer {
    /// Renders the map to the given canvas.
    ///
    /// This function draws the static map texture to the screen at the correct
    /// position and scale.
    pub fn render_map<T: RenderTarget>(canvas: &mut Canvas<T>, atlas: &mut SpriteAtlas, map_texture: &mut AtlasTile) {
        let dest = Rect::new(
            crate::constants::BOARD_PIXEL_OFFSET.x as i32,
            crate::constants::BOARD_PIXEL_OFFSET.y as i32,
            crate::constants::BOARD_PIXEL_SIZE.x,
            crate::constants::BOARD_PIXEL_SIZE.y,
        );
        let _ = map_texture.render(canvas, atlas, dest);
    }

    /// Renders a debug visualization of the navigation graph.
    ///
    /// This function is intended for development and debugging purposes. It draws the
    /// nodes and edges of the graph on top of the map, allowing for visual
    /// inspection of the navigation paths.
    pub fn debug_render_nodes<T: RenderTarget>(graph: &crate::entity::graph::Graph, canvas: &mut Canvas<T>) {
        for i in 0..graph.node_count() {
            let node = graph.get_node(i).unwrap();
            let pos = node.position + crate::constants::BOARD_PIXEL_OFFSET.as_vec2();

            // Draw connections
            canvas.set_draw_color(Color::BLUE);

            for edge in graph.adjacency_list[i].edges() {
                let end_pos = graph.get_node(edge.target).unwrap().position + crate::constants::BOARD_PIXEL_OFFSET.as_vec2();
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
