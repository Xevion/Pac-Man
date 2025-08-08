use pacman::constants::*;

#[test]
fn test_raw_board_structure() {
    assert_eq!(RAW_BOARD.len(), BOARD_CELL_SIZE.y as usize);

    for row in RAW_BOARD.iter() {
        assert_eq!(row.len(), BOARD_CELL_SIZE.x as usize);
    }

    // Test boundaries
    assert!(RAW_BOARD[0].chars().all(|c| c == '#'));
    assert!(RAW_BOARD[RAW_BOARD.len() - 1].chars().all(|c| c == '#'));

    // Test tunnel row
    let tunnel_row = RAW_BOARD[14];
    assert_eq!(tunnel_row.chars().next().unwrap(), 'T');
    assert_eq!(tunnel_row.chars().last().unwrap(), 'T');
}

#[test]
fn test_raw_board_content() {
    let power_pellet_count = RAW_BOARD.iter().flat_map(|row| row.chars()).filter(|&c| c == 'o').count();
    assert_eq!(power_pellet_count, 4);

    assert!(RAW_BOARD.iter().any(|row| row.contains('X')));
    assert!(RAW_BOARD.iter().any(|row| row.contains("==")));
}
