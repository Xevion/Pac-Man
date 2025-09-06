use pacman::texture::ttf::{TtfAtlas, TtfRenderer};
use sdl2::pixels::Color;

mod common;

#[test]
fn text_width_calculates_correctly_for_empty_string() {
    let (mut canvas, texture_creator, _sdl) = common::setup_sdl().unwrap();
    let _ttf_context = sdl2::ttf::init().unwrap();
    let font = _ttf_context.load_font("assets/game/TerminalVector.ttf", 16).unwrap();

    let mut atlas = TtfAtlas::new(&texture_creator, &font).unwrap();
    atlas.populate_atlas(&mut canvas, &texture_creator, &font).unwrap();

    let renderer = TtfRenderer::new(1.0);
    let width = renderer.text_width(&atlas, "");

    assert_eq!(width, 0);
}

#[test]
fn text_width_calculates_correctly_for_single_character() {
    let (mut canvas, texture_creator, _sdl) = common::setup_sdl().unwrap();
    let _ttf_context = sdl2::ttf::init().unwrap();
    let font = _ttf_context.load_font("assets/game/TerminalVector.ttf", 16).unwrap();

    let mut atlas = TtfAtlas::new(&texture_creator, &font).unwrap();
    atlas.populate_atlas(&mut canvas, &texture_creator, &font).unwrap();

    let renderer = TtfRenderer::new(1.0);
    let width = renderer.text_width(&atlas, "A");

    assert!(width > 0);
}

#[test]
fn text_width_scales_correctly() {
    let (mut canvas, texture_creator, _sdl) = common::setup_sdl().unwrap();
    let _ttf_context = sdl2::ttf::init().unwrap();
    let font = _ttf_context.load_font("assets/game/TerminalVector.ttf", 16).unwrap();

    let mut atlas = TtfAtlas::new(&texture_creator, &font).unwrap();
    atlas.populate_atlas(&mut canvas, &texture_creator, &font).unwrap();

    let renderer1 = TtfRenderer::new(1.0);
    let renderer2 = TtfRenderer::new(2.0);

    let width1 = renderer1.text_width(&atlas, "Test");
    let width2 = renderer2.text_width(&atlas, "Test");

    assert_eq!(width2, width1 * 2);
}

#[test]
fn text_height_returns_non_zero_for_valid_atlas() {
    let (mut canvas, texture_creator, _sdl) = common::setup_sdl().unwrap();
    let _ttf_context = sdl2::ttf::init().unwrap();
    let font = _ttf_context.load_font("assets/game/TerminalVector.ttf", 16).unwrap();

    let mut atlas = TtfAtlas::new(&texture_creator, &font).unwrap();
    atlas.populate_atlas(&mut canvas, &texture_creator, &font).unwrap();

    let renderer = TtfRenderer::new(1.0);
    let height = renderer.text_height(&atlas);

    assert!(height > 0);
}

#[test]
fn text_height_scales_correctly() {
    let (mut canvas, texture_creator, _sdl) = common::setup_sdl().unwrap();
    let _ttf_context = sdl2::ttf::init().unwrap();
    let font = _ttf_context.load_font("assets/game/TerminalVector.ttf", 16).unwrap();

    let mut atlas = TtfAtlas::new(&texture_creator, &font).unwrap();
    atlas.populate_atlas(&mut canvas, &texture_creator, &font).unwrap();

    let renderer1 = TtfRenderer::new(1.0);
    let renderer2 = TtfRenderer::new(2.0);

    let height1 = renderer1.text_height(&atlas);
    let height2 = renderer2.text_height(&atlas);

    assert_eq!(height2, height1 * 2);
}

#[test]
fn render_text_handles_empty_string() {
    let (mut canvas, texture_creator, _sdl) = common::setup_sdl().unwrap();
    let _ttf_context = sdl2::ttf::init().unwrap();
    let font = _ttf_context.load_font("assets/game/TerminalVector.ttf", 16).unwrap();

    let mut atlas = TtfAtlas::new(&texture_creator, &font).unwrap();
    atlas.populate_atlas(&mut canvas, &texture_creator, &font).unwrap();

    let renderer = TtfRenderer::new(1.0);
    let result = renderer.render_text(&mut canvas, &mut atlas, "", glam::Vec2::new(0.0, 0.0), Color::WHITE);

    assert!(result.is_ok());
}

#[test]
fn render_text_handles_single_character() {
    let (mut canvas, texture_creator, _sdl) = common::setup_sdl().unwrap();
    let _ttf_context = sdl2::ttf::init().unwrap();
    let font = _ttf_context.load_font("assets/game/TerminalVector.ttf", 16).unwrap();

    let mut atlas = TtfAtlas::new(&texture_creator, &font).unwrap();
    atlas.populate_atlas(&mut canvas, &texture_creator, &font).unwrap();

    let renderer = TtfRenderer::new(1.0);
    let result = renderer.render_text(&mut canvas, &mut atlas, "A", glam::Vec2::new(10.0, 10.0), Color::RED);

    assert!(result.is_ok());
}
