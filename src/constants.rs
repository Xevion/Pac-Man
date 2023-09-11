pub const BOARD_WIDTH: u32 = 28;
pub const BOARD_HEIGHT: u32 = 31; // Adjusted to fit map texture?
pub const CELL_SIZE: u32 = 24;

pub const BOARD_OFFSET: (u32, u32) = (0, 3); // Relative cell offset for where map text / grid starts

pub const WINDOW_WIDTH: u32 = CELL_SIZE * BOARD_WIDTH;
pub const WINDOW_HEIGHT: u32 = CELL_SIZE * (BOARD_HEIGHT + 6); // Map texture is 6 cells taller (3 above, 3 below) than the grid

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum MapTile {
    Empty,
    Wall,
    Pellet,
    PowerPellet,
    StartingPosition(u8),
}

pub const RAW_BOARD: [&str; BOARD_HEIGHT as usize] = [
    "############################",
    "#............##............#",
    "#.####.#####.##.#####.####.#",
    "#o####.#####.##.#####.####o#",
    "#.####.#####.##.#####.####.#",
    "#..........................#",
    "#.####.##.########.##.####.#",
    "#.####.##.########.##.####.#",
    "#......##....##....##......#",
    "######.##### ## #####.######",
    "     #.##### ## #####.#     ",
    "     #.##    1     ##.#     ",
    "     #.## ###==### ##.#     ",
    "######.## #      # ##.######",
    "      .   #2 3 4 #   .      ",
    "######.## #      # ##.######",
    "     #.## ######## ##.#     ",
    "     #.##          ##.#     ",
    "     #.## ######## ##.#     ",
    "######.## ######## ##.######",
    "#............##............#",
    "#.####.#####.##.#####.####.#",
    "#.####.#####.##.#####.####.#",
    "#o..##.......0 .......##..o#",
    "###.##.##.########.##.##.###",
    "###.##.##.########.##.##.###",
    "#......##....##....##......#",
    "#.##########.##.##########.#",
    "#.##########.##.##########.#",
    "#..........................#",
    "############################",
];