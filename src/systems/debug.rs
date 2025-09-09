//! Debug rendering system
use crate::constants::{self, BOARD_PIXEL_OFFSET};
use crate::map::builder::Map;
use crate::systems::{Collider, CursorPosition, NodeId, Position, SystemTimings};
use crate::texture::ttf::{TtfAtlas, TtfRenderer};
use bevy_ecs::resource::Resource;
use bevy_ecs::system::{Query, Res};
use glam::{IVec2, Vec2};
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;
use smallvec::SmallVec;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
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

/// Resource to hold pre-computed batched line segments
#[derive(Resource, Default, Debug, Clone)]
pub struct BatchedLinesResource {
    horizontal_lines: Vec<(i32, i32, i32)>, // (y, x_start, x_end)
    vertical_lines: Vec<(i32, i32, i32)>,   // (x, y_start, y_end)
}

impl BatchedLinesResource {
    /// Computes and caches batched line segments for the map graph
    pub fn new(map: &Map, scale: f32) -> Self {
        let mut horizontal_segments: HashMap<i32, Vec<(i32, i32)>> = HashMap::new();
        let mut vertical_segments: HashMap<i32, Vec<(i32, i32)>> = HashMap::new();
        let mut processed_edges: HashSet<(u16, u16)> = HashSet::new();

        // Process all edges and group them by axis
        for (start_node_id, edge) in map.graph.edges() {
            // Acquire a stable key for the edge (from < to)
            let edge_key = (start_node_id.min(edge.target), start_node_id.max(edge.target));

            // Skip if we've already processed this edge in the reverse direction
            if processed_edges.contains(&edge_key) {
                continue;
            }
            processed_edges.insert(edge_key);

            let start_pos = map.graph.get_node(start_node_id).unwrap().position;
            let end_pos = map.graph.get_node(edge.target).unwrap().position;

            let start = transform_position_with_offset(start_pos, scale);
            let end = transform_position_with_offset(end_pos, scale);

            // Determine if this is a horizontal or vertical line
            if (start.y - end.y).abs() < 2 {
                // Horizontal line (allowing for slight vertical variance)
                let y = start.y;
                let x_min = start.x.min(end.x);
                let x_max = start.x.max(end.x);
                horizontal_segments.entry(y).or_default().push((x_min, x_max));
            } else if (start.x - end.x).abs() < 2 {
                // Vertical line (allowing for slight horizontal variance)
                let x = start.x;
                let y_min = start.y.min(end.y);
                let y_max = start.y.max(end.y);
                vertical_segments.entry(x).or_default().push((y_min, y_max));
            }
        }

        /// Merges overlapping or adjacent segments into continuous lines
        fn merge_segments(segments: Vec<(i32, i32)>) -> Vec<(i32, i32)> {
            if segments.is_empty() {
                return Vec::new();
            }

            let mut merged = Vec::new();
            let mut current_start = segments[0].0;
            let mut current_end = segments[0].1;

            for &(start, end) in segments.iter().skip(1) {
                if start <= current_end + 1 {
                    // Adjacent or overlapping
                    current_end = current_end.max(end);
                } else {
                    merged.push((current_start, current_end));
                    current_start = start;
                    current_end = end;
                }
            }

            merged.push((current_start, current_end));
            merged
        }

        // Convert to flat vectors for fast iteration during rendering
        let horizontal_lines = horizontal_segments
            .into_iter()
            .flat_map(|(y, mut segments)| {
                segments.sort_unstable_by_key(|(start, _)| *start);
                let merged = merge_segments(segments);
                merged.into_iter().map(move |(x_start, x_end)| (y, x_start, x_end))
            })
            .collect::<Vec<_>>();

        let vertical_lines = vertical_segments
            .into_iter()
            .flat_map(|(x, mut segments)| {
                segments.sort_unstable_by_key(|(start, _)| *start);
                let merged = merge_segments(segments);
                merged.into_iter().map(move |(y_start, y_end)| (x, y_start, y_end))
            })
            .collect::<Vec<_>>();

        Self {
            horizontal_lines,
            vertical_lines,
        }
    }

    pub fn render(&self, canvas: &mut Canvas<Window>) {
        // Render horizontal lines
        for &(y, x_start, x_end) in &self.horizontal_lines {
            let points = [Point::new(x_start, y), Point::new(x_end, y)];
            let _ = canvas.draw_lines(&points[..]);
        }

        // Render vertical lines
        for &(x, y_start, y_end) in &self.vertical_lines {
            let points = [Point::new(x, y_start), Point::new(x, y_end)];
            let _ = canvas.draw_lines(&points[..]);
        }
    }
}

/// Transforms a position from logical canvas coordinates to output canvas coordinates (with board offset)
fn transform_position_with_offset(pos: Vec2, scale: f32) -> IVec2 {
    ((pos + BOARD_PIXEL_OFFSET.as_vec2()) * scale).as_ivec2()
}

/// Renders timing information in the top-left corner of the screen using the debug text atlas
#[cfg_attr(coverage_nightly, coverage(off))]
fn render_timing_display(
    canvas: &mut Canvas<Window>,
    timings: &SystemTimings,
    current_tick: u64,
    text_renderer: &TtfRenderer,
    atlas: &mut TtfAtlas,
) {
    // Format timing information using the formatting module
    let lines = timings.format_timing_display(current_tick);
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
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn debug_render_system(
    canvas: &mut Canvas<Window>,
    ttf_atlas: &mut TtfAtlasResource,
    batched_lines: &Res<BatchedLinesResource>,
    debug_state: &Res<DebugState>,
    timings: &Res<SystemTimings>,
    timing: &Res<crate::systems::profiling::Timing>,
    map: &Res<Map>,
    colliders: &Query<(&Collider, &Position)>,
    cursor: &Res<CursorPosition>,
) {
    if !debug_state.enabled {
        return;
    }
    // Create debug text renderer
    let text_renderer = TtfRenderer::new(1.0);

    let cursor_world_pos = match &**cursor {
        CursorPosition::None => None,
        CursorPosition::Some { position, .. } => Some(position - BOARD_PIXEL_OFFSET.as_vec2()),
    };

    // Clear the debug canvas
    canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
    canvas.clear();

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

    canvas.set_draw_color(Color::GREEN);
    {
        let rects = colliders
            .iter()
            .map(|(collider, position)| {
                let pos = position.get_pixel_position(&map.graph).unwrap();

                // Transform position and size using common methods
                let pos = (pos * constants::LARGE_SCALE).as_ivec2();
                let size = (collider.size * constants::LARGE_SCALE) as u32;

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
        canvas.draw_rects(&rects).unwrap();
    }

    canvas.set_draw_color(Color {
        a: f32_to_u8(0.65),
        ..Color::RED
    });
    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);

    // Use cached batched line segments
    batched_lines.render(canvas);

    {
        let rects: Vec<_> = map
            .graph
            .nodes()
            .enumerate()
            .filter_map(|(id, node)| {
                let pos = transform_position_with_offset(node.position, constants::LARGE_SCALE);
                let size = (2.0 * constants::LARGE_SCALE) as u32;
                let rect = Rect::new(pos.x - (size as i32 / 2), pos.y - (size as i32 / 2), size, size);

                // If the node is the one closest to the cursor, draw it immediately
                if closest_node == Some(id) {
                    canvas.set_draw_color(Color::YELLOW);
                    canvas.fill_rect(rect).unwrap();
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
        canvas.set_draw_color(Color::BLUE);
        canvas.fill_rects(&rects).unwrap();
    }

    // Render node ID if a node is highlighted
    if let Some(closest_node_id) = closest_node {
        let node = map.graph.get_node(closest_node_id as NodeId).unwrap();
        let pos = transform_position_with_offset(node.position, constants::LARGE_SCALE);

        let node_id_text = closest_node_id.to_string();
        let text_pos = Vec2::new((pos.x + 10) as f32, (pos.y - 5) as f32);

        text_renderer
            .render_text(
                canvas,
                &mut ttf_atlas.0,
                &node_id_text,
                text_pos,
                Color {
                    a: f32_to_u8(0.9),
                    ..Color::WHITE
                },
            )
            .unwrap();
    }

    // Render timing information in the top-left corner
    // Use previous tick since current tick is incomplete (frame is still running)
    let current_tick = timing.get_current_tick();
    let previous_tick = current_tick.saturating_sub(1);
    render_timing_display(canvas, timings, previous_tick, &text_renderer, &mut ttf_atlas.0);
}
