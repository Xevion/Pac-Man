use crate::error::{GameError, TextureError};
use crate::map::builder::Map;
use crate::systems::components::{DeltaTime, DirectionalAnimated, Movable, MovementState, Position, Renderable};
use crate::texture::sprite::SpriteAtlas;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::EventWriter;
use bevy_ecs::system::{NonSendMut, Query, Res};
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;

/// Updates the directional animated texture of an entity.
///
/// This runs before the render system so it can update the sprite based on the current direction of travel, as well as whether the entity is moving.
pub fn directional_render_system(
    dt: Res<DeltaTime>,
    mut renderables: Query<(&MovementState, &Movable, &mut DirectionalAnimated, &mut Renderable)>,
    mut errors: EventWriter<GameError>,
) {
    for (movement_state, movable, mut texture, mut renderable) in renderables.iter_mut() {
        let stopped = matches!(movement_state, MovementState::Stopped);
        let current_direction = movable.current_direction;

        let texture = if stopped {
            texture.stopped_textures[current_direction.as_usize()].as_mut()
        } else {
            texture.textures[current_direction.as_usize()].as_mut()
        };

        if let Some(texture) = texture {
            if !stopped {
                texture.tick(dt.0);
            }
            renderable.sprite = *texture.current_tile();
        } else {
            errors.write(TextureError::RenderFailed(format!("Entity has no texture")).into());
            continue;
        }
    }
}

/// A non-send resource for the map texture. This just wraps the texture with a type so it can be differentiated when exposed as a resource.
pub struct MapTextureResource(pub Texture<'static>);

/// A non-send resource for the backbuffer texture. This just wraps the texture with a type so it can be differentiated when exposed as a resource.
pub struct BackbufferResource(pub Texture<'static>);

pub fn render_system(
    mut canvas: NonSendMut<&mut Canvas<Window>>,
    map_texture: NonSendMut<MapTextureResource>,
    mut backbuffer: NonSendMut<BackbufferResource>,
    mut atlas: NonSendMut<SpriteAtlas>,
    map: Res<Map>,
    renderables: Query<(Entity, &mut Renderable, &Position)>,
    mut errors: EventWriter<GameError>,
) {
    // Clear the main canvas first
    canvas.set_draw_color(sdl2::pixels::Color::BLACK);
    canvas.clear();

    // Render to backbuffer
    canvas
        .with_texture_canvas(&mut backbuffer.0, |backbuffer_canvas| {
            // Clear the backbuffer
            backbuffer_canvas.set_draw_color(sdl2::pixels::Color::BLACK);
            backbuffer_canvas.clear();

            // Copy the pre-rendered map texture to the backbuffer
            backbuffer_canvas
                .copy(&map_texture.0, None, None)
                .err()
                .map(|e| errors.write(TextureError::RenderFailed(e.to_string()).into()));

            // Render all entities to the backbuffer
            for (_, mut renderable, position) in renderables
            // .iter_mut()
            // .sort_by_key::<&mut Renderable, _, _>(|(renderable, renderable, renderable)| renderable.layer)
            // .collect()
            {
                let pos = position.get_pixel_pos(&map.graph);
                match pos {
                    Ok(pos) => {
                        let dest = crate::helpers::centered_with_size(
                            glam::IVec2::new(pos.x as i32, pos.y as i32),
                            glam::UVec2::new(renderable.sprite.size.x as u32, renderable.sprite.size.y as u32),
                        );

                        renderable
                            .sprite
                            .render(backbuffer_canvas, &mut atlas, dest)
                            .err()
                            .map(|e| errors.write(TextureError::RenderFailed(e.to_string()).into()));
                    }
                    Err(e) => {
                        errors.write(e.into());
                    }
                }
            }
        })
        .err()
        .map(|e| errors.write(TextureError::RenderFailed(e.to_string()).into()));

    // Copy backbuffer to main canvas and present
    canvas
        .copy(&backbuffer.0, None, None)
        .err()
        .map(|e| errors.write(TextureError::RenderFailed(e.to_string()).into()));

    canvas.present();
}
