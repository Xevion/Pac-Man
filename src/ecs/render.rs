use crate::ecs::{render, Position, Renderable};
use crate::entity::graph::Graph;
use crate::error::{EntityError, GameError, TextureError};
use crate::map::builder::Map;
use crate::texture::sprite::{Sprite, SpriteAtlas};
use bevy_ecs::entity::Entity;
use bevy_ecs::event::EventWriter;
use bevy_ecs::query::With;
use bevy_ecs::system::{NonSendMut, Query, Res};
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;

pub struct MapTextureResource(pub Texture<'static>);
pub struct BackbufferResource(pub Texture<'static>);

pub fn render_system(
    mut canvas: NonSendMut<&mut Canvas<Window>>,
    map_texture: NonSendMut<MapTextureResource>,
    mut backbuffer: NonSendMut<BackbufferResource>,
    mut atlas: NonSendMut<SpriteAtlas>,
    map: Res<Map>,
    renderables: Query<(Entity, &Renderable, &Position)>,
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
            for (_, renderable, position) in renderables.iter() {
                let pos = position.get_pixel_pos(&map.graph);
                match pos {
                    Ok(pos) => {
                        renderable
                            .sprite
                            .render(backbuffer_canvas, &mut atlas, pos)
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
