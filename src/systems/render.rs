use crate::map::builder::Map;
use crate::map::direction::Direction;
use crate::systems::{
    debug_render_system, BatchedLinesResource, Collider, CursorPosition, DebugState, DebugTextureResource, GameStage, PlayerLife,
    PlayerLives, Position, ScoreResource, StartupSequence, SystemId, SystemTimings, TouchState, TtfAtlasResource,
};
use crate::texture::sprite::{AtlasTile, SpriteAtlas};
use crate::texture::sprites::{GameSprite, PacmanSprite};
use crate::texture::text::TextTexture;
use crate::{
    constants::{BOARD_BOTTOM_PIXEL_OFFSET, CANVAS_SIZE, CELL_SIZE},
    error::{GameError, TextureError},
};
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::EventWriter;
use bevy_ecs::query::{Changed, Or, With, Without};
use bevy_ecs::removal_detection::RemovedComponents;
use bevy_ecs::resource::Resource;
use bevy_ecs::system::{Commands, NonSendMut, Query, Res, ResMut};
use glam::Vec2;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{BlendMode, Canvas, Texture};
use sdl2::video::Window;
use std::cmp::Ordering;
use std::time::Instant;

/// A component for entities that have a sprite, with a layer for ordering.
///
/// This is intended to be modified by other entities allowing animation.
#[derive(Component)]
pub struct Renderable {
    pub sprite: AtlasTile,
    pub layer: u8,
}

#[derive(Resource, Default)]
pub struct RenderDirty(pub bool);

#[derive(Component)]
pub struct Hidden;

/// Enum to identify which texture is being rendered to in the combined render system
#[derive(Debug, Clone, Copy)]
enum RenderTarget {
    Backbuffer,
    Debug,
}

#[allow(clippy::type_complexity)]
pub fn dirty_render_system(
    mut dirty: ResMut<RenderDirty>,
    changed: Query<(), Or<(Changed<Renderable>, Changed<Position>)>>,
    removed_hidden: RemovedComponents<Hidden>,
    removed_renderables: RemovedComponents<Renderable>,
) {
    let changed_count = changed.iter().count();
    let removed_hidden_count = removed_hidden.len();
    let removed_renderables_count = removed_renderables.len();

    if changed_count > 0 || removed_hidden_count > 0 || removed_renderables_count > 0 {
        dirty.0 = true;
    }
}

/// System that manages player life sprite entities.
/// Spawns and despawns life sprite entities based on changes to PlayerLives resource.
/// Each life sprite is positioned based on its index (0, 1, 2, etc. from left to right).
pub fn player_life_sprite_system(
    mut commands: Commands,
    atlas: NonSendMut<SpriteAtlas>,
    current_life_sprites: Query<(Entity, &PlayerLife)>,
    player_lives: Res<PlayerLives>,
    mut errors: EventWriter<GameError>,
) {
    let displayed_lives = player_lives.0.saturating_sub(1);

    // Get current life sprite entities, sorted by index
    let mut current_sprites: Vec<_> = current_life_sprites.iter().collect();
    current_sprites.sort_by_key(|(_, life)| life.index);
    let current_count = current_sprites.len() as u8;

    // Calculate the difference
    let diff = (displayed_lives as i8) - (current_count as i8);

    match diff.cmp(&0) {
        // Ignore when the number of lives displayed is correct
        Ordering::Equal => {}
        // Spawn new life sprites
        Ordering::Greater => {
            let life_sprite = match atlas.get_tile(&GameSprite::Pacman(PacmanSprite::Moving(Direction::Left, 1)).to_path()) {
                Ok(sprite) => sprite,
                Err(e) => {
                    errors.write(e.into());
                    return;
                }
            };

            for i in 0..diff {
                let position = calculate_life_sprite_position(i as u32);

                commands.spawn((
                    PlayerLife { index: i as u32 },
                    Renderable {
                        sprite: life_sprite,
                        layer: 255, // High layer to render on top
                    },
                    PixelPosition {
                        pixel_position: position,
                    },
                ));
            }
        }
        // Remove excess life sprites (highest indices first)
        Ordering::Less => {
            let to_remove = diff.unsigned_abs();
            let sprites_to_remove: Vec<_> = current_sprites
                .iter()
                .rev() // Start from highest index
                .take(to_remove as usize)
                .map(|(entity, _)| *entity)
                .collect();

            for entity in sprites_to_remove {
                commands.entity(entity).despawn();
            }
        }
    }
}

/// Component for Renderables to store an exact pixel position
#[derive(Component)]
pub struct PixelPosition {
    pub pixel_position: Vec2,
}

/// Calculates the pixel position for a life sprite based on its index
fn calculate_life_sprite_position(index: u32) -> Vec2 {
    let start_x = CELL_SIZE * 2; // 2 cells from left
    let start_y = CANVAS_SIZE.y - BOARD_BOTTOM_PIXEL_OFFSET.y + (CELL_SIZE / 2) + 1; // In bottom area
    let sprite_spacing = CELL_SIZE + CELL_SIZE / 2; // 1.5 cells between sprites

    let x = start_x + ((index as f32) * (sprite_spacing as f32 * 1.5)).round() as u32;
    let y = start_y - CELL_SIZE / 2;

    Vec2::new((x + CELL_SIZE) as f32, (y + CELL_SIZE) as f32)
}

/// A non-send resource for the map texture. This just wraps the texture with a type so it can be differentiated when exposed as a resource.
pub struct MapTextureResource(pub Texture);

/// A non-send resource for the backbuffer texture. This just wraps the texture with a type so it can be differentiated when exposed as a resource.
pub struct BackbufferResource(pub Texture);

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

/// Renders the HUD (score, lives, etc.) on top of the game.
#[allow(clippy::too_many_arguments)]
pub fn hud_render_system(
    mut backbuffer: NonSendMut<BackbufferResource>,
    mut canvas: NonSendMut<&mut Canvas<Window>>,
    mut atlas: NonSendMut<SpriteAtlas>,
    score: Res<ScoreResource>,
    stage: Res<GameStage>,
    mut errors: EventWriter<GameError>,
) {
    let _ = canvas.with_texture_canvas(&mut backbuffer.0, |canvas| {
        let mut text_renderer = TextTexture::new(1.0);

        // Render lives and high score text in white
        let lives_text = "1UP   HIGH SCORE   ";
        let lives_position = glam::UVec2::new(4 + 8 * 3, 2); // x_offset + lives_offset * 8, y_offset

        if let Err(e) = text_renderer.render(canvas, &mut atlas, lives_text, lives_position) {
            errors.write(TextureError::RenderFailed(format!("Failed to render lives text: {}", e)).into());
        }

        // Render score text
        let score_text = format!("{:02}", score.0);
        let score_offset = 7 - (score_text.len() as i32);
        let score_position = glam::UVec2::new(4 + 8 * score_offset as u32, 10); // x_offset + score_offset * 8, 8 + y_offset

        if let Err(e) = text_renderer.render(canvas, &mut atlas, &score_text, score_position) {
            errors.write(TextureError::RenderFailed(format!("Failed to render score text: {}", e)).into());
        }

        // Render high score text
        let high_score_text = format!("{:02}", score.0);
        let high_score_offset = 17 - (high_score_text.len() as i32);
        let high_score_position = glam::UVec2::new(4 + 8 * high_score_offset as u32, 10); // x_offset + score_offset * 8, 8 + y_offset
        if let Err(e) = text_renderer.render(canvas, &mut atlas, &high_score_text, high_score_position) {
            errors.write(TextureError::RenderFailed(format!("Failed to render high score text: {}", e)).into());
        }

        // Render GAME OVER text
        if matches!(*stage, GameStage::GameOver) {
            let game_over_text = "GAME  OVER";
            let game_over_width = text_renderer.text_width(game_over_text);
            let game_over_position = glam::UVec2::new((CANVAS_SIZE.x - game_over_width) / 2, 160);
            if let Err(e) = text_renderer.render_with_color(canvas, &mut atlas, game_over_text, game_over_position, Color::RED) {
                errors.write(TextureError::RenderFailed(format!("Failed to render GAME OVER text: {}", e)).into());
            }
        }

        // Render text based on StartupSequence stage
        if matches!(
            *stage,
            GameStage::Starting(StartupSequence::TextOnly { .. })
                | GameStage::Starting(StartupSequence::CharactersVisible { .. })
        ) {
            let ready_text = "READY!";
            let ready_width = text_renderer.text_width(ready_text);
            let ready_position = glam::UVec2::new((CANVAS_SIZE.x - ready_width) / 2, 160);
            if let Err(e) = text_renderer.render_with_color(canvas, &mut atlas, ready_text, ready_position, Color::YELLOW) {
                errors.write(TextureError::RenderFailed(format!("Failed to render READY text: {}", e)).into());
            }

            if matches!(*stage, GameStage::Starting(StartupSequence::TextOnly { .. })) {
                let player_one_text = "PLAYER ONE";
                let player_one_width = text_renderer.text_width(player_one_text);
                let player_one_position = glam::UVec2::new((CANVAS_SIZE.x - player_one_width) / 2, 113);

                if let Err(e) =
                    text_renderer.render_with_color(canvas, &mut atlas, player_one_text, player_one_position, Color::CYAN)
                {
                    errors.write(TextureError::RenderFailed(format!("Failed to render PLAYER ONE text: {}", e)).into());
                }
            }
        }
    });
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn render_system(
    canvas: &mut Canvas<Window>,
    map_texture: &NonSendMut<MapTextureResource>,
    atlas: &mut SpriteAtlas,
    map: &Res<Map>,
    dirty: &Res<RenderDirty>,
    renderables: &Query<
        (Entity, &Renderable, Option<&Position>, Option<&PixelPosition>),
        (Without<Hidden>, Or<(With<Position>, With<PixelPosition>)>),
    >,
    errors: &mut EventWriter<GameError>,
) {
    if !dirty.0 {
        return;
    }

    // Clear the backbuffer
    canvas.set_draw_color(sdl2::pixels::Color::BLACK);
    canvas.clear();

    // Copy the pre-rendered map texture to the backbuffer
    if let Err(e) = canvas.copy(&map_texture.0, None, None) {
        errors.write(TextureError::RenderFailed(e.to_string()).into());
    }

    // Render all entities to the backbuffer
    for (_entity, renderable, position, pixel_position) in renderables
        .iter()
        .sort_by_key::<(Entity, &Renderable, Option<&Position>, Option<&PixelPosition>), _>(|(_, renderable, _, _)| {
            renderable.layer
        })
        .rev()
    {
        let pos = if let Some(position) = position {
            position.get_pixel_position(&map.graph)
        } else {
            Ok(pixel_position
                .expect("Pixel position should be present via query filtering, but got None on both")
                .pixel_position)
        };

        match pos {
            Ok(pos) => {
                let dest = Rect::from_center(
                    Point::from((pos.x as i32, pos.y as i32)),
                    renderable.sprite.size.x as u32,
                    renderable.sprite.size.y as u32,
                );

                renderable
                    .sprite
                    .render(canvas, atlas, dest)
                    .err()
                    .map(|e| errors.write(TextureError::RenderFailed(e.to_string()).into()));
            }
            Err(e) => {
                errors.write(e);
            }
        }
    }
}

/// Combined render system that renders to both backbuffer and debug textures in a single
/// with_multiple_texture_canvas call for reduced overhead
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn combined_render_system(
    mut canvas: NonSendMut<&mut Canvas<Window>>,
    map_texture: NonSendMut<MapTextureResource>,
    mut backbuffer: NonSendMut<BackbufferResource>,
    mut debug_texture: NonSendMut<DebugTextureResource>,
    mut atlas: NonSendMut<SpriteAtlas>,
    mut ttf_atlas: NonSendMut<TtfAtlasResource>,
    batched_lines: Res<BatchedLinesResource>,
    debug_state: Res<DebugState>,
    timings: Res<SystemTimings>,
    timing: Res<crate::systems::profiling::Timing>,
    map: Res<Map>,
    dirty: Res<RenderDirty>,
    renderables: Query<
        (Entity, &Renderable, Option<&Position>, Option<&PixelPosition>),
        (Without<Hidden>, Or<(With<Position>, With<PixelPosition>)>),
    >,
    colliders: Query<(&Collider, &Position)>,
    cursor: Res<CursorPosition>,
    mut errors: EventWriter<GameError>,
) {
    if !dirty.0 {
        return;
    }

    // Prepare textures and render targets
    let textures = [
        (&mut backbuffer.0, RenderTarget::Backbuffer),
        (&mut debug_texture.0, RenderTarget::Debug),
    ];

    // Record timing for each system independently
    let mut render_duration = None;
    let mut debug_render_duration = None;

    let result = canvas.with_multiple_texture_canvas(textures.iter(), |texture_canvas, render_target| match render_target {
        RenderTarget::Backbuffer => {
            let start_time = Instant::now();

            render_system(
                texture_canvas,
                &map_texture,
                &mut atlas,
                &map,
                &dirty,
                &renderables,
                &mut errors,
            );

            render_duration = Some(start_time.elapsed());
        }
        RenderTarget::Debug => {
            if !debug_state.enabled {
                return;
            }

            let start_time = Instant::now();

            debug_render_system(
                texture_canvas,
                &mut ttf_atlas,
                &batched_lines,
                &debug_state,
                &timings,
                &timing,
                &map,
                &colliders,
                &cursor,
            );

            debug_render_duration = Some(start_time.elapsed());
        }
    });

    if let Err(e) = result {
        errors.write(TextureError::RenderFailed(e.to_string()).into());
    }

    // Record timings for each system independently
    let current_tick = timing.get_current_tick();

    if let Some(duration) = render_duration {
        timings.add_timing(SystemId::Render, duration, current_tick);
    }
    if let Some(duration) = debug_render_duration {
        timings.add_timing(SystemId::DebugRender, duration, current_tick);
    }
}

pub fn present_system(
    mut canvas: NonSendMut<&mut Canvas<Window>>,
    mut dirty: ResMut<RenderDirty>,
    backbuffer: NonSendMut<BackbufferResource>,
    debug_texture: NonSendMut<DebugTextureResource>,
    debug_state: Res<DebugState>,
) {
    if dirty.0 {
        // Copy the backbuffer to the main canvas
        canvas.copy(&backbuffer.0, None, None).unwrap();

        // Copy the debug texture to the canvas
        if debug_state.enabled {
            canvas.set_blend_mode(BlendMode::Blend);
            canvas.copy(&debug_texture.0, None, None).unwrap();
        }

        canvas.present();
        dirty.0 = false;
    }
}
