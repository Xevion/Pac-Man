use assert2::{assert, check};
use pacman::constants::{BOARD_CELL_SIZE, MapTile, RAW_BOARD};
use pacman::error::ParseError;
use pacman::map::parser::MapTileParser;
use rstest::rstest;

#[rstest]
#[case('#', MapTile::Wall)]
#[case('.', MapTile::Pellet)]
#[case('o', MapTile::PowerPellet)]
#[case(' ', MapTile::Empty)]
#[case('T', MapTile::Tunnel)]
#[case('X', MapTile::Empty)]
#[case('=', MapTile::Wall)]
fn test_parse_character(#[case] char: char, #[case] expected: MapTile) {
    assert!(let Ok(tile) = MapTileParser::parse_character(char));
    check!(tile == expected);
}

#[test]
fn test_parse_character_unknown() {
    check!(MapTileParser::parse_character('Z').is_err());
}

#[test]
fn test_parse_board() {
    let result = MapTileParser::parse_board(RAW_BOARD);
    assert!(let Ok(parsed) = result);
    check!(parsed.tiles.len() == BOARD_CELL_SIZE.x as usize);
    check!(parsed.tiles[0].len() == BOARD_CELL_SIZE.y as usize);
    check!(parsed.house_door[0].is_some());
    check!(parsed.house_door[1].is_some());
    check!(parsed.tunnel_ends[0].is_some());
    check!(parsed.tunnel_ends[1].is_some());
    check!(parsed.pacman_start.is_some());
}

#[test]
fn test_parse_board_invalid_character() {
    let mut invalid_board = RAW_BOARD.map(|s| s.to_string());
    invalid_board[0] = "###########################Z".to_string();

    let result = MapTileParser::parse_board(invalid_board.each_ref().map(|s| s.as_str()));
    assert!(let Err(ParseError::UnknownCharacter { character: 'Z', .. }) = result);
}
