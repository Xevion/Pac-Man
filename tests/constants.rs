use pacman::constants::*;
use speculoos::prelude::*;

#[test]
fn test_raw_board_structure() {
    // Test board dimensions match expected size
    assert_that(&RAW_BOARD.len()).is_equal_to(BOARD_CELL_SIZE.y as usize);
    for row in RAW_BOARD.iter() {
        assert_that(&row.len()).is_equal_to(BOARD_CELL_SIZE.x as usize);
    }

    // Test boundaries are properly walled
    assert_that(&RAW_BOARD[0].chars().all(|c| c == '#')).is_true();
    assert_that(&RAW_BOARD[RAW_BOARD.len() - 1].chars().all(|c| c == '#')).is_true();
}

#[test]
fn test_raw_board_contains_required_elements() {
    // Test that essential game elements are present
    assert_that(&RAW_BOARD.iter().any(|row| row.contains('X'))).is_true();
    assert_that(&RAW_BOARD.iter().any(|row| row.contains("=="))).is_true();
    assert_that(&RAW_BOARD.iter().any(|row| row.chars().any(|c| c == 'T'))).is_true();
    assert_that(&RAW_BOARD.iter().any(|row| row.chars().any(|c| c == 'o'))).is_true();
}
