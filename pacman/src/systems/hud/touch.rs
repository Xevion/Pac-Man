use crate::error::{GameError, TextureError};
use crate::systems::{BackbufferResource, TouchState};
use bevy_ecs::event::EventWriter;
use bevy_ecs::system::{NonSendMut, Res};
use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::render::{BlendMode, Canvas};
use sdl2::video::Window;

/// Renders touch UI overlay for mobile/testing.
pub fn touch_ui_render_system(
    mut backbuffer: NonSendMut<BackbufferResource>,
    mut canvas: NonSendMut<&mut Canvas<Window>>,
    touch_state: Res<TouchState>,
    mut errors: EventWriter<GameError>,
) {
    if let Some(ref touch_data) = touch_state.active_touch {
        let _ = canvas.with_texture_canvas(&mut backbuffer.0, |canvas| {
            // Set blend mode for transparency
            canvas.set_blend_mode(BlendMode::Blend);

            // Draw semi-transparent circle at touch start position
            canvas.set_draw_color(Color::RGBA(255, 255, 255, 100));
            let center = Point::new(touch_data.start_pos.x as i32, touch_data.start_pos.y as i32);

            // Draw a simple circle by drawing filled rectangles (basic approach)
            let radius = 30;
            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    if dx * dx + dy * dy <= radius * radius {
                        let point = Point::new(center.x + dx, center.y + dy);
                        if let Err(e) = canvas.draw_point(point) {
                            errors.write(TextureError::RenderFailed(format!("Touch UI render error: {}", e)).into());
                            return;
                        }
                    }
                }
            }

            // Draw direction indicator if we have a direction
            if let Some(direction) = touch_data.current_direction {
                canvas.set_draw_color(Color::RGBA(0, 255, 0, 150));

                // Draw arrow indicating direction
                let arrow_length = 40;
                let (dx, dy) = match direction {
                    crate::map::direction::Direction::Up => (0, -arrow_length),
                    crate::map::direction::Direction::Down => (0, arrow_length),
                    crate::map::direction::Direction::Left => (-arrow_length, 0),
                    crate::map::direction::Direction::Right => (arrow_length, 0),
                };

                let end_point = Point::new(center.x + dx, center.y + dy);
                if let Err(e) = canvas.draw_line(center, end_point) {
                    errors.write(TextureError::RenderFailed(format!("Touch arrow render error: {}", e)).into());
                }

                // Draw arrowhead (simple approach)
                let arrow_size = 8;
                match direction {
                    crate::map::direction::Direction::Up => {
                        let _ = canvas.draw_line(end_point, Point::new(end_point.x - arrow_size, end_point.y + arrow_size));
                        let _ = canvas.draw_line(end_point, Point::new(end_point.x + arrow_size, end_point.y + arrow_size));
                    }
                    crate::map::direction::Direction::Down => {
                        let _ = canvas.draw_line(end_point, Point::new(end_point.x - arrow_size, end_point.y - arrow_size));
                        let _ = canvas.draw_line(end_point, Point::new(end_point.x + arrow_size, end_point.y - arrow_size));
                    }
                    crate::map::direction::Direction::Left => {
                        let _ = canvas.draw_line(end_point, Point::new(end_point.x + arrow_size, end_point.y - arrow_size));
                        let _ = canvas.draw_line(end_point, Point::new(end_point.x + arrow_size, end_point.y + arrow_size));
                    }
                    crate::map::direction::Direction::Right => {
                        let _ = canvas.draw_line(end_point, Point::new(end_point.x - arrow_size, end_point.y - arrow_size));
                        let _ = canvas.draw_line(end_point, Point::new(end_point.x - arrow_size, end_point.y + arrow_size));
                    }
                }
            }
        });
    }
}
