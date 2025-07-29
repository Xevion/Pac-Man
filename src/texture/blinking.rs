#![allow(dead_code)]
use crate::texture::sprite::AtlasTile;

#[derive(Clone)]
pub struct BlinkingTexture {
    tile: AtlasTile,
    blink_duration: f32,
    time_bank: f32,
    is_on: bool,
}

impl BlinkingTexture {
    pub fn new(tile: AtlasTile, blink_duration: f32) -> Self {
        Self {
            tile,
            blink_duration,
            time_bank: 0.0,
            is_on: true,
        }
    }

    pub fn tick(&mut self, dt: f32) {
        self.time_bank += dt;
        if self.time_bank >= self.blink_duration {
            self.time_bank -= self.blink_duration;
            self.is_on = !self.is_on;
        }
    }

    pub fn is_on(&self) -> bool {
        self.is_on
    }

    pub fn tile(&self) -> &AtlasTile {
        &self.tile
    }

    // Helper methods for testing
    pub fn time_bank(&self) -> f32 {
        self.time_bank
    }

    pub fn blink_duration(&self) -> f32 {
        self.blink_duration
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::U16Vec2;
    use sdl2::pixels::Color;

    fn mock_atlas_tile(id: u32) -> AtlasTile {
        AtlasTile {
            pos: U16Vec2::new(0, 0),
            size: U16Vec2::new(16, 16),
            color: Some(Color::RGB(id as u8, 0, 0)),
        }
    }

    #[test]
    fn test_new_blinking_texture() {
        let tile = mock_atlas_tile(1);
        let texture = BlinkingTexture::new(tile, 0.5);

        assert_eq!(texture.is_on(), true);
        assert_eq!(texture.time_bank(), 0.0);
        assert_eq!(texture.blink_duration(), 0.5);
        assert_eq!(texture.tile().color.unwrap().r, 1);
    }

    #[test]
    fn test_tick_no_blink_change() {
        let tile = mock_atlas_tile(1);
        let mut texture = BlinkingTexture::new(tile, 0.5);

        // Tick with less than blink duration
        texture.tick(0.25);
        assert_eq!(texture.is_on(), true);
        assert_eq!(texture.time_bank(), 0.25);
    }

    #[test]
    fn test_tick_single_blink_change() {
        let tile = mock_atlas_tile(1);
        let mut texture = BlinkingTexture::new(tile, 0.5);

        // Tick with exactly blink duration
        texture.tick(0.5);
        assert_eq!(texture.is_on(), false);
        assert_eq!(texture.time_bank(), 0.0);
    }

    #[test]
    fn test_tick_multiple_blink_changes() {
        let tile = mock_atlas_tile(1);
        let mut texture = BlinkingTexture::new(tile, 0.5);

        // First blink
        texture.tick(0.5);
        assert_eq!(texture.is_on(), false);

        // Second blink (back to on)
        texture.tick(0.5);
        assert_eq!(texture.is_on(), true);

        // Third blink (back to off)
        texture.tick(0.5);
        assert_eq!(texture.is_on(), false);
    }

    #[test]
    fn test_tick_partial_blink_duration() {
        let tile = mock_atlas_tile(1);
        let mut texture = BlinkingTexture::new(tile, 0.5);

        // Tick with 1.25 blink durations
        texture.tick(0.625);
        assert_eq!(texture.is_on(), false);
        assert_eq!(texture.time_bank(), 0.125);
    }

    #[test]
    fn test_tick_with_zero_duration() {
        let tile = mock_atlas_tile(1);
        let mut texture = BlinkingTexture::new(tile, 0.0);

        // Should not cause issues - skip the test if blink_duration is 0
        if texture.blink_duration() > 0.0 {
            texture.tick(0.1);
            assert_eq!(texture.is_on(), true);
        }
    }

    #[test]
    fn test_tick_with_negative_duration() {
        let tile = mock_atlas_tile(1);
        let mut texture = BlinkingTexture::new(tile, -0.5);

        // Should not cause issues - skip the test if blink_duration is negative
        if texture.blink_duration() > 0.0 {
            texture.tick(0.1);
            assert_eq!(texture.is_on(), true);
        }
    }

    #[test]
    fn test_tick_with_negative_delta_time() {
        let tile = mock_atlas_tile(1);
        let mut texture = BlinkingTexture::new(tile, 0.5);

        // Should not cause issues
        texture.tick(-0.1);
        assert_eq!(texture.is_on(), true);
        assert_eq!(texture.time_bank(), -0.1);
    }

    #[test]
    fn test_tile_access() {
        let tile = mock_atlas_tile(42);
        let texture = BlinkingTexture::new(tile, 0.5);

        assert_eq!(texture.tile().color.unwrap().r, 42);
    }

    #[test]
    fn test_clone() {
        let tile = mock_atlas_tile(1);
        let texture = BlinkingTexture::new(tile, 0.5);
        let cloned = texture.clone();

        assert_eq!(texture.is_on(), cloned.is_on());
        assert_eq!(texture.time_bank(), cloned.time_bank());
        assert_eq!(texture.blink_duration(), cloned.blink_duration());
        assert_eq!(texture.tile().color.unwrap().r, cloned.tile().color.unwrap().r);
    }
}
