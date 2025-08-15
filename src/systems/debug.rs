//! Debug rendering system
use crate::constants::BOARD_PIXEL_OFFSET;
use crate::map::builder::Map;
use crate::systems::components::Collider;
use crate::systems::movement::Position;
use crate::systems::render::BackbufferResource;
use bevy_ecs::prelude::*;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;

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
    let scale = scale_x.min(scale_y); // Use the smaller scale to maintain aspect ratio

    let x = (pos.0 * scale) as i32;
    let y = (pos.1 * scale) as i32;
    (x, y)
}

/// Transforms a position from logical canvas coordinates to output canvas coordinates (with board offset)
fn transform_position_with_offset(pos: (f32, f32), output_size: (u32, u32), logical_size: (u32, u32)) -> (i32, i32) {
    let scale_x = output_size.0 as f32 / logical_size.0 as f32;
    let scale_y = output_size.1 as f32 / logical_size.1 as f32;
    let scale = scale_x.min(scale_y); // Use the smaller scale to maintain aspect ratio

    let x = ((pos.0 + BOARD_PIXEL_OFFSET.x as f32) * scale) as i32;
    let y = ((pos.1 + BOARD_PIXEL_OFFSET.y as f32) * scale) as i32;
    (x, y)
}

/// Transforms a size from logical canvas coordinates to output canvas coordinates
fn transform_size(size: f32, output_size: (u32, u32), logical_size: (u32, u32)) -> u32 {
    let scale_x = output_size.0 as f32 / logical_size.0 as f32;
    let scale_y = output_size.1 as f32 / logical_size.1 as f32;
    let scale = scale_x.min(scale_y); // Use the smaller scale to maintain aspect ratio

    (size * scale) as u32
}

pub fn debug_render_system(
    mut canvas: NonSendMut<&mut Canvas<Window>>,
    backbuffer: NonSendMut<BackbufferResource>,
    mut debug_texture: NonSendMut<DebugTextureResource>,
    debug_state: Res<DebugState>,
    map: Res<Map>,
    colliders: Query<(&Collider, &Position)>,
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

    // Draw debug info on the high-resolution debug texture
    canvas
        .with_texture_canvas(&mut debug_texture.0, |debug_canvas| match *debug_state {
            DebugState::Graph => {
                debug_canvas.set_draw_color(Color::RED);
                for (start_node, end_node) in map.graph.edges() {
                    let start_node = map.graph.get_node(start_node).unwrap().position;
                    let end_node = map.graph.get_node(end_node.target).unwrap().position;

                    // Transform positions using common method
                    let (start_x, start_y) =
                        transform_position_with_offset((start_node.x, start_node.y), output_size, logical_size);
                    let (end_x, end_y) = transform_position_with_offset((end_node.x, end_node.y), output_size, logical_size);

                    debug_canvas.draw_line((start_x, start_y), (end_x, end_y)).unwrap();
                }

                debug_canvas.set_draw_color(Color::BLUE);
                for node in map.graph.nodes() {
                    let pos = node.position;

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
                    let pos = position.get_pixel_pos(&map.graph).unwrap();

                    // Transform position and size using common methods
                    let (x, y) = transform_position((pos.x, pos.y), output_size, logical_size);
                    let size = transform_size(collider.size, output_size, logical_size);

                    // Center the collision box on the entity
                    let rect = Rect::new(x - (size as i32 / 2), y - (size as i32 / 2), size, size);
                    debug_canvas.draw_rect(rect).unwrap();
                }
            }
            _ => {}
        })
        .unwrap();

    // Draw the debug texture directly onto the main canvas at full resolution
    canvas.copy(&debug_texture.0, None, None).unwrap();
}
