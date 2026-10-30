use assert2::{assert, check};
use pacman::texture::{sprite::SpriteAtlas, text::TextTexture};

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
    check!(!text_texture.get_char_map().contains_key(&c));

    // Get the tile from the atlas, which caches the tile in the char_map
    let result = text_texture.get_tile(c, atlas);

    assert!(let Ok(tile) = result);
    check!(tile.is_some());

    // Check that the tile is now cached in the char_map
    check!(text_texture.get_char_map().contains_key(&c));
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
        if let Err(e) = text_texture.render_with_color(
            &mut canvas,
            &mut atlas,
            &string,
            glam::UVec2::new(0, 0),
            sdl2::pixels::Color::WHITE,
        ) {
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

        check!(width > 0);
        check!(height > 0);
    }

    Ok(())
}

#[test]
fn test_text_scale() -> Result<(), String> {
    let string = "ABCDEFG !-/\"";
    let base_width = (string.len() * 8) as u32;

    let text_texture = TextTexture::new(0.5);
    check!(text_texture.text_height() == 4);
    check!(text_texture.text_width("") == 0);
    check!(text_texture.text_width(string) == base_width / 2);

    let text_texture = TextTexture::new(2.0);
    check!(text_texture.text_height() == 16);
    check!(text_texture.text_width(string) == base_width * 2);
    check!(text_texture.text_width("") == 0);

    let text_texture = TextTexture::new(1.0);
    check!(text_texture.text_height() == 8);
    check!(text_texture.text_width(string) == base_width);
    check!(text_texture.text_width("") == 0);

    Ok(())
}
