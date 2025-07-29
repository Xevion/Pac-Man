use glam::IVec2;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn opposite(&self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }

    pub fn as_ivec2(&self) -> IVec2 {
        (*self).into()
    }
}

impl From<Direction> for IVec2 {
    fn from(dir: Direction) -> Self {
        match dir {
            Direction::Up => -IVec2::Y,
            Direction::Down => IVec2::Y,
            Direction::Left => -IVec2::X,
            Direction::Right => IVec2::X,
        }
    }
}

pub const DIRECTIONS: [Direction; 4] = [Direction::Up, Direction::Down, Direction::Left, Direction::Right];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction_opposite() {
        assert_eq!(Direction::Up.opposite(), Direction::Down);
        assert_eq!(Direction::Down.opposite(), Direction::Up);
        assert_eq!(Direction::Left.opposite(), Direction::Right);
        assert_eq!(Direction::Right.opposite(), Direction::Left);
    }

    #[test]
    fn test_direction_as_ivec2() {
        assert_eq!(Direction::Up.as_ivec2(), -IVec2::Y);
        assert_eq!(Direction::Down.as_ivec2(), IVec2::Y);
        assert_eq!(Direction::Left.as_ivec2(), -IVec2::X);
        assert_eq!(Direction::Right.as_ivec2(), IVec2::X);
    }

    #[test]
    fn test_direction_from_ivec2() {
        assert_eq!(IVec2::from(Direction::Up), -IVec2::Y);
        assert_eq!(IVec2::from(Direction::Down), IVec2::Y);
        assert_eq!(IVec2::from(Direction::Left), -IVec2::X);
        assert_eq!(IVec2::from(Direction::Right), IVec2::X);
    }

    #[test]
    fn test_directions_constant() {
        assert_eq!(DIRECTIONS.len(), 4);
        assert!(DIRECTIONS.contains(&Direction::Up));
        assert!(DIRECTIONS.contains(&Direction::Down));
        assert!(DIRECTIONS.contains(&Direction::Left));
        assert!(DIRECTIONS.contains(&Direction::Right));
    }

    #[test]
    fn test_direction_equality() {
        assert_eq!(Direction::Up, Direction::Up);
        assert_ne!(Direction::Up, Direction::Down);
        assert_ne!(Direction::Left, Direction::Right);
    }

    #[test]
    fn test_direction_clone() {
        let dir = Direction::Up;
        let cloned = dir;
        assert_eq!(dir, cloned);
    }

    #[test]
    fn test_direction_hash() {
        use std::collections::HashMap;
        let mut map = HashMap::new();
        map.insert(Direction::Up, "up");
        map.insert(Direction::Down, "down");

        assert_eq!(map.get(&Direction::Up), Some(&"up"));
        assert_eq!(map.get(&Direction::Down), Some(&"down"));
        assert_eq!(map.get(&Direction::Left), None);
    }
}
