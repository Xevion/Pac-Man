//! Rendering pipeline: a two-layer compositor plus a native-resolution overlay.
//!
//! The map and entities render into a maze-local backbuffer texture
//! (`PLAYFIELD_SIZE`, scale 1). Each frame runs `backbuffer_render_system`
//! (entities into the backbuffer) -> `hud_overlay_system` (gameplay text onto the
//! backbuffer) -> `composite_maze_system` (clear the window, blit the backbuffer at
//! integer scale into `Layout::maze`) -> `chrome_render_system` (window-space HUD
//! panels) -> `debug_overlay_system` (graph/timing annotations drawn straight onto
//! the window at native resolution) -> `present_system`. Every stage is gated by
//! `RenderDirty`, so an unchanged frame skips the GPU work entirely.

use crate::error::{GameError, TextureError};
use crate::map::builder::Map;
use crate::systems::input::{CursorPosition, TouchState};
use crate::systems::layout::Layout;
use crate::systems::movement::Position;
use crate::texture::sprite::{AtlasTile, SpriteAtlas};
use bevy_ecs::change_detection::DetectChanges;
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::EventWriter;
use bevy_ecs::query::{Changed, Or};
use bevy_ecs::removal_detection::RemovedComponents;
use bevy_ecs::resource::Resource;
use bevy_ecs::system::{NonSendMut, Query, Res, ResMut, SystemParam};
use sdl2::rect::{Point, Rect};
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;

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

#[allow(clippy::type_complexity)]
pub fn dirty_render_system(
    mut dirty: ResMut<RenderDirty>,
    changed: Query<(), Or<(Changed<Renderable>, Changed<Position>, Changed<Visibility>)>>,
    removed_renderables: RemovedComponents<Renderable>,
    cursor: Res<CursorPosition>,
    touch_state: Res<TouchState>,
    layout: Res<Layout>,
) {
    if changed.iter().count() > 0
        || !removed_renderables.is_empty()
        || cursor.is_changed()
        || touch_state.is_changed()
        || layout.is_changed()
    {
        dirty.0 = true;
    }
}

/// A non-send resource for the map texture. This just wraps the texture with a type so it can be differentiated when exposed as a resource.
pub struct MapTextureResource(pub Texture);

/// A non-send resource for the backbuffer texture. This just wraps the texture with a type so it can be differentiated when exposed as a resource.
pub struct BackbufferResource(pub Texture);

/// Owned wrapper for the SDL2 canvas, stored as a non-send ECS resource.
pub struct CanvasResource(pub Canvas<Window>);

impl std::ops::Deref for CanvasResource {
    type Target = Canvas<Window>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for CanvasResource {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[allow(clippy::type_complexity)]
pub fn render_system(
    canvas: &mut Canvas<Window>,
    map_texture: &NonSendMut<MapTextureResource>,
    atlas: &mut SpriteAtlas,
    map: &Res<Map>,
    dirty: &Res<RenderDirty>,
    renderables: &Query<(Entity, &Renderable, &Position, Option<&Visibility>)>,
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
        .filter(|(_, _, _, visibility)| visibility.copied().unwrap_or_default().is_visible())
        .collect();

    visible_entities.sort_by_key(|(_, renderable, _, _)| renderable.layer);
    visible_entities.reverse();

    // Render all visible entities to the backbuffer
    for (_entity, renderable, position, _visibility) in visible_entities {
        match position.get_pixel_position(&map.graph) {
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

/// Grouped NonSendMut render surface resources.
#[derive(SystemParam)]
pub struct RenderSurfaces<'w> {
    pub canvas: NonSendMut<'w, CanvasResource>,
    pub map_texture: NonSendMut<'w, MapTextureResource>,
    pub backbuffer: NonSendMut<'w, BackbufferResource>,
    pub atlas: NonSendMut<'w, SpriteAtlas>,
}

/// Renders the map and entities into the maze-local backbuffer texture. Timing is
/// recorded by the schedule's `profile` wrapper; the compositor and the
/// window-space debug overlay run as their own later stages.
pub fn backbuffer_render_system(
    mut surfaces: RenderSurfaces,
    map: Res<Map>,
    dirty: Res<RenderDirty>,
    renderables: Query<(Entity, &Renderable, &Position, Option<&Visibility>)>,
    mut errors: EventWriter<GameError>,
) {
    if !dirty.0 {
        return;
    }

    let _zone = tracing::debug_span!("backbuffer_texture").entered();
    let result = surfaces
        .canvas
        .with_texture_canvas(&mut surfaces.backbuffer.0, |texture_canvas| {
            render_system(
                texture_canvas,
                &surfaces.map_texture,
                &mut surfaces.atlas,
                &map,
                &dirty,
                &renderables,
                &mut errors,
            );
        });

    if let Err(e) = result {
        errors.write(TextureError::RenderFailed(e.to_string()).into());
    }
}

/// Composites the maze onto the window: clears the whole window to black (the
/// integer-scaled maze rarely fills it exactly, so the surplus reads as black
/// bands) and blits the maze backbuffer into the scaled maze rect. Window-space
/// chrome and the debug overlay draw on top of this in later stages.
pub fn composite_maze_system(
    mut canvas: NonSendMut<CanvasResource>,
    dirty: Res<RenderDirty>,
    backbuffer: NonSendMut<BackbufferResource>,
    layout: Res<Layout>,
    mut errors: EventWriter<GameError>,
) {
    if !dirty.0 {
        return;
    }

    let _zone = tracing::debug_span!("composite_maze").entered();
    canvas.set_draw_color(sdl2::pixels::Color::BLACK);
    canvas.clear();
    if let Err(e) = canvas.copy(&backbuffer.0, None, layout.maze) {
        errors.write(TextureError::RenderFailed(e).into());
    }
}

/// Presents the fully composited frame and clears the dirty flag.
pub fn present_system(mut canvas: NonSendMut<CanvasResource>, mut dirty: ResMut<RenderDirty>) {
    if dirty.0 {
        let _zone = tracing::debug_span!("sdl_present").entered();
        canvas.present();
        dirty.0 = false;
    }
}
