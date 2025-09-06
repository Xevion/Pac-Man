use pacman::error::{
    AssetError, EntityError, GameError, GameResult, IntoGameError, MapError, OptionExt, ParseError, ResultExt, TextureError,
};
use speculoos::prelude::*;
use std::io;

#[test]
fn test_game_error_from_asset_error() {
    let asset_error = AssetError::NotFound("test.png".to_string());
    let game_error: GameError = asset_error.into();
    assert_that(&matches!(game_error, GameError::Asset(_))).is_true();
}

#[test]
fn test_game_error_from_parse_error() {
    let parse_error = ParseError::UnknownCharacter('Z');
    let game_error: GameError = parse_error.into();
    assert_that(&matches!(game_error, GameError::MapParse(_))).is_true();
}

#[test]
fn test_game_error_from_map_error() {
    let map_error = MapError::NodeNotFound(42);
    let game_error: GameError = map_error.into();
    assert_that(&matches!(game_error, GameError::Map(_))).is_true();
}

#[test]
fn test_game_error_from_texture_error() {
    let texture_error = TextureError::LoadFailed("Failed to load".to_string());
    let game_error: GameError = texture_error.into();
    assert_that(&matches!(game_error, GameError::Texture(_))).is_true();
}

#[test]
fn test_game_error_from_entity_error() {
    let entity_error = EntityError::NodeNotFound(10);
    let game_error: GameError = entity_error.into();
    assert_that(&matches!(game_error, GameError::Entity(_))).is_true();
}

#[test]
fn test_game_error_from_io_error() {
    let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
    let game_error: GameError = io_error.into();
    assert_that(&matches!(game_error, GameError::Io(_))).is_true();
}

#[test]
fn test_asset_error_from_io_error() {
    let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "Permission denied");
    let asset_error: AssetError = io_error.into();
    assert_that(&matches!(asset_error, AssetError::Io(_))).is_true();
}

#[test]
fn test_parse_error_display() {
    let error = ParseError::UnknownCharacter('!');
    assert_that(&error.to_string()).is_equal_to("Unknown character in board: !".to_string());

    let error = ParseError::InvalidHouseDoorCount(3);
    assert_that(&error.to_string()).is_equal_to("House door must have exactly 2 positions, found 3".to_string());
}

#[test]
fn test_entity_error_display() {
    let error = EntityError::NodeNotFound(42);
    assert_that(&error.to_string()).is_equal_to("Node not found in graph: 42".to_string());

    let error = EntityError::EdgeNotFound { from: 1, to: 2 };
    assert_that(&error.to_string()).is_equal_to("Edge not found: from 1 to 2".to_string());
}

#[test]
fn test_into_game_error_trait() {
    let result: Result<i32, io::Error> = Err(io::Error::new(io::ErrorKind::Other, "test error"));
    let game_result: GameResult<i32> = result.into_game_error();

    assert_that(&game_result.is_err()).is_true();
    if let Err(GameError::InvalidState(msg)) = game_result {
        assert_that(&msg.contains("test error")).is_true();
    } else {
        panic!("Expected InvalidState error");
    }
}

#[test]
fn test_into_game_error_trait_success() {
    let result: Result<i32, io::Error> = Ok(42);
    let game_result: GameResult<i32> = result.into_game_error();

    assert_that(&game_result.unwrap()).is_equal_to(42);
}

#[test]
fn test_option_ext_some() {
    let option: Option<i32> = Some(42);
    let result: GameResult<i32> = option.ok_or_game_error(|| GameError::InvalidState("Not found".to_string()));

    assert_that(&result.unwrap()).is_equal_to(42);
}

#[test]
fn test_option_ext_none() {
    let option: Option<i32> = None;
    let result: GameResult<i32> = option.ok_or_game_error(|| GameError::InvalidState("Not found".to_string()));

    assert_that(&result.is_err()).is_true();
    if let Err(GameError::InvalidState(msg)) = result {
        assert_that(&msg).is_equal_to("Not found".to_string());
    } else {
        panic!("Expected InvalidState error");
    }
}

#[test]
fn test_result_ext_success() {
    let result: Result<i32, io::Error> = Ok(42);
    let game_result: GameResult<i32> = result.with_context(|_| GameError::InvalidState("Context".to_string()));

    assert_that(&game_result.unwrap()).is_equal_to(42);
}

#[test]
fn test_result_ext_error() {
    let result: Result<i32, io::Error> = Err(io::Error::new(io::ErrorKind::Other, "original error"));
    let game_result: GameResult<i32> = result.with_context(|_| GameError::InvalidState("Context error".to_string()));

    assert_that(&game_result.is_err()).is_true();
    if let Err(GameError::InvalidState(msg)) = game_result {
        assert_that(&msg).is_equal_to("Context error".to_string());
    } else {
        panic!("Expected InvalidState error");
    }
}
