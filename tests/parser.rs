use pacman::constants::{BOARD_CELL_SIZE, RAW_BOARD};
use pacman::map::parser::{MapTileParser, ParseError};

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
        assert!(matches!(MapTileParser::parse_character(char).unwrap(), _expected));
    }

    assert!(MapTileParser::parse_character('Z').is_err());
}

#[test]
fn test_parse_board() {
    let result = MapTileParser::parse_board(RAW_BOARD);
    assert!(result.is_ok());

    let parsed = result.unwrap();
    assert_eq!(parsed.tiles.len(), BOARD_CELL_SIZE.x as usize);
    assert_eq!(parsed.tiles[0].len(), BOARD_CELL_SIZE.y as usize);
    assert!(parsed.house_door[0].is_some());
    assert!(parsed.house_door[1].is_some());
    assert!(parsed.tunnel_ends[0].is_some());
    assert!(parsed.tunnel_ends[1].is_some());
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
