use glam::U16Vec2;
use pacman::texture::animated::{AnimatedTexture, AnimatedTextureError};
use pacman::texture::sprite::AtlasTile;
use sdl2::pixels::Color;

fn mock_atlas_tile(id: u32) -> AtlasTile {
    AtlasTile {
        pos: U16Vec2::new(0, 0),
        size: U16Vec2::new(16, 16),
        color: Some(Color::RGB(id as u8, 0, 0)),
    }
}

#[test]
fn test_new_animated_texture_zero_duration() {
    let tiles = vec![mock_atlas_tile(1), mock_atlas_tile(2)];
    let result = AnimatedTexture::new(tiles, 0.0);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), AnimatedTextureError::InvalidFrameDuration(0.0)));
}

#[test]
fn test_new_animated_texture_negative_duration() {
    let tiles = vec![mock_atlas_tile(1), mock_atlas_tile(2)];
    let result = AnimatedTexture::new(tiles, -0.1);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        AnimatedTextureError::InvalidFrameDuration(-0.1)
    ));
}

#[test]
fn test_tick_multiple_frame_changes() {
    let tiles = vec![mock_atlas_tile(1), mock_atlas_tile(2), mock_atlas_tile(3)];
    let mut texture = AnimatedTexture::new(tiles, 0.1).unwrap();

    // Tick with 2.5 frame durations
    texture.tick(0.25);
    assert_eq!(texture.current_frame(), 2);
    assert!((texture.time_bank() - 0.05).abs() < 0.001);
}

#[test]
fn test_tick_wrap_around() {
    let tiles = vec![mock_atlas_tile(1), mock_atlas_tile(2)];
    let mut texture = AnimatedTexture::new(tiles, 0.1).unwrap();

    // Advance to last frame
    texture.tick(0.1);
    assert_eq!(texture.current_frame(), 1);

    // Advance again to wrap around
    texture.tick(0.1);
    assert_eq!(texture.current_frame(), 0);
}

#[test]
fn test_single_tile_animation() {
    let tiles = vec![mock_atlas_tile(1)];
    let mut texture = AnimatedTexture::new(tiles, 0.1).unwrap();

    // Should stay on same frame
    texture.tick(0.1);
    assert_eq!(texture.current_frame(), 0);
    assert_eq!(texture.current_tile().color.unwrap().r, 1);
}
