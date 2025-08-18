use pacman::constants::*;

#[test]
fn test_raw_board_structure() {
    // Test board dimensions match expected size
    assert_eq!(RAW_BOARD.len(), BOARD_CELL_SIZE.y as usize);
    for row in RAW_BOARD.iter() {
        assert_eq!(row.len(), BOARD_CELL_SIZE.x as usize);
    }

    // Test boundaries are properly walled
    assert!(RAW_BOARD[0].chars().all(|c| c == '#'));
    assert!(RAW_BOARD[RAW_BOARD.len() - 1].chars().all(|c| c == '#'));
}

#[test]
fn test_raw_board_contains_required_elements() {
    // Test that essential game elements are present
    assert!(
        RAW_BOARD.iter().any(|row| row.contains('X')),
        "Board should contain Pac-Man start position"
    );
    assert!(
        RAW_BOARD.iter().any(|row| row.contains("==")),
        "Board should contain ghost house door"
    );
    assert!(
        RAW_BOARD.iter().any(|row| row.chars().any(|c| c == 'T')),
        "Board should contain tunnel entrances"
    );
    assert!(
        RAW_BOARD.iter().any(|row| row.chars().any(|c| c == 'o')),
        "Board should contain power pellets"
    );
}
