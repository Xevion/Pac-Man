//! Debug rendering system
use std::cmp::Ordering;

use crate::constants::BOARD_PIXEL_OFFSET;
use crate::map::builder::Map;
use crate::systems::components::Collider;
use crate::systems::movement::Position;
use crate::systems::profiling::SystemTimings;
use crate::systems::render::BackbufferResource;
use bevy_ecs::prelude::*;
use glam::Vec2;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};

#[derive(Resource, Default, Debug, Copy, Clone)]
pub struct CursorPosition(pub Vec2);

#[derive(Resource, Default, Debug, Copy, Clone, PartialEq)]
pub enum DebugState {
    #[default]
    Off,
    Graph,
    Collision,
}

impl DebugState {
    pub fn next(&self) -> Self {
        match self {
            DebugState::Off => DebugState::Graph,
            DebugState::Graph => DebugState::Collision,
            DebugState::Collision => DebugState::Off,
        }
    }
}

/// Resource to hold the debug texture for persistent rendering
pub struct DebugTextureResource(pub Texture<'static>);

/// Transforms a position from logical canvas coordinates to output canvas coordinates
fn transform_position(pos: (f32, f32), output_size: (u32, u32), logical_size: (u32, u32)) -> (i32, i32) {
    let scale_x = output_size.0 as f32 / logical_size.0 as f32;
    let scale_y = output_size.1 as f32 / logical_size.1 as f32;
    let scale = scale_x.min(scale_y);

    let offset_x = (output_size.0 as f32 - logical_size.0 as f32 * scale) / 2.0;
    let offset_y = (output_size.1 as f32 - logical_size.1 as f32 * scale) / 2.0;

    let x = (pos.0 * scale + offset_x) as i32;
    let y = (pos.1 * scale + offset_y) as i32;
    (x, y)
}

/// Transforms a position from logical canvas coordinates to output canvas coordinates (with board offset)
fn transform_position_with_offset(pos: (f32, f32), output_size: (u32, u32), logical_size: (u32, u32)) -> (i32, i32) {
    let scale_x = output_size.0 as f32 / logical_size.0 as f32;
    let scale_y = output_size.1 as f32 / logical_size.1 as f32;
    let scale = scale_x.min(scale_y);

    let offset_x = (output_size.0 as f32 - logical_size.0 as f32 * scale) / 2.0;
    let offset_y = (output_size.1 as f32 - logical_size.1 as f32 * scale) / 2.0;

    let x = ((pos.0 + BOARD_PIXEL_OFFSET.x as f32) * scale + offset_x) as i32;
    let y = ((pos.1 + BOARD_PIXEL_OFFSET.y as f32) * scale + offset_y) as i32;
    (x, y)
}

/// Transforms a size from logical canvas coordinates to output canvas coordinates
fn transform_size(size: f32, output_size: (u32, u32), logical_size: (u32, u32)) -> u32 {
    let scale_x = output_size.0 as f32 / logical_size.0 as f32;
    let scale_y = output_size.1 as f32 / logical_size.1 as f32;
    let scale = scale_x.min(scale_y); // Use the smaller scale to maintain aspect ratio

    (size * scale) as u32
}

/// Renders timing information in the top-left corner of the screen
fn render_timing_display(
    canvas: &mut Canvas<Window>,
    texture_creator: &mut TextureCreator<WindowContext>,
    timings: &SystemTimings,
) {
    // Get TTF context
    let ttf_context = sdl2::ttf::init().unwrap();

    // Load font
    let font = ttf_context.load_font("assets/site/TerminalVector.ttf", 12).unwrap();

    // Format timing information using the formatting module
    let lines = timings.format_timing_display();
    let line_height = 14; // Approximate line height for 12pt font
    let padding = 10;

    // Calculate background dimensions
    let max_width = lines
        .iter()
        .filter(|l| !l.is_empty()) // Don't consider empty lines for width
        .map(|line| font.size_of(line).unwrap().0)
        .max()
        .unwrap_or(0);

    // Only draw background if there is text to display
    if max_width > 0 {
        let total_height = (lines.len() as u32) * line_height as u32;
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

        // Render each line
        let surface = font.render(line).blended(Color::RGBA(255, 255, 255, 200)).unwrap();
        let texture = texture_creator.create_texture_from_surface(&surface).unwrap();

        // Position each line below the previous one
        let y_pos = padding + (i * line_height) as i32;
        let dest = Rect::new(padding, y_pos, texture.query().width, texture.query().height);
        canvas.copy(&texture, None, dest).unwrap();
    }
}

pub fn debug_render_system(
    mut canvas: NonSendMut<&mut Canvas<Window>>,
    backbuffer: NonSendMut<BackbufferResource>,
    mut debug_texture: NonSendMut<DebugTextureResource>,
    debug_state: Res<DebugState>,
    timings: Res<SystemTimings>,
    map: Res<Map>,
    colliders: Query<(&Collider, &Position)>,
    cursor: Res<CursorPosition>,
) {
    if *debug_state == DebugState::Off {
        return;
    }

    // Get canvas sizes for coordinate transformation
    let output_size = canvas.output_size().unwrap();
    let logical_size = canvas.logical_size();

    // Copy the current backbuffer to the debug texture
    canvas
        .with_texture_canvas(&mut debug_texture.0, |debug_canvas| {
            // Clear the debug canvas
            debug_canvas.set_draw_color(Color::BLACK);
            debug_canvas.clear();

            // Copy the backbuffer to the debug canvas
            debug_canvas.copy(&backbuffer.0, None, None).unwrap();
        })
        .unwrap();

    // Get texture creator before entering the closure to avoid borrowing conflicts
    let mut texture_creator = canvas.texture_creator();

    let cursor_world_pos = cursor.0 - BOARD_PIXEL_OFFSET.as_vec2();

    // Draw debug info on the high-resolution debug texture
    canvas
        .with_texture_canvas(&mut debug_texture.0, |debug_canvas| {
            match *debug_state {
                DebugState::Graph => {
                    // Find the closest node to the cursor

                    let closest_node = map
                        .graph
                        .nodes()
                        .map(|node| node.position.distance(cursor_world_pos))
                        .enumerate()
                        .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(Ordering::Less))
                        .map(|(id, _)| id);

                    debug_canvas.set_draw_color(Color::RED);
                    for (start_node, end_node) in map.graph.edges() {
                        let start_node_model = map.graph.get_node(start_node).unwrap();
                        let end_node = map.graph.get_node(end_node.target).unwrap().position;

                        // Transform positions using common method
                        let (start_x, start_y) = transform_position_with_offset(
                            (start_node_model.position.x, start_node_model.position.y),
                            output_size,
                            logical_size,
                        );
                        let (end_x, end_y) = transform_position_with_offset((end_node.x, end_node.y), output_size, logical_size);

                        debug_canvas.draw_line((start_x, start_y), (end_x, end_y)).unwrap();
                    }

                    for (id, node) in map.graph.nodes().enumerate() {
                        let pos = node.position;

                        // Set color based on whether the node is the closest to the cursor
                        if Some(id) == closest_node {
                            debug_canvas.set_draw_color(Color::YELLOW);
                        } else {
                            debug_canvas.set_draw_color(Color::BLUE);
                        }

                        // Transform position using common method
                        let (x, y) = transform_position_with_offset((pos.x, pos.y), output_size, logical_size);
                        let size = transform_size(4.0, output_size, logical_size);

                        debug_canvas
                            .fill_rect(Rect::new(x - (size as i32 / 2), y - (size as i32 / 2), size, size))
                            .unwrap();
                    }
                }
                DebugState::Collision => {
                    debug_canvas.set_draw_color(Color::GREEN);
                    for (collider, position) in colliders.iter() {
                        let pos = position.get_pixel_position(&map.graph).unwrap();

                        // Transform position and size using common methods
                        let (x, y) = transform_position((pos.x, pos.y), output_size, logical_size);
                        let size = transform_size(collider.size, output_size, logical_size);

                        // Center the collision box on the entity
                        let rect = Rect::new(x - (size as i32 / 2), y - (size as i32 / 2), size, size);
                        debug_canvas.draw_rect(rect).unwrap();
                    }
                }
                _ => {}
            }

            // Render node ID if a node is highlighted
            if let DebugState::Graph = *debug_state {
                if let Some(closest_node_id) = map
                    .graph
                    .nodes()
                    .map(|node| node.position.distance(cursor_world_pos))
                    .enumerate()
                    .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(Ordering::Less))
                    .map(|(id, _)| id)
                {
                    let node = map.graph.get_node(closest_node_id).unwrap();
                    let (x, y) = transform_position_with_offset((node.position.x, node.position.y), output_size, logical_size);

                    let ttf_context = sdl2::ttf::init().unwrap();
                    let font = ttf_context.load_font("assets/site/TerminalVector.ttf", 12).unwrap();
                    let surface = font.render(&closest_node_id.to_string()).blended(Color::WHITE).unwrap();
                    let texture = texture_creator.create_texture_from_surface(&surface).unwrap();
                    let dest = Rect::new(x + 10, y - 5, texture.query().width, texture.query().height);
                    debug_canvas.copy(&texture, None, dest).unwrap();
                }
            }

            // Render timing information in the top-left corner
            render_timing_display(debug_canvas, &mut texture_creator, &timings);
        })
        .unwrap();

    // Draw the debug texture directly onto the main canvas at full resolution
    canvas.copy(&debug_texture.0, None, None).unwrap();
}
