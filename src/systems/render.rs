use crate::constants::CANVAS_SIZE;
use crate::error::{GameError, TextureError};
use crate::map::builder::Map;
use crate::systems::{
    DebugState, DebugTextureResource, DeltaTime, DirectionalAnimated, Position, Renderable, ScoreResource, StartupSequence,
    Velocity,
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

#[derive(Resource, Default)]
pub struct RenderDirty(pub bool);

#[derive(Component)]
pub struct Hidden;

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

/// Updates the directional animated texture of an entity.
///
/// This runs before the render system so it can update the sprite based on the current direction of travel, as well as whether the entity is moving.
pub fn directional_render_system(
    dt: Res<DeltaTime>,
    mut renderables: Query<(&Position, &Velocity, &mut DirectionalAnimated, &mut Renderable)>,
    mut errors: EventWriter<GameError>,
) {
    for (position, velocity, mut texture, mut renderable) in renderables.iter_mut() {
        let stopped = matches!(position, Position::Stopped { .. });
        let current_direction = velocity.direction;

        let texture = if stopped {
            texture.stopped_textures[current_direction.as_usize()].as_mut()
        } else {
            texture.textures[current_direction.as_usize()].as_mut()
        };

        if let Some(texture) = texture {
            if !stopped {
                texture.tick(dt.0);
            }
            let new_tile = *texture.current_tile();
            if renderable.sprite != new_tile {
                renderable.sprite = new_tile;
            }
        } else {
            errors.write(TextureError::RenderFailed("Entity has no texture".to_string()).into());
            continue;
        }
    }
}

/// A non-send resource for the map texture. This just wraps the texture with a type so it can be differentiated when exposed as a resource.
pub struct MapTextureResource(pub Texture<'static>);

/// A non-send resource for the backbuffer texture. This just wraps the texture with a type so it can be differentiated when exposed as a resource.
pub struct BackbufferResource(pub Texture<'static>);

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

        // Render score text in yellow (Pac-Man's color)
        let score_text = format!("{:02}", score.0);
        let score_offset = 7 - (score_text.len() as i32);
        let score_position = glam::UVec2::new(4 + 8 * score_offset as u32, 10); // x_offset + score_offset * 8, 8 + y_offset

        if let Err(e) = text_renderer.render(canvas, &mut atlas, &score_text, score_position) {
            errors.write(TextureError::RenderFailed(format!("Failed to render score text: {}", e)).into());
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
    mut canvas: NonSendMut<&mut Canvas<Window>>,
    map_texture: NonSendMut<MapTextureResource>,
    mut backbuffer: NonSendMut<BackbufferResource>,
    mut atlas: NonSendMut<SpriteAtlas>,
    map: Res<Map>,
    dirty: Res<RenderDirty>,
    renderables: Query<(Entity, &Renderable, &Position), Without<Hidden>>,
    mut errors: EventWriter<GameError>,
) {
    if !dirty.0 {
        return;
    }
    // Render to backbuffer
    canvas
        .with_texture_canvas(&mut backbuffer.0, |backbuffer_canvas| {
            // Clear the backbuffer
            backbuffer_canvas.set_draw_color(sdl2::pixels::Color::BLACK);
            backbuffer_canvas.clear();

            // Copy the pre-rendered map texture to the backbuffer
            if let Err(e) = backbuffer_canvas.copy(&map_texture.0, None, None) {
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
                            .render(backbuffer_canvas, &mut atlas, dest)
                            .err()
                            .map(|e| errors.write(TextureError::RenderFailed(e.to_string()).into()));
                    }
                    Err(e) => {
                        errors.write(e);
                    }
                }
            }
        })
        .err()
        .map(|e| errors.write(TextureError::RenderFailed(e.to_string()).into()));

    canvas.copy(&backbuffer.0, None, None).unwrap();
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
