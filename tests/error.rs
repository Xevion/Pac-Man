use pacman::error::{GameError, GameResult, IntoGameError, OptionExt, ResultExt};
use speculoos::prelude::*;
use std::io;

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
