use glam::Vec2;
use pacman::constants::{BOARD_CELL_SIZE, CELL_SIZE};
use pacman::map::Map;

fn create_minimal_test_board() -> [&'static str; BOARD_CELL_SIZE.y as usize] {
    let mut board = [""; BOARD_CELL_SIZE.y as usize];
    board[0] = "############################";
    board[1] = "#............##............#";
    board[2] = "#.####.#####.##.#####.####.#";
    board[3] = "#o####.#####.##.#####.####o#";
    board[4] = "#.####.#####.##.#####.####.#";
    board[5] = "#..........................#";
    board[6] = "#.####.##.########.##.####.#";
    board[7] = "#.####.##.########.##.####.#";
    board[8] = "#......##....##....##......#";
    board[9] = "######.##### ## #####.######";
    board[10] = "     #.##### ## #####.#     ";
    board[11] = "     #.##    ==    ##.#     ";
    board[12] = "     #.## ######## ##.#     ";
    board[13] = "######.## ######## ##.######";
    board[14] = "T     .   ########   .     T";
    board[15] = "######.## ######## ##.######";
    board[16] = "     #.## ######## ##.#     ";
    board[17] = "     #.##          ##.#     ";
    board[18] = "     #.## ######## ##.#     ";
    board[19] = "######.## ######## ##.######";
    board[20] = "#............##............#";
    board[21] = "#.####.#####.##.#####.####.#";
    board[22] = "#.####.#####.##.#####.####.#";
    board[23] = "#o..##.......X .......##..o#";
    board[24] = "###.##.##.########.##.##.###";
    board[25] = "###.##.##.########.##.##.###";
    board[26] = "#......##....##....##......#";
    board[27] = "#.##########.##.##########.#";
    board[28] = "#.##########.##.##########.#";
    board[29] = "#..........................#";
    board[30] = "############################";
    board
}

#[test]
fn test_map_creation() {
    let board = create_minimal_test_board();
    let map = Map::new(board).unwrap();

    assert!(map.graph.node_count() > 0);
    assert!(!map.grid_to_node.is_empty());

    // Check that some connections were made
    let mut has_connections = false;
    for intersection in &map.graph.adjacency_list {
        if intersection.edges().next().is_some() {
            has_connections = true;
            break;
        }
    }
    assert!(has_connections);
}

#[test]
fn test_map_starting_positions() {
    let board = create_minimal_test_board();
    let map = Map::new(board).unwrap();

    let pacman_pos = map.find_starting_position(0);
    assert!(pacman_pos.is_some());
    assert!(pacman_pos.unwrap().x < BOARD_CELL_SIZE.x);
    assert!(pacman_pos.unwrap().y < BOARD_CELL_SIZE.y);

    let nonexistent_pos = map.find_starting_position(99);
    assert_eq!(nonexistent_pos, None);
}

#[test]
fn test_map_node_positions() {
    let board = create_minimal_test_board();
    let map = Map::new(board).unwrap();

    for (grid_pos, &node_id) in &map.grid_to_node {
        let node = map.graph.get_node(node_id).unwrap();
        let expected_pos = Vec2::new((grid_pos.x * CELL_SIZE as i32) as f32, (grid_pos.y * CELL_SIZE as i32) as f32)
            + Vec2::splat(CELL_SIZE as f32 / 2.0);

        assert_eq!(node.position, expected_pos);
    }
}
