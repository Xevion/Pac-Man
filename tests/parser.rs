use pacman::constants::{BOARD_CELL_SIZE, RAW_BOARD};
use pacman::error::ParseError;
use pacman::map::parser::MapTileParser;
use speculoos::prelude::*;

#[test]
fn test_parse_character() {
    let test_cases = [
        ('#', pacman::constants::MapTile::Wall),
        ('.', pacman::constants::MapTile::Pellet),
        ('o', pacman::constants::MapTile::PowerPellet),
        (' ', pacman::constants::MapTile::Empty),
        ('T', pacman::constants::MapTile::Tunnel),
        ('X', pacman::constants::MapTile::Empty),
        ('=', pacman::constants::MapTile::Wall),
    ];

    for (char, _expected) in test_cases {
        assert_that(&matches!(MapTileParser::parse_character(char).unwrap(), _expected)).is_true();
    }

    assert_that(&MapTileParser::parse_character('Z').is_err()).is_true();
}

#[test]
fn test_parse_board() {
    let result = MapTileParser::parse_board(RAW_BOARD);
    assert_that(&result.is_ok()).is_true();

    let parsed = result.unwrap();
    assert_that(&parsed.tiles.len()).is_equal_to(BOARD_CELL_SIZE.x as usize);
    assert_that(&parsed.tiles[0].len()).is_equal_to(BOARD_CELL_SIZE.y as usize);
    assert_that(&parsed.house_door[0].is_some()).is_true();
    assert_that(&parsed.house_door[1].is_some()).is_true();
    assert_that(&parsed.tunnel_ends[0].is_some()).is_true();
    assert_that(&parsed.tunnel_ends[1].is_some()).is_true();
    assert_that(&parsed.pacman_start.is_some()).is_true();
}

#[test]
fn test_parse_board_invalid_character() {
    let mut invalid_board = RAW_BOARD.map(|s| s.to_string());
    invalid_board[0] = "###########################Z".to_string();

    let result = MapTileParser::parse_board(invalid_board.each_ref().map(|s| s.as_str()));
    assert_that(&result.is_err()).is_true();
    assert_that(&matches!(result.unwrap_err(), ParseError::UnknownCharacter('Z'))).is_true();
}
