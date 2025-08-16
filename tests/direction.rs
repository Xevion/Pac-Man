use glam::IVec2;
use pacman::map::direction::*;

#[test]
fn test_direction_opposite() {
    let test_cases = [
        (Direction::Up, Direction::Down),
        (Direction::Down, Direction::Up),
        (Direction::Left, Direction::Right),
        (Direction::Right, Direction::Left),
    ];

    for (dir, expected) in test_cases {
        assert_eq!(dir.opposite(), expected);
    }
}

#[test]
fn test_direction_as_ivec2() {
    let test_cases = [
        (Direction::Up, -IVec2::Y),
        (Direction::Down, IVec2::Y),
        (Direction::Left, -IVec2::X),
        (Direction::Right, IVec2::X),
    ];

    for (dir, expected) in test_cases {
        assert_eq!(dir.as_ivec2(), expected);
        assert_eq!(IVec2::from(dir), expected);
    }
}
