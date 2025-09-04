//! Debug rendering system
use std::cmp::Ordering;

use crate::constants::BOARD_PIXEL_OFFSET;
use crate::map::builder::Map;
use crate::systems::{Collider, CursorPosition, NodeId, Position, SystemTimings};
use crate::texture::ttf::{TtfAtlas, TtfRenderer};
use bevy_ecs::resource::Resource;
use bevy_ecs::system::{NonSendMut, Query, Res};
use glam::{IVec2, UVec2, Vec2};
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;
use smallvec::SmallVec;
use tracing::warn;

#[derive(Resource, Default, Debug, Copy, Clone)]
pub struct DebugState {
    pub enabled: bool,
}

fn f32_to_u8(value: f32) -> u8 {
    (value * 255.0) as u8
}

/// Resource to hold the debug texture for persistent rendering
pub struct DebugTextureResource(pub Texture);

/// Resource to hold the TTF text atlas
pub struct TtfAtlasResource(pub TtfAtlas);

/// Transforms a position from logical canvas coordinates to output canvas coordinates (with board offset)
fn transform_position_with_offset(pos: Vec2, scale: f32) -> IVec2 {
    ((pos + BOARD_PIXEL_OFFSET.as_vec2()) * scale).as_ivec2()
}

/// Renders timing information in the top-left corner of the screen using the debug text atlas
fn render_timing_display(
    canvas: &mut Canvas<Window>,
    timings: &SystemTimings,
    text_renderer: &TtfRenderer,
    atlas: &mut TtfAtlas,
) {
    // Format timing information using the formatting module
    let lines = timings.format_timing_display();
    let line_height = text_renderer.text_height(atlas) as i32 + 2; // Add 2px line spacing
    let padding = 10;

    // Calculate background dimensions
    let max_width = lines
        .iter()
        .filter(|l| !l.is_empty()) // Don't consider empty lines for width
        .map(|line| text_renderer.text_width(atlas, line))
        .max()
        .unwrap_or(0);

    // Only draw background if there is text to display
    let total_height = (lines.len() as u32) * line_height as u32;
    if max_width > 0 && total_height > 0 {
        let bg_padding = 5;

        // Draw background
        let bg_rect = Rect::new(
            padding - bg_padding,
            padding - bg_padding,
            max_width + (bg_padding * 2) as u32,
            total_height + bg_padding as u32,
        );
        canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
        canvas.set_draw_color(Color::RGBA(40, 40, 40, 180));
        canvas.fill_rect(bg_rect).unwrap();
    }

    for (i, line) in lines.iter().enumerate() {
        if line.is_empty() {
            continue;
        }

        // Position each line below the previous one
        let y_pos = padding + (i as i32 * line_height);
        let position = Vec2::new(padding as f32, y_pos as f32);

        // Render the line using the debug text renderer
        text_renderer
            .render_text(canvas, atlas, line, position, Color::RGBA(255, 255, 255, 200))
            .unwrap();
    }
}

#[allow(clippy::too_many_arguments)]
pub fn debug_render_system(
    mut canvas: NonSendMut<&mut Canvas<Window>>,
    mut debug_texture: NonSendMut<DebugTextureResource>,
    mut ttf_atlas: NonSendMut<TtfAtlasResource>,
    debug_state: Res<DebugState>,
    timings: Res<SystemTimings>,
    map: Res<Map>,
    colliders: Query<(&Collider, &Position)>,
    cursor: Res<CursorPosition>,
) {
    if !debug_state.enabled {
        return;
    }
    let scale =
        (UVec2::from(canvas.output_size().unwrap()).as_vec2() / UVec2::from(canvas.logical_size()).as_vec2()).min_element();

    // Create debug text renderer
    let text_renderer = TtfRenderer::new(1.0);

    let cursor_world_pos = match *cursor {
        CursorPosition::None => None,
        CursorPosition::Some { position, .. } => Some(position - BOARD_PIXEL_OFFSET.as_vec2()),
    };

    // Draw debug info on the high-resolution debug texture
    canvas
        .with_texture_canvas(&mut debug_texture.0, |debug_canvas| {
            // Clear the debug canvas
            debug_canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
            debug_canvas.clear();

            // Find the closest node to the cursor
            let closest_node = if let Some(cursor_world_pos) = cursor_world_pos {
                map.graph
                    .nodes()
                    .map(|node| node.position.distance(cursor_world_pos))
                    .enumerate()
                    .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(Ordering::Less))
                    .map(|(id, _)| id)
            } else {
                None
            };

            debug_canvas.set_draw_color(Color::GREEN);
            {
                let rects = colliders
                    .iter()
                    .map(|(collider, position)| {
                        let pos = position.get_pixel_position(&map.graph).unwrap();

                        // Transform position and size using common methods
                        let pos = (pos * scale).as_ivec2();
                        let size = (collider.size * scale) as u32;

                        Rect::from_center(Point::from((pos.x, pos.y)), size, size)
                    })
                    .collect::<SmallVec<[Rect; 100]>>();
                if rects.len() > rects.capacity() {
                    warn!(
                        capacity = rects.capacity(),
                        count = rects.len(),
                        "Collider rects capacity exceeded"
                    );
                }
                debug_canvas.draw_rects(&rects).unwrap();
            }

            debug_canvas.set_draw_color(Color {
                a: f32_to_u8(0.4),
                ..Color::RED
            });
            debug_canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
            for (start_node, end_node) in map.graph.edges() {
                let start_node_model = map.graph.get_node(start_node).unwrap();
                let end_node = map.graph.get_node(end_node.target).unwrap().position;

                // Transform positions using common method
                let start = transform_position_with_offset(start_node_model.position, scale);
                let end = transform_position_with_offset(end_node, scale);

                debug_canvas
                    .draw_line(Point::from((start.x, start.y)), Point::from((end.x, end.y)))
                    .unwrap();
            }

            {
                let rects: Vec<_> = map
                    .graph
                    .nodes()
                    .enumerate()
                    .filter_map(|(id, node)| {
                        let pos = transform_position_with_offset(node.position, scale);
                        let size = (2.0 * scale) as u32;
                        let rect = Rect::new(pos.x - (size as i32 / 2), pos.y - (size as i32 / 2), size, size);

                        // If the node is the one closest to the cursor, draw it immediately
                        if closest_node == Some(id) {
                            debug_canvas.set_draw_color(Color::YELLOW);
                            debug_canvas.fill_rect(rect).unwrap();
                            return None;
                        }

                        Some(rect)
                    })
                    .collect();

                if rects.len() > rects.capacity() {
                    warn!(
                        capacity = rects.capacity(),
                        count = rects.len(),
                        "Node rects capacity exceeded"
                    );
                }

                // Draw the non-closest nodes all at once in blue
                debug_canvas.set_draw_color(Color::BLUE);
                debug_canvas.fill_rects(&rects).unwrap();
            }

            // Render node ID if a node is highlighted
            if let Some(closest_node_id) = closest_node {
                let node = map.graph.get_node(closest_node_id as NodeId).unwrap();
                let pos = transform_position_with_offset(node.position, scale);

                let node_id_text = closest_node_id.to_string();
                let text_pos = Vec2::new((pos.x + 10) as f32, (pos.y - 5) as f32);

                text_renderer
                    .render_text(
                        debug_canvas,
                        &mut ttf_atlas.0,
                        &node_id_text,
                        text_pos,
                        Color {
                            a: f32_to_u8(0.4),
                            ..Color::WHITE
                        },
                    )
                    .unwrap();
            }

            // Render timing information in the top-left corner
            render_timing_display(debug_canvas, &timings, &text_renderer, &mut ttf_atlas.0);
        })
        .unwrap();
}
