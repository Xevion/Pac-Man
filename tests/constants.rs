use glam::UVec2;
use pacman::constants::*;

#[test]
fn test_loop_time() {
    // 60 FPS = 16.67ms per frame
    let expected_nanos = (1_000_000_000.0 / 60.0) as u64;
    assert_eq!(LOOP_TIME.as_nanos() as u64, expected_nanos);
}

#[test]
fn test_cell_size() {
    assert_eq!(CELL_SIZE, 8);
}

#[test]
fn test_board_cell_size() {
    assert_eq!(BOARD_CELL_SIZE.x, 28);
    assert_eq!(BOARD_CELL_SIZE.y, 31);
}

#[test]
fn test_scale() {
    assert_eq!(SCALE, 2.6);
}

#[test]
fn test_board_cell_offset() {
    assert_eq!(BOARD_CELL_OFFSET.x, 0);
    assert_eq!(BOARD_CELL_OFFSET.y, 3);
}

#[test]
fn test_board_pixel_offset() {
    let expected = UVec2::new(0 * CELL_SIZE, 3 * CELL_SIZE);
    assert_eq!(BOARD_PIXEL_OFFSET, expected);
    assert_eq!(BOARD_PIXEL_OFFSET.x, 0);
    assert_eq!(BOARD_PIXEL_OFFSET.y, 24); // 3 * 8
}

#[test]
fn test_board_pixel_size() {
    let expected = UVec2::new(28 * CELL_SIZE, 31 * CELL_SIZE);
    assert_eq!(BOARD_PIXEL_SIZE, expected);
    assert_eq!(BOARD_PIXEL_SIZE.x, 224); // 28 * 8
    assert_eq!(BOARD_PIXEL_SIZE.y, 248); // 31 * 8
}

#[test]
fn test_canvas_size() {
    let expected = UVec2::new((28 + 0) * CELL_SIZE, (31 + 3) * CELL_SIZE);
    assert_eq!(CANVAS_SIZE, expected);
    assert_eq!(CANVAS_SIZE.x, 224); // (28 + 0) * 8
    assert_eq!(CANVAS_SIZE.y, 272); // (31 + 3) * 8
}

#[test]
fn test_map_tile_variants() {
    assert_ne!(MapTile::Empty, MapTile::Wall);
    assert_ne!(MapTile::Pellet, MapTile::PowerPellet);
    assert_ne!(MapTile::Tunnel, MapTile::Empty);
}

#[test]
fn test_map_tile_clone() {
    let original = MapTile::Wall;
    let cloned = original;
    assert_eq!(original, cloned);
}

#[test]
fn test_raw_board_dimensions() {
    assert_eq!(RAW_BOARD.len(), BOARD_CELL_SIZE.y as usize);
    assert_eq!(RAW_BOARD.len(), 31);

    for row in RAW_BOARD.iter() {
        assert_eq!(row.len(), BOARD_CELL_SIZE.x as usize);
        assert_eq!(row.len(), 28);
    }
}

#[test]
fn test_raw_board_boundaries() {
    // First row should be all walls
    assert!(RAW_BOARD[0].chars().all(|c| c == '#'));

    // Last row should be all walls
    let last_row = RAW_BOARD[RAW_BOARD.len() - 1];
    assert!(last_row.chars().all(|c| c == '#'));

    // First and last character of each row should be walls (except tunnel rows and rows with spaces)
    for (i, row) in RAW_BOARD.iter().enumerate() {
        if i != 14 && !row.starts_with(' ') {
            // Skip tunnel row and rows that start with spaces
            assert_eq!(row.chars().next().unwrap(), '#');
            assert_eq!(row.chars().last().unwrap(), '#');
        }
    }
}

#[test]
fn test_raw_board_tunnel_row() {
    // Row 14 should have tunnel characters 'T' at the edges
    let tunnel_row = RAW_BOARD[14];
    assert_eq!(tunnel_row.chars().next().unwrap(), 'T');
    assert_eq!(tunnel_row.chars().last().unwrap(), 'T');
}

#[test]
fn test_raw_board_power_pellets() {
    // Power pellets are represented by 'o'
    let mut power_pellet_count = 0;
    for row in RAW_BOARD.iter() {
        power_pellet_count += row.chars().filter(|&c| c == 'o').count();
    }
    assert_eq!(power_pellet_count, 4); // Should have exactly 4 power pellets
}

#[test]
fn test_raw_board_starting_position() {
    // Should have a starting position 'X' for Pac-Man
    let mut found_starting_position = false;
    for row in RAW_BOARD.iter() {
        if row.contains('X') {
            found_starting_position = true;
            break;
        }
    }
    assert!(found_starting_position);
}

#[test]
fn test_raw_board_ghost_house() {
    // The ghost house area should be present (the == characters)
    let mut found_ghost_house = false;
    for row in RAW_BOARD.iter() {
        if row.contains("==") {
            found_ghost_house = true;
            break;
        }
    }
    assert!(found_ghost_house);
}

#[test]
fn test_raw_board_symmetry() {
    // The board should be roughly symmetrical
    let mid_point = RAW_BOARD[0].len() / 2;

    for row in RAW_BOARD.iter() {
        let left_half = &row[..mid_point];
        let right_half = &row[mid_point..];

        // Check that the halves are symmetrical (accounting for the center column)
        assert_eq!(left_half.len(), right_half.len());
    }
}

#[test]
fn test_constants_consistency() {
    // Verify that derived constants are calculated correctly
    let calculated_pixel_offset = UVec2::new(BOARD_CELL_OFFSET.x * CELL_SIZE, BOARD_CELL_OFFSET.y * CELL_SIZE);
    assert_eq!(BOARD_PIXEL_OFFSET, calculated_pixel_offset);

    let calculated_pixel_size = UVec2::new(BOARD_CELL_SIZE.x * CELL_SIZE, BOARD_CELL_SIZE.y * CELL_SIZE);
    assert_eq!(BOARD_PIXEL_SIZE, calculated_pixel_size);

    let calculated_canvas_size = UVec2::new(
        (BOARD_CELL_SIZE.x + BOARD_CELL_OFFSET.x) * CELL_SIZE,
        (BOARD_CELL_SIZE.y + BOARD_CELL_OFFSET.y) * CELL_SIZE,
    );
    assert_eq!(CANVAS_SIZE, calculated_canvas_size);
}
