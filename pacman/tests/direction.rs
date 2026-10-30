use assert2::check;
use pacman::map::direction::*;
use rstest::rstest;

#[rstest]
#[case(Direction::Up, Direction::Down)]
#[case(Direction::Down, Direction::Up)]
#[case(Direction::Left, Direction::Right)]
#[case(Direction::Right, Direction::Left)]
fn test_direction_opposite(#[case] dir: Direction, #[case] expected: Direction) {
    check!(dir.opposite() == expected);
}

#[test]
fn test_direction_opposite_symmetry() {
    // Test that opposite() is symmetric: opposite(opposite(d)) == d
    for &dir in &Direction::DIRECTIONS {
        check!(dir.opposite().opposite() == dir);
    }
}

#[test]
fn test_direction_opposite_exhaustive() {
    // Test that every direction has a unique opposite
    let mut opposites = std::collections::HashSet::new();
    for &dir in &Direction::DIRECTIONS {
        let opposite = dir.opposite();
        check!(opposites.insert(opposite));
    }
    check!(opposites.len() == 4);
}

#[test]
fn test_direction_as_usize_exhaustive() {
    // Test that as_usize() returns unique values for all directions
    let mut usizes = std::collections::HashSet::new();
    for &dir in &Direction::DIRECTIONS {
        let usize_val = dir.as_usize();
        check!(usizes.insert(usize_val));
    }
    check!(usizes.len() == 4);
}

#[test]
fn test_direction_as_ivec2_exhaustive() {
    // Test that as_ivec2() returns unique values for all directions
    let mut ivec2s = std::collections::HashSet::new();
    for &dir in &Direction::DIRECTIONS {
        let ivec2_val = dir.as_ivec2();
        check!(ivec2s.insert(ivec2_val));
    }
    check!(ivec2s.len() == 4);
}
