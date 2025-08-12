//! Map rendering functionality.

use crate::texture::sprite::{AtlasTile, SpriteAtlas};
use crate::texture::text::TextTexture;
use glam::Vec2;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Canvas, RenderTarget};

use crate::error::{EntityError, GameError, GameResult};

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
        if let Err(e) = map_texture.render(canvas, atlas, dest) {
            tracing::error!("Failed to render map: {}", e);
        }
    }

    /// Renders a debug visualization with cursor-based highlighting.
    ///
    /// This function provides interactive debugging by highlighting the nearest node
    /// to the cursor, showing its ID, and highlighting its connections.
    pub fn debug_render_with_cursor<T: RenderTarget>(
        graph: &crate::entity::graph::Graph,
        canvas: &mut Canvas<T>,
        text_renderer: &mut TextTexture,
        atlas: &mut SpriteAtlas,
        cursor_pos: Vec2,
    ) -> GameResult<()> {
        // Find the nearest node to the cursor
        let nearest_node = Self::find_nearest_node(graph, cursor_pos);

        // Draw all connections in blue
        canvas.set_draw_color(Color::RGB(0, 0, 128)); // Dark blue for regular connections
        for i in 0..graph.node_count() {
            let node = graph.get_node(i).ok_or(GameError::Entity(EntityError::NodeNotFound(i)))?;
            let pos = node.position + crate::constants::BOARD_PIXEL_OFFSET.as_vec2();

            for edge in graph.adjacency_list[i].edges() {
                let end_pos = graph
                    .get_node(edge.target)
                    .ok_or(GameError::Entity(EntityError::NodeNotFound(edge.target)))?
                    .position
                    + crate::constants::BOARD_PIXEL_OFFSET.as_vec2();
                canvas
                    .draw_line((pos.x as i32, pos.y as i32), (end_pos.x as i32, end_pos.y as i32))
                    .map_err(|e| GameError::Sdl(e.to_string()))?;
            }
        }

        // Draw all nodes in green
        canvas.set_draw_color(Color::RGB(0, 128, 0)); // Dark green for regular nodes
        for i in 0..graph.node_count() {
            let node = graph.get_node(i).ok_or(GameError::Entity(EntityError::NodeNotFound(i)))?;
            let pos = node.position + crate::constants::BOARD_PIXEL_OFFSET.as_vec2();

            canvas
                .fill_rect(Rect::new(0, 0, 3, 3).centered_on(Point::new(pos.x as i32, pos.y as i32)))
                .map_err(|e| GameError::Sdl(e.to_string()))?;
        }

        // Highlight connections from the nearest node in bright blue
        if let Some(nearest_id) = nearest_node {
            let nearest_pos = graph
                .get_node(nearest_id)
                .ok_or(GameError::Entity(EntityError::NodeNotFound(nearest_id)))?
                .position
                + crate::constants::BOARD_PIXEL_OFFSET.as_vec2();

            canvas.set_draw_color(Color::RGB(0, 255, 255)); // Bright cyan for highlighted connections
            for edge in graph.adjacency_list[nearest_id].edges() {
                let end_pos = graph
                    .get_node(edge.target)
                    .ok_or(GameError::Entity(EntityError::NodeNotFound(edge.target)))?
                    .position
                    + crate::constants::BOARD_PIXEL_OFFSET.as_vec2();
                canvas
                    .draw_line(
                        (nearest_pos.x as i32, nearest_pos.y as i32),
                        (end_pos.x as i32, end_pos.y as i32),
                    )
                    .map_err(|e| GameError::Sdl(e.to_string()))?;
            }

            // Highlight the nearest node in bright green
            canvas.set_draw_color(Color::RGB(0, 255, 0)); // Bright green for highlighted node
            canvas
                .fill_rect(Rect::new(0, 0, 5, 5).centered_on(Point::new(nearest_pos.x as i32, nearest_pos.y as i32)))
                .map_err(|e| GameError::Sdl(e.to_string()))?;

            // Draw node ID text (small, offset to top right)
            text_renderer.set_scale(0.5); // Small text
            let id_text = format!("#{nearest_id}");
            let text_pos = glam::UVec2::new(
                (nearest_pos.x + 4.0) as u32, // Offset to the right
                (nearest_pos.y - 6.0) as u32, // Offset to the top
            );
            if let Err(e) = text_renderer.render(canvas, atlas, &id_text, text_pos) {
                tracing::error!("Failed to render node ID text: {}", e);
            }
        }

        Ok(())
    }

    /// Finds the nearest node to the given cursor position.
    pub fn find_nearest_node(graph: &crate::entity::graph::Graph, cursor_pos: Vec2) -> Option<usize> {
        let mut nearest_id = None;
        let mut nearest_distance = f32::INFINITY;

        for i in 0..graph.node_count() {
            if let Some(node) = graph.get_node(i) {
                let node_pos = node.position + crate::constants::BOARD_PIXEL_OFFSET.as_vec2();
                let distance = cursor_pos.distance(node_pos);

                if distance < nearest_distance {
                    nearest_distance = distance;
                    nearest_id = Some(i);
                }
            }
        }

        nearest_id
    }
}
