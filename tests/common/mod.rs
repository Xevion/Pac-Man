#![allow(dead_code)]

use pacman::{
    asset::{get_asset_bytes, Asset},
    game::state::ATLAS_FRAMES,
    texture::sprite::{AtlasMapper, SpriteAtlas},
};
use sdl2::{
    image::LoadTexture,
    render::{Canvas, Texture, TextureCreator},
    video::{Window, WindowContext},
    Sdl,
};

pub fn setup_sdl() -> Result<(Canvas<Window>, TextureCreator<WindowContext>, Sdl), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window("test", 800, 600)
        .position_centered()
        .hidden()
        .build()
        .map_err(|e| e.to_string())?;
    let canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();
    Ok((canvas, texture_creator, sdl_context))
}

pub fn create_atlas(canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) -> SpriteAtlas {
    let texture_creator = canvas.texture_creator();
    let atlas_bytes = get_asset_bytes(Asset::Atlas).unwrap();

    let texture = texture_creator.load_texture_bytes(&atlas_bytes).unwrap();
    let texture: Texture<'static> = unsafe { std::mem::transmute(texture) };

    let atlas_mapper = AtlasMapper {
        frames: ATLAS_FRAMES.into_iter().map(|(k, v)| (k.to_string(), *v)).collect(),
    };

    SpriteAtlas::new(texture, atlas_mapper)
}
