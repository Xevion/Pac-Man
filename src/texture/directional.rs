use anyhow::Result;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, RenderTarget};
use std::collections::HashMap;

use crate::entity::direction::Direction;
use crate::texture::animated::AnimatedTexture;
use crate::texture::sprite::SpriteAtlas;

#[derive(Clone)]
pub struct DirectionalAnimatedTexture {
    textures: HashMap<Direction, AnimatedTexture>,
    stopped_textures: HashMap<Direction, AnimatedTexture>,
}

impl DirectionalAnimatedTexture {
    pub fn new(textures: HashMap<Direction, AnimatedTexture>, stopped_textures: HashMap<Direction, AnimatedTexture>) -> Self {
        Self {
            textures,
            stopped_textures,
        }
    }

    pub fn tick(&mut self, dt: f32) {
        for texture in self.textures.values_mut() {
            texture.tick(dt);
        }
    }

    pub fn render<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        atlas: &mut SpriteAtlas,
        dest: Rect,
        direction: Direction,
    ) -> Result<()> {
        if let Some(texture) = self.textures.get(&direction) {
            texture.render(canvas, atlas, dest)
        } else {
            Ok(())
        }
    }

    pub fn render_stopped<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        atlas: &mut SpriteAtlas,
        dest: Rect,
        direction: Direction,
    ) -> Result<()> {
        if let Some(texture) = self.stopped_textures.get(&direction) {
            texture.render(canvas, atlas, dest)
        } else {
            Ok(())
        }
    }

    // Helper methods for testing
    pub fn has_direction(&self, direction: Direction) -> bool {
        self.textures.contains_key(&direction)
    }

    pub fn has_stopped_direction(&self, direction: Direction) -> bool {
        self.stopped_textures.contains_key(&direction)
    }

    pub fn texture_count(&self) -> usize {
        self.textures.len()
    }

    pub fn stopped_texture_count(&self) -> usize {
        self.stopped_textures.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::texture::sprite::AtlasTile;
    use glam::U16Vec2;
    use sdl2::pixels::Color;

    fn mock_atlas_tile(id: u32) -> AtlasTile {
        AtlasTile {
            pos: U16Vec2::new(0, 0),
            size: U16Vec2::new(16, 16),
            color: Some(Color::RGB(id as u8, 0, 0)),
        }
    }

    fn mock_animated_texture(id: u32) -> AnimatedTexture {
        AnimatedTexture::new(vec![mock_atlas_tile(id)], 0.1).expect("Invalid frame duration")
    }

    #[test]
    fn test_new_directional_animated_texture() {
        let mut textures = HashMap::new();
        let mut stopped_textures = HashMap::new();

        textures.insert(Direction::Up, mock_animated_texture(1));
        textures.insert(Direction::Down, mock_animated_texture(2));
        stopped_textures.insert(Direction::Up, mock_animated_texture(3));
        stopped_textures.insert(Direction::Down, mock_animated_texture(4));

        let texture = DirectionalAnimatedTexture::new(textures, stopped_textures);

        assert_eq!(texture.texture_count(), 2);
        assert_eq!(texture.stopped_texture_count(), 2);
        assert!(texture.has_direction(Direction::Up));
        assert!(texture.has_direction(Direction::Down));
        assert!(!texture.has_direction(Direction::Left));
        assert!(texture.has_stopped_direction(Direction::Up));
        assert!(texture.has_stopped_direction(Direction::Down));
        assert!(!texture.has_stopped_direction(Direction::Left));
    }

    #[test]
    fn test_tick() {
        let mut textures = HashMap::new();
        textures.insert(Direction::Up, mock_animated_texture(1));
        textures.insert(Direction::Down, mock_animated_texture(2));

        let mut texture = DirectionalAnimatedTexture::new(textures, HashMap::new());

        // Should not panic
        texture.tick(0.1);
        assert_eq!(texture.texture_count(), 2);
    }

    #[test]
    fn test_empty_texture() {
        let texture = DirectionalAnimatedTexture::new(HashMap::new(), HashMap::new());

        assert_eq!(texture.texture_count(), 0);
        assert_eq!(texture.stopped_texture_count(), 0);
        assert!(!texture.has_direction(Direction::Up));
        assert!(!texture.has_stopped_direction(Direction::Up));
    }

    #[test]
    fn test_partial_directions() {
        let mut textures = HashMap::new();
        textures.insert(Direction::Up, mock_animated_texture(1));

        let texture = DirectionalAnimatedTexture::new(textures, HashMap::new());

        assert_eq!(texture.texture_count(), 1);
        assert!(texture.has_direction(Direction::Up));
        assert!(!texture.has_direction(Direction::Down));
        assert!(!texture.has_direction(Direction::Left));
        assert!(!texture.has_direction(Direction::Right));
    }

    #[test]
    fn test_clone() {
        let mut textures = HashMap::new();
        textures.insert(Direction::Up, mock_animated_texture(1));

        let texture = DirectionalAnimatedTexture::new(textures, HashMap::new());
        let cloned = texture.clone();

        assert_eq!(texture.texture_count(), cloned.texture_count());
        assert_eq!(texture.stopped_texture_count(), cloned.stopped_texture_count());
        assert_eq!(texture.has_direction(Direction::Up), cloned.has_direction(Direction::Up));
    }

    #[test]
    fn test_all_directions() {
        let mut textures = HashMap::new();
        textures.insert(Direction::Up, mock_animated_texture(1));
        textures.insert(Direction::Down, mock_animated_texture(2));
        textures.insert(Direction::Left, mock_animated_texture(3));
        textures.insert(Direction::Right, mock_animated_texture(4));

        let texture = DirectionalAnimatedTexture::new(textures, HashMap::new());

        assert_eq!(texture.texture_count(), 4);
        assert!(texture.has_direction(Direction::Up));
        assert!(texture.has_direction(Direction::Down));
        assert!(texture.has_direction(Direction::Left));
        assert!(texture.has_direction(Direction::Right));
    }
}
