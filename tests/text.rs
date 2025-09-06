use pacman::texture::{sprite::SpriteAtlas, text::TextTexture};
use speculoos::prelude::*;

mod common;

use common::create_atlas;

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
    assert_that(&text_texture.get_char_map().contains_key(&c)).is_false();

    // Get the tile from the atlas, which caches the tile in the char_map
    let tile = text_texture.get_tile(c, atlas);

    assert_that(&tile.is_ok()).is_true();
    assert_that(&tile.unwrap().is_some()).is_true();

    // Check that the tile is now cached in the char_map
    assert_that(&text_texture.get_char_map().contains_key(&c)).is_true();
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

        assert_that(&(width > 0)).is_true();
        assert_that(&(height > 0)).is_true();
    }

    Ok(())
}

#[test]
fn test_text_scale() -> Result<(), String> {
    let string = "ABCDEFG !-/\"";
    let base_width = (string.len() * 8) as u32;

    let mut text_texture = TextTexture::new(0.5);

    assert_that(&text_texture.scale()).is_equal_to(0.5);
    assert_that(&text_texture.text_height()).is_equal_to(4);
    assert_that(&text_texture.text_width("")).is_equal_to(0);
    assert_that(&text_texture.text_width(string)).is_equal_to(base_width / 2);

    text_texture.set_scale(2.0);
    assert_that(&text_texture.scale()).is_equal_to(2.0);
    assert_that(&text_texture.text_height()).is_equal_to(16);
    assert_that(&text_texture.text_width(string)).is_equal_to(base_width * 2);
    assert_that(&text_texture.text_width("")).is_equal_to(0);

    text_texture.set_scale(1.0);
    assert_that(&text_texture.scale()).is_equal_to(1.0);
    assert_that(&text_texture.text_height()).is_equal_to(8);
    assert_that(&text_texture.text_width(string)).is_equal_to(base_width);
    assert_that(&text_texture.text_width("")).is_equal_to(0);

    Ok(())
}

#[test]
fn test_text_color() -> Result<(), String> {
    let mut text_texture = TextTexture::new(1.0);

    // Test default color (should be None initially)
    assert_that(&text_texture.color()).is_equal_to(None);

    // Test setting color
    let test_color = sdl2::pixels::Color::YELLOW;
    text_texture.set_color(test_color);
    assert_that(&text_texture.color()).is_equal_to(Some(test_color));

    // Test changing color
    let new_color = sdl2::pixels::Color::RED;
    text_texture.set_color(new_color);
    assert_that(&text_texture.color()).is_equal_to(Some(new_color));

    Ok(())
}
