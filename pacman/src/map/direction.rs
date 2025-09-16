use glam::I8Vec2;
use strum_macros::AsRefStr;

/// The four cardinal directions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, AsRefStr)]
#[repr(usize)]
#[strum(serialize_all = "lowercase")]
pub enum Direction {
    Up,
    Down,
    Left,
    #[default]
    Right,
}

impl Direction {
    /// The four cardinal directions.
    /// This is just a convenience constant for iterating over the directions.
    pub const DIRECTIONS: [Direction; 4] = [Direction::Up, Direction::Down, Direction::Left, Direction::Right];

    /// Returns the opposite direction. Constant time.
    pub const fn opposite(self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }

    /// Returns the direction as an I8Vec2.
    pub fn as_ivec2(self) -> I8Vec2 {
        self.into()
    }

    /// Returns the direction as a usize (0-3). Constant time.
    /// This is useful for indexing into arrays.
    pub const fn as_usize(self) -> usize {
        match self {
            Direction::Up => 0,
            Direction::Down => 1,
            Direction::Left => 2,
            Direction::Right => 3,
        }
    }
}

impl From<Direction> for I8Vec2 {
    fn from(dir: Direction) -> Self {
        match dir {
            Direction::Up => -I8Vec2::Y,
            Direction::Down => I8Vec2::Y,
            Direction::Left => -I8Vec2::X,
            Direction::Right => I8Vec2::X,
        }
    }
}
