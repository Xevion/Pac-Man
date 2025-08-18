use pacman::error::{
    AnimatedTextureError, AssetError, EntityError, GameError, GameResult, IntoGameError, MapError, OptionExt, ParseError,
    ResultExt, TextureError,
};
use std::io;

#[test]
fn test_game_error_from_asset_error() {
    let asset_error = AssetError::NotFound("test.png".to_string());
    let game_error: GameError = asset_error.into();
    assert!(matches!(game_error, GameError::Asset(_)));
}

#[test]
fn test_game_error_from_parse_error() {
    let parse_error = ParseError::UnknownCharacter('Z');
    let game_error: GameError = parse_error.into();
    assert!(matches!(game_error, GameError::MapParse(_)));
}

#[test]
fn test_game_error_from_map_error() {
    let map_error = MapError::NodeNotFound(42);
    let game_error: GameError = map_error.into();
    assert!(matches!(game_error, GameError::Map(_)));
}

#[test]
fn test_game_error_from_texture_error() {
    let texture_error = TextureError::LoadFailed("Failed to load".to_string());
    let game_error: GameError = texture_error.into();
    assert!(matches!(game_error, GameError::Texture(_)));
}

#[test]
fn test_game_error_from_entity_error() {
    let entity_error = EntityError::NodeNotFound(10);
    let game_error: GameError = entity_error.into();
    assert!(matches!(game_error, GameError::Entity(_)));
}

#[test]
fn test_game_error_from_io_error() {
    let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
    let game_error: GameError = io_error.into();
    assert!(matches!(game_error, GameError::Io(_)));
}

#[test]
fn test_texture_error_from_animated_error() {
    let animated_error = AnimatedTextureError::InvalidFrameDuration(-1.0);
    let texture_error: TextureError = animated_error.into();
    assert!(matches!(texture_error, TextureError::Animated(_)));
}

#[test]
fn test_asset_error_from_io_error() {
    let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "Permission denied");
    let asset_error: AssetError = io_error.into();
    assert!(matches!(asset_error, AssetError::Io(_)));
}

#[test]
fn test_parse_error_display() {
    let error = ParseError::UnknownCharacter('!');
    assert_eq!(error.to_string(), "Unknown character in board: !");

    let error = ParseError::InvalidHouseDoorCount(3);
    assert_eq!(error.to_string(), "House door must have exactly 2 positions, found 3");
}

#[test]
fn test_entity_error_display() {
    let error = EntityError::NodeNotFound(42);
    assert_eq!(error.to_string(), "Node not found in graph: 42");

    let error = EntityError::EdgeNotFound { from: 1, to: 2 };
    assert_eq!(error.to_string(), "Edge not found: from 1 to 2");
}

#[test]
fn test_animated_texture_error_display() {
    let error = AnimatedTextureError::InvalidFrameDuration(0.0);
    assert_eq!(error.to_string(), "Frame duration must be positive, got 0");
}

#[test]
fn test_into_game_error_trait() {
    let result: Result<i32, io::Error> = Err(io::Error::new(io::ErrorKind::Other, "test error"));
    let game_result: GameResult<i32> = result.into_game_error();

    assert!(game_result.is_err());
    if let Err(GameError::InvalidState(msg)) = game_result {
        assert!(msg.contains("test error"));
    } else {
        panic!("Expected InvalidState error");
    }
}

#[test]
fn test_into_game_error_trait_success() {
    let result: Result<i32, io::Error> = Ok(42);
    let game_result: GameResult<i32> = result.into_game_error();

    assert_eq!(game_result.unwrap(), 42);
}

#[test]
fn test_option_ext_some() {
    let option: Option<i32> = Some(42);
    let result: GameResult<i32> = option.ok_or_game_error(|| GameError::InvalidState("Not found".to_string()));

    assert_eq!(result.unwrap(), 42);
}

#[test]
fn test_option_ext_none() {
    let option: Option<i32> = None;
    let result: GameResult<i32> = option.ok_or_game_error(|| GameError::InvalidState("Not found".to_string()));

    assert!(result.is_err());
    if let Err(GameError::InvalidState(msg)) = result {
        assert_eq!(msg, "Not found");
    } else {
        panic!("Expected InvalidState error");
    }
}

#[test]
fn test_result_ext_success() {
    let result: Result<i32, io::Error> = Ok(42);
    let game_result: GameResult<i32> = result.with_context(|_| GameError::InvalidState("Context".to_string()));

    assert_eq!(game_result.unwrap(), 42);
}

#[test]
fn test_result_ext_error() {
    let result: Result<i32, io::Error> = Err(io::Error::new(io::ErrorKind::Other, "original error"));
    let game_result: GameResult<i32> = result.with_context(|_| GameError::InvalidState("Context error".to_string()));

    assert!(game_result.is_err());
    if let Err(GameError::InvalidState(msg)) = game_result {
        assert_eq!(msg, "Context error");
    } else {
        panic!("Expected InvalidState error");
    }
}

#[test]
fn test_error_chain_conversions() {
    // Test that we can convert through multiple levels
    let animated_error = AnimatedTextureError::InvalidFrameDuration(-5.0);
    let texture_error: TextureError = animated_error.into();
    let game_error: GameError = texture_error.into();

    assert!(matches!(game_error, GameError::Texture(TextureError::Animated(_))));
}
