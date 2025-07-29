use anyhow::Result;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, RenderTarget};
use thiserror::Error;

use crate::texture::sprite::{AtlasTile, SpriteAtlas};

#[derive(Error, Debug)]
pub enum AnimatedTextureError {
    #[error("Frame duration must be positive, got {0}")]
    InvalidFrameDuration(f32),
}

#[derive(Debug, Clone)]
pub struct AnimatedTexture {
    tiles: Vec<AtlasTile>,
    frame_duration: f32,
    current_frame: usize,
    time_bank: f32,
}

impl AnimatedTexture {
    pub fn new(tiles: Vec<AtlasTile>, frame_duration: f32) -> Result<Self, AnimatedTextureError> {
        if frame_duration <= 0.0 {
            return Err(AnimatedTextureError::InvalidFrameDuration(frame_duration));
        }

        Ok(Self {
            tiles,
            frame_duration,
            current_frame: 0,
            time_bank: 0.0,
        })
    }

    pub fn tick(&mut self, dt: f32) {
        self.time_bank += dt;
        while self.time_bank >= self.frame_duration {
            self.time_bank -= self.frame_duration;
            self.current_frame = (self.current_frame + 1) % self.tiles.len();
        }
    }

    pub fn current_tile(&self) -> &AtlasTile {
        &self.tiles[self.current_frame]
    }

    pub fn render<T: RenderTarget>(&self, canvas: &mut Canvas<T>, atlas: &mut SpriteAtlas, dest: Rect) -> Result<()> {
        let mut tile = *self.current_tile();
        tile.render(canvas, atlas, dest)
    }

    /// Returns the current frame index.
    #[allow(dead_code)]
    pub fn current_frame(&self) -> usize {
        self.current_frame
    }

    /// Returns the time bank.
    #[allow(dead_code)]
    pub fn time_bank(&self) -> f32 {
        self.time_bank
    }

    /// Returns the frame duration.
    #[allow(dead_code)]
    pub fn frame_duration(&self) -> f32 {
        self.frame_duration
    }

    /// Returns the number of tiles in the animation.
    #[allow(dead_code)]
    pub fn tiles_len(&self) -> usize {
        self.tiles.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::U16Vec2;
    use sdl2::pixels::Color;

    impl AtlasTile {
        fn mock(id: u32) -> Self {
            AtlasTile {
                pos: U16Vec2::new(0, 0),
                size: U16Vec2::new(16, 16),
                color: Some(Color::RGB(id as u8, 0, 0)),
            }
        }
    }

    #[test]
    fn test_new_animated_texture() {
        let tiles = vec![AtlasTile::mock(1), AtlasTile::mock(2), AtlasTile::mock(3)];
        let texture = AnimatedTexture::new(tiles.clone(), 0.1).unwrap();

        assert_eq!(texture.current_frame(), 0);
        assert_eq!(texture.time_bank(), 0.0);
        assert_eq!(texture.frame_duration(), 0.1);
        assert_eq!(texture.tiles_len(), 3);
    }

    #[test]
    fn test_new_animated_texture_zero_duration() {
        let tiles = vec![AtlasTile::mock(1), AtlasTile::mock(2)];
        let result = AnimatedTexture::new(tiles, 0.0);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AnimatedTextureError::InvalidFrameDuration(0.0)));
    }

    #[test]
    fn test_new_animated_texture_negative_duration() {
        let tiles = vec![AtlasTile::mock(1), AtlasTile::mock(2)];
        let result = AnimatedTexture::new(tiles, -0.1);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AnimatedTextureError::InvalidFrameDuration(-0.1)
        ));
    }

    #[test]
    fn test_tick_no_frame_change() {
        let tiles = vec![AtlasTile::mock(1), AtlasTile::mock(2)];
        let mut texture = AnimatedTexture::new(tiles, 0.1).unwrap();

        // Tick with less than frame duration
        texture.tick(0.05);
        assert_eq!(texture.current_frame(), 0);
        assert_eq!(texture.time_bank(), 0.05);
    }

    #[test]
    fn test_tick_single_frame_change() {
        let tiles = vec![AtlasTile::mock(1), AtlasTile::mock(2)];
        let mut texture = AnimatedTexture::new(tiles, 0.1).unwrap();

        // Tick with exactly frame duration
        texture.tick(0.1);
        assert_eq!(texture.current_frame(), 1);
        assert_eq!(texture.time_bank(), 0.0);
    }

    #[test]
    fn test_tick_multiple_frame_changes() {
        let tiles = vec![AtlasTile::mock(1), AtlasTile::mock(2), AtlasTile::mock(3)];
        let mut texture = AnimatedTexture::new(tiles, 0.1).unwrap();

        // Tick with 2.5 frame durations
        texture.tick(0.25);
        assert_eq!(texture.current_frame(), 2);
        assert!((texture.time_bank() - 0.05).abs() < 0.001);
    }

    #[test]
    fn test_tick_wrap_around() {
        let tiles = vec![AtlasTile::mock(1), AtlasTile::mock(2)];
        let mut texture = AnimatedTexture::new(tiles, 0.1).unwrap();

        // Advance to last frame
        texture.tick(0.1);
        assert_eq!(texture.current_frame(), 1);

        // Advance again to wrap around
        texture.tick(0.1);
        assert_eq!(texture.current_frame(), 0);
    }

    #[test]
    fn test_current_tile() {
        let tiles = vec![AtlasTile::mock(1), AtlasTile::mock(2)];
        let texture = AnimatedTexture::new(tiles, 0.1).unwrap();

        // Should return first tile initially
        assert_eq!(texture.current_tile().color.unwrap().r, 1);
    }

    #[test]
    fn test_current_tile_after_frame_change() {
        let tiles = vec![AtlasTile::mock(1), AtlasTile::mock(2)];
        let mut texture = AnimatedTexture::new(tiles, 0.1).unwrap();

        // Advance one frame
        texture.tick(0.1);
        assert_eq!(texture.current_tile().color.unwrap().r, 2);
    }

    #[test]
    fn test_single_tile_animation() {
        let tiles = vec![AtlasTile::mock(1)];
        let mut texture = AnimatedTexture::new(tiles, 0.1).unwrap();

        // Should stay on same frame
        texture.tick(0.1);
        assert_eq!(texture.current_frame(), 0);
        assert_eq!(texture.current_tile().color.unwrap().r, 1);
    }
}
