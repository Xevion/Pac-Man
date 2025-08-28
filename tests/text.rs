use pacman::texture::{sprite::SpriteAtlas, text::TextTexture};

use crate::common::create_atlas;

mod common;

/// Helper function to get all characters that should be in the atlas
fn get_all_chars() -> String {
    let mut chars = Vec::new();
    chars.extend('A'..='Z');
    chars.extend('0'..='9');
    chars.extend(['!', '-', '"', '/']);
    chars.into_iter().collect()
}

/// Helper function to check if a character is in the atlas and char_map
fn check_char(text_texture: &mut TextTexture, atlas: &mut SpriteAtlas, c: char) {
    // Check that the character is not in the char_map yet
    assert!(
        !text_texture.get_char_map().contains_key(&c),
        "Character {c} should not yet be in char_map"
    );

    // Get the tile from the atlas, which caches the tile in the char_map
    let tile = text_texture.get_tile(c, atlas);

    assert!(tile.is_ok(), "Failed to get tile for character {c}");
    assert!(tile.unwrap().is_some(), "Tile for character {c} not found in atlas");

    // Check that the tile is now cached in the char_map
    assert!(
        text_texture.get_char_map().contains_key(&c),
        "Tile for character {c} was not cached in char_map"
    );
}

#[test]
fn test_chars() -> Result<(), String> {
    let (mut canvas, ..) = common::setup_sdl().map_err(|e| e.to_string())?;
    let mut atlas = create_atlas(&mut canvas);
    let mut text_texture = TextTexture::default();

    get_all_chars()
        .chars()
        .for_each(|c| check_char(&mut text_texture, &mut atlas, c));

    Ok(())
}

#[test]
fn test_render() -> Result<(), String> {
    let (mut canvas, ..) = common::setup_sdl().map_err(|e| e.to_string())?;
    let mut atlas = create_atlas(&mut canvas);
    let mut text_texture = TextTexture::default();

    let test_strings = vec!["Hello, world!".to_string(), get_all_chars()];

    for string in test_strings {
        if let Err(e) = text_texture.render(&mut canvas, &mut atlas, &string, glam::UVec2::new(0, 0)) {
            return Err(e.to_string());
        }
    }

    Ok(())
}

#[test]
fn test_text_width() -> Result<(), String> {
    let text_texture = TextTexture::default();

    let test_strings = vec!["Hello, world!".to_string(), get_all_chars()];

    for string in test_strings {
        let width = text_texture.text_width(&string);
        let height = text_texture.text_height();

        assert!(width > 0, "Width for string {string} should be greater than 0");
        assert!(height > 0, "Height for string {string} should be greater than 0");
    }

    Ok(())
}

#[test]
fn test_text_scale() -> Result<(), String> {
    let string = "ABCDEFG !-/\"";
    let base_width = (string.len() * 8) as u32;

    let mut text_texture = TextTexture::new(0.5);

    assert_eq!(text_texture.scale(), 0.5);
    assert_eq!(text_texture.text_height(), 4);
    assert_eq!(text_texture.text_width(""), 0);
    assert_eq!(text_texture.text_width(string), base_width / 2);

    text_texture.set_scale(2.0);
    assert_eq!(text_texture.scale(), 2.0);
    assert_eq!(text_texture.text_height(), 16);
    assert_eq!(text_texture.text_width(string), base_width * 2);
    assert_eq!(text_texture.text_width(""), 0);

    text_texture.set_scale(1.0);
    assert_eq!(text_texture.scale(), 1.0);
    assert_eq!(text_texture.text_height(), 8);
    assert_eq!(text_texture.text_width(string), base_width);
    assert_eq!(text_texture.text_width(""), 0);

    Ok(())
}

#[test]
fn test_text_color() -> Result<(), String> {
    let mut text_texture = TextTexture::new(1.0);

    // Test default color (should be None initially)
    assert_eq!(text_texture.color(), None);

    // Test setting color
    let test_color = sdl2::pixels::Color::YELLOW;
    text_texture.set_color(test_color);
    assert_eq!(text_texture.color(), Some(test_color));

    // Test changing color
    let new_color = sdl2::pixels::Color::RED;
    text_texture.set_color(new_color);
    assert_eq!(text_texture.color(), Some(new_color));

    Ok(())
}
