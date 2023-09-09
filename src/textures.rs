
use sdl2::{
    image::LoadTexture,
    render::{Texture, TextureCreator},
    video::WindowContext,
};

pub struct TextureManager<'a> {
    pub map: Texture<'a>,
    pub pacman: Texture<'a>,
}

impl<'a> TextureManager<'a> {
    pub fn new(texture_creator: &'a TextureCreator<WindowContext>) -> Self {
        let map_texture = texture_creator
            .load_texture("assets/map.png")
            .expect("Could not load pacman texture");
        
        let pacman_atlas = texture_creator
            .load_texture("assets/pacman.png")
            .expect("Could not load pacman texture");
        
        
        TextureManager {
            map: map_texture,
            pacman: pacman_atlas,
        }
    }
}