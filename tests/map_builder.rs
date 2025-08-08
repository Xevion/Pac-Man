use glam::{IVec2, Vec2};
use pacman::constants::{BOARD_CELL_SIZE, CELL_SIZE};
use pacman::map::Map;

fn create_minimal_test_board() -> [&'static str; BOARD_CELL_SIZE.y as usize] {
    let mut board = [""; BOARD_CELL_SIZE.y as usize];
    // Create a minimal valid board with house doors
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
fn test_find_starting_position_pacman() {
    let board = create_minimal_test_board();
    let map = Map::new(board);

    let pacman_pos = map.find_starting_position(0);
    assert!(pacman_pos.is_some());

    let pos = pacman_pos.unwrap();
    // Pacman should be found somewhere in the board
    assert!(pos.x < BOARD_CELL_SIZE.x);
    assert!(pos.y < BOARD_CELL_SIZE.y);
}

#[test]
fn test_find_starting_position_nonexistent() {
    let board = create_minimal_test_board();
    let map = Map::new(board);

    let pos = map.find_starting_position(99); // Non-existent entity
    assert!(pos.is_none());
}

#[test]
fn test_map_graph_construction() {
    let board = create_minimal_test_board();
    let map = Map::new(board);

    // Check that nodes were created
    assert!(map.graph.node_count() > 0);

    // Check that grid_to_node mapping was created
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
fn test_map_grid_to_node_mapping() {
    let board = create_minimal_test_board();
    let map = Map::new(board);

    // Check that Pac-Man's position is mapped
    let pacman_pos = map.find_starting_position(0).unwrap();
    let grid_pos = IVec2::new(pacman_pos.x as i32, pacman_pos.y as i32);

    assert!(map.grid_to_node.contains_key(&grid_pos));
    let node_id = map.grid_to_node[&grid_pos];
    assert!(map.graph.get_node(node_id).is_some());
}

#[test]
fn test_map_node_positions() {
    let board = create_minimal_test_board();
    let map = Map::new(board);

    // Check that node positions are correctly calculated
    for (grid_pos, &node_id) in &map.grid_to_node {
        let node = map.graph.get_node(node_id).unwrap();
        let expected_pos = Vec2::new((grid_pos.x * CELL_SIZE as i32) as f32, (grid_pos.y * CELL_SIZE as i32) as f32)
            + Vec2::splat(CELL_SIZE as f32 / 2.0);

        assert_eq!(node.position, expected_pos);
    }
}
