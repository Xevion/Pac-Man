use pacman::constants::{BOARD_CELL_SIZE, RAW_BOARD};
use pacman::map::parser::{MapTileParser, ParseError};

#[test]
fn test_parse_character() {
    assert!(matches!(
        MapTileParser::parse_character('#').unwrap(),
        pacman::constants::MapTile::Wall
    ));
    assert!(matches!(
        MapTileParser::parse_character('.').unwrap(),
        pacman::constants::MapTile::Pellet
    ));
    assert!(matches!(
        MapTileParser::parse_character('o').unwrap(),
        pacman::constants::MapTile::PowerPellet
    ));
    assert!(matches!(
        MapTileParser::parse_character(' ').unwrap(),
        pacman::constants::MapTile::Empty
    ));
    assert!(matches!(
        MapTileParser::parse_character('T').unwrap(),
        pacman::constants::MapTile::Tunnel
    ));
    assert!(matches!(
        MapTileParser::parse_character('X').unwrap(),
        pacman::constants::MapTile::Empty
    ));
    assert!(matches!(
        MapTileParser::parse_character('=').unwrap(),
        pacman::constants::MapTile::Wall
    ));

    // Test invalid character
    assert!(MapTileParser::parse_character('Z').is_err());
}

#[test]
fn test_parse_board() {
    let result = MapTileParser::parse_board(RAW_BOARD);
    assert!(result.is_ok());

    let parsed = result.unwrap();

    // Verify we have tiles
    assert_eq!(parsed.tiles.len(), BOARD_CELL_SIZE.x as usize);
    assert_eq!(parsed.tiles[0].len(), BOARD_CELL_SIZE.y as usize);

    // Verify we found house door positions
    assert!(parsed.house_door[0].is_some());
    assert!(parsed.house_door[1].is_some());

    // Verify we found tunnel ends
    assert!(parsed.tunnel_ends[0].is_some());
    assert!(parsed.tunnel_ends[1].is_some());

    // Verify we found Pac-Man's starting position
    assert!(parsed.pacman_start.is_some());
}

#[test]
fn test_parse_board_invalid_character() {
    let mut invalid_board = RAW_BOARD.clone();
    invalid_board[0] = "###########################Z";

    let result = MapTileParser::parse_board(invalid_board);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ParseError::UnknownCharacter('Z')));
}
