use glam::U16Vec2;
use pacman::texture::sprite::{AtlasMapper, AtlasTile, MapperFrame};
use sdl2::pixels::Color;
use speculoos::prelude::*;
use std::collections::HashMap;

mod common;

#[test]
fn test_atlas_mapper_frame_lookup() {
    let mut frames = HashMap::new();
    frames.insert(
        "test".to_string(),
        MapperFrame {
            pos: U16Vec2::new(10, 20),
            size: U16Vec2::new(32, 64),
        },
    );

    let mapper = AtlasMapper { frames };

    // Test direct frame lookup
    let frame = mapper.frames.get("test");
    assert_that(&frame.is_some()).is_true();
    let frame = frame.unwrap();
    assert_that(&frame.pos).is_equal_to(U16Vec2::new(10, 20));
    assert_that(&frame.size).is_equal_to(U16Vec2::new(32, 64));
}

#[test]
fn test_atlas_mapper_multiple_frames() {
    let mut frames = HashMap::new();
    frames.insert(
        "tile1".to_string(),
        MapperFrame {
            pos: U16Vec2::new(0, 0),
            size: U16Vec2::new(32, 32),
        },
    );
    frames.insert(
        "tile2".to_string(),
        MapperFrame {
            pos: U16Vec2::new(32, 0),
            size: U16Vec2::new(64, 64),
        },
    );

    let mapper = AtlasMapper { frames };

    assert_that(&mapper.frames.len()).is_equal_to(2);
    assert_that(&mapper.frames.contains_key("tile1")).is_true();
    assert_that(&mapper.frames.contains_key("tile2")).is_true();
    assert_that(&mapper.frames.contains_key("tile3")).is_false();
    assert_that(&mapper.frames.contains_key("nonexistent")).is_false();
}

#[test]
fn test_atlas_tile_new_and_with_color() {
    let pos = U16Vec2::new(10, 20);
    let size = U16Vec2::new(30, 40);
    let color = Color::RGB(100, 150, 200);

    let tile = AtlasTile::new(pos, size, None);
    assert_that(&tile.pos).is_equal_to(pos);
    assert_that(&tile.size).is_equal_to(size);
    assert_that(&tile.color).is_equal_to(None);

    let tile_with_color = tile.with_color(color);
    assert_that(&tile_with_color.color).is_equal_to(Some(color));
}
