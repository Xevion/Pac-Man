use lazy_static::lazy_static;

pub const BOARD_WIDTH: u32 = 28;
pub const BOARD_HEIGHT: u32 = 37; // Adjusted to fit map texture?
pub const CELL_SIZE: u32 = 24;

pub const WINDOW_WIDTH: u32 = CELL_SIZE * BOARD_WIDTH;
pub const WINDOW_HEIGHT: u32 = CELL_SIZE * BOARD_HEIGHT;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum MapTile {
    Empty,
    Wall,
    Pellet,
    PowerPellet,
    StartingPosition(u8),
}

pub const RAW_BOARD: [&str; BOARD_HEIGHT as usize] = [
    "                            ",
    "                            ",
    "                            ",
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
    "                            ",
    "                            ",
    "                            ",
];

lazy_static! {
    pub static ref BOARD: [[MapTile; BOARD_HEIGHT as usize]; BOARD_HEIGHT as usize] = {
        let mut board = [[MapTile::Empty; BOARD_HEIGHT as usize]; BOARD_HEIGHT as usize];

        for y in 0..BOARD_HEIGHT as usize {
            let line = RAW_BOARD[y];

            for x in 0..BOARD_WIDTH as usize {
                if x >= line.len() {
                    break;
                }
                
                let i = (y * (BOARD_WIDTH as usize) + x) as usize;
                let character = line
                    .chars()
                    .nth(x as usize)
                    .unwrap_or_else(|| panic!("Could not get character at {} = ({}, {})", i, x, y));

                let tile = match character {
                    '#' => MapTile::Wall,
                    '.' => MapTile::Pellet,
                    'o' => MapTile::PowerPellet,
                    ' ' => MapTile::Empty,
                    c @ '0' | c @ '1' | c @ '2' | c @ '3' | c @ '4' => {
                        MapTile::StartingPosition(c.to_digit(10).unwrap() as u8)
                    },
                    '=' => MapTile::Empty,
                    _ => panic!("Unknown character in board: {}", character),
                };

                board[x as usize][y as usize] = tile;
            }
        }

        board
    };
}
