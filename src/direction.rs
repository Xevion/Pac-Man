#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn angle(&self) -> f64 {
        match self {
            Direction::Right => 0f64,
            Direction::Down => 90f64,
            Direction::Left => 180f64,
            Direction::Up => 270f64,
        }
    }

    pub fn offset(&self) -> (i32, i32) {
        match self {
            Direction::Right => (1, 0),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Up => (0, -1),
        }
    }
}