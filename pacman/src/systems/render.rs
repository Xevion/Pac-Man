use crate::error::{GameError, TextureError};
use crate::map::builder::Map;
use crate::systems::{
    debug_render_system, BatchedLinesResource, Collider, CursorPosition, DebugState, DebugTextureResource, Position, SystemId,
    SystemTimings, TouchState, TtfAtlasResource,
};
use crate::texture::sprite::{AtlasTile, SpriteAtlas};
use bevy_ecs::change_detection::DetectChanges;
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::EventWriter;
use bevy_ecs::query::{Changed, Or, With};
use bevy_ecs::removal_detection::RemovedComponents;
use bevy_ecs::resource::Resource;
use bevy_ecs::system::{NonSendMut, Query, Res, ResMut};
use glam::Vec2;
use sdl2::rect::{Point, Rect};
use sdl2::render::{BlendMode, Canvas, Texture};
use sdl2::video::Window;
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

/// A component that controls entity visibility in the render system.
///
/// Entities without this component are considered visible by default.
/// This allows for efficient rendering where only entities that need
/// visibility control have this component.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Visibility(pub bool);

impl Default for Visibility {
    fn default() -> Self {
        Self(true) // Default to visible
    }
}

impl Visibility {
    /// Creates a visible Visibility component
    pub fn visible() -> Self {
        Self(true)
    }

    /// Creates a hidden Visibility component
    pub fn hidden() -> Self {
        Self(false)
    }

    /// Returns true if the entity is visible
    pub fn is_visible(&self) -> bool {
        self.0
    }

    /// Returns true if the entity is hidden
    #[allow(dead_code)] // Used in tests
    pub fn is_hidden(&self) -> bool {
        !self.0
    }

    /// Makes the entity visible
    pub fn show(&mut self) {
        self.0 = true;
    }

    /// Toggles the visibility state
    pub fn toggle(&mut self) {
        self.0 = !self.0;
    }
}

/// Enum to identify which texture is being rendered to in the combined render system
#[derive(Debug, Clone, Copy)]
enum RenderTarget {
    Backbuffer,
    Debug,
}

#[allow(clippy::type_complexity)]
pub fn dirty_render_system(
    mut dirty: ResMut<RenderDirty>,
    changed: Query<(), Or<(Changed<Renderable>, Changed<Position>, Changed<Visibility>)>>,
    removed_renderables: RemovedComponents<Renderable>,
    cursor: Res<CursorPosition>,
    touch_state: Res<TouchState>,
) {
    if changed.iter().count() > 0 || !removed_renderables.is_empty() || cursor.is_changed() || touch_state.is_changed() {
        dirty.0 = true;
    }
}

/// Component for Renderables to store an exact pixel position
#[derive(Component)]
pub struct PixelPosition {
    pub pixel_position: Vec2,
}

/// A non-send resource for the map texture. This just wraps the texture with a type so it can be differentiated when exposed as a resource.
pub struct MapTextureResource(pub Texture);

/// A non-send resource for the backbuffer texture. This just wraps the texture with a type so it can be differentiated when exposed as a resource.
pub struct BackbufferResource(pub Texture);

#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn render_system(
    canvas: &mut Canvas<Window>,
    map_texture: &NonSendMut<MapTextureResource>,
    atlas: &mut SpriteAtlas,
    map: &Res<Map>,
    dirty: &Res<RenderDirty>,
    renderables: &Query<
        (
            Entity,
            &Renderable,
            Option<&Position>,
            Option<&PixelPosition>,
            Option<&Visibility>,
        ),
        Or<(With<Position>, With<PixelPosition>)>,
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

    // Collect and filter visible entities, then sort by layer
    let mut visible_entities: Vec<_> = renderables
        .iter()
        .filter(|(_, _, _, _, visibility)| visibility.copied().unwrap_or_default().is_visible())
        .collect();

    visible_entities.sort_by_key(|(_, renderable, _, _, _)| renderable.layer);
    visible_entities.reverse();

    // Render all visible entities to the backbuffer
    for (_entity, renderable, position, pixel_position, _visibility) in visible_entities {
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
        (
            Entity,
            &Renderable,
            Option<&Position>,
            Option<&PixelPosition>,
            Option<&Visibility>,
        ),
        Or<(With<Position>, With<PixelPosition>)>,
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
