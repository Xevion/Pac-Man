use crate::constants::CANVAS_SIZE;
use crate::error::{GameError, TextureError};
use crate::map::builder::Map;
use crate::systems::{
    debug_render_system, BatchedLinesResource, Collider, CursorPosition, DebugState, DebugTextureResource, DeltaTime,
    DirectionalAnimation, LinearAnimation, Position, Renderable, ScoreResource, StartupSequence, SystemId, SystemTimings,
    TtfAtlasResource, Velocity,
};
use crate::texture::sprite::SpriteAtlas;
use crate::texture::text::TextTexture;
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::EventWriter;
use bevy_ecs::query::{Changed, Or, Without};
use bevy_ecs::removal_detection::RemovedComponents;
use bevy_ecs::resource::Resource;
use bevy_ecs::system::{NonSendMut, Query, Res, ResMut};
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{BlendMode, Canvas, Texture};
use sdl2::video::Window;
use std::time::Instant;

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
    if !changed.is_empty() || !removed_hidden.is_empty() || !removed_renderables.is_empty() {
        dirty.0 = true;
    }
}

/// Updates directional animated entities with synchronized timing across directions.
///
/// This runs before the render system to update sprites based on current direction and movement state.
/// All directions share the same frame timing to ensure perfect synchronization.
pub fn directional_render_system(
    dt: Res<DeltaTime>,
    mut query: Query<(&Position, &Velocity, &mut DirectionalAnimation, &mut Renderable)>,
) {
    let ticks = (dt.0 * 60.0).round() as u16; // Convert from seconds to ticks at 60 ticks/sec

    for (position, velocity, mut anim, mut renderable) in query.iter_mut() {
        let stopped = matches!(position, Position::Stopped { .. });

        // Only tick animation when moving to preserve stopped frame
        if !stopped {
            // Tick shared animation state
            anim.time_bank += ticks;
            while anim.time_bank >= anim.frame_duration {
                anim.time_bank -= anim.frame_duration;
                anim.current_frame += 1;
            }
        }

        // Get tiles for current direction and movement state
        let tiles = if stopped {
            anim.stopped_tiles.get(velocity.direction)
        } else {
            anim.moving_tiles.get(velocity.direction)
        };

        if !tiles.is_empty() {
            let new_tile = tiles.get_tile(anim.current_frame);
            if renderable.sprite != new_tile {
                renderable.sprite = new_tile;
            }
        }
    }
}

/// Updates linear animated entities (used for non-directional animations like frightened ghosts).
///
/// This system handles entities that use LinearAnimation component for simple frame cycling.
pub fn linear_render_system(dt: Res<DeltaTime>, mut query: Query<(&mut LinearAnimation, &mut Renderable)>) {
    let ticks = (dt.0 * 60.0).round() as u16; // Convert from seconds to ticks at 60 ticks/sec

    for (mut anim, mut renderable) in query.iter_mut() {
        // Tick animation
        anim.time_bank += ticks;
        while anim.time_bank >= anim.frame_duration {
            anim.time_bank -= anim.frame_duration;
            anim.current_frame += 1;
        }

        if !anim.tiles.is_empty() {
            let new_tile = anim.tiles.get_tile(anim.current_frame);
            if renderable.sprite != new_tile {
                renderable.sprite = new_tile;
            }
        }
    }
}

/// A non-send resource for the map texture. This just wraps the texture with a type so it can be differentiated when exposed as a resource.
pub struct MapTextureResource(pub Texture);

/// A non-send resource for the backbuffer texture. This just wraps the texture with a type so it can be differentiated when exposed as a resource.
pub struct BackbufferResource(pub Texture);

/// Renders the HUD (score, lives, etc.) on top of the game.
pub fn hud_render_system(
    mut backbuffer: NonSendMut<BackbufferResource>,
    mut canvas: NonSendMut<&mut Canvas<Window>>,
    mut atlas: NonSendMut<SpriteAtlas>,
    score: Res<ScoreResource>,
    startup: Res<StartupSequence>,
    mut errors: EventWriter<GameError>,
) {
    let _ = canvas.with_texture_canvas(&mut backbuffer.0, |canvas| {
        let mut text_renderer = TextTexture::new(1.0);

        // Render lives and high score text in white
        let lives = 3; // TODO: Get from actual lives resource
        let lives_text = format!("{lives}UP   HIGH SCORE   ");
        let lives_position = glam::UVec2::new(4 + 8 * 3, 2); // x_offset + lives_offset * 8, y_offset

        if let Err(e) = text_renderer.render(canvas, &mut atlas, &lives_text, lives_position) {
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

        // Render text based on StartupSequence stage
        if matches!(
            *startup,
            StartupSequence::TextOnly { .. } | StartupSequence::CharactersVisible { .. }
        ) {
            let ready_text = "READY!";
            let ready_width = text_renderer.text_width(ready_text);
            let ready_position = glam::UVec2::new((CANVAS_SIZE.x - ready_width) / 2, 160);
            if let Err(e) = text_renderer.render_with_color(canvas, &mut atlas, ready_text, ready_position, Color::YELLOW) {
                errors.write(TextureError::RenderFailed(format!("Failed to render READY text: {}", e)).into());
            }

            if matches!(*startup, StartupSequence::TextOnly { .. }) {
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
pub fn render_system(
    canvas: &mut Canvas<Window>,
    map_texture: &NonSendMut<MapTextureResource>,
    atlas: &mut SpriteAtlas,
    map: &Res<Map>,
    dirty: &Res<RenderDirty>,
    renderables: &Query<(Entity, &Renderable, &Position), Without<Hidden>>,
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
    for (_, renderable, position) in renderables
        .iter()
        .sort_by_key::<(Entity, &Renderable, &Position), _>(|(_, renderable, _)| renderable.layer)
        .rev()
    {
        let pos = position.get_pixel_position(&map.graph);
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
    map: Res<Map>,
    dirty: Res<RenderDirty>,
    renderables: Query<(Entity, &Renderable, &Position), Without<Hidden>>,
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
    if let Some(duration) = render_duration {
        timings.add_timing(SystemId::Render, duration);
    }
    if let Some(duration) = debug_render_duration {
        timings.add_timing(SystemId::DebugRender, duration);
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
