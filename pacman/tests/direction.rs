use pacman::map::direction::*;
use speculoos::prelude::*;

#[test]
fn test_direction_opposite() {
    let test_cases = [
        (Direction::Up, Direction::Down),
        (Direction::Down, Direction::Up),
        (Direction::Left, Direction::Right),
        (Direction::Right, Direction::Left),
    ];

    for (dir, expected) in test_cases {
        assert_that(&dir.opposite()).is_equal_to(expected);
    }
}

#[test]
fn test_direction_opposite_symmetry() {
    // Test that opposite() is symmetric: opposite(opposite(d)) == d
    for &dir in &Direction::DIRECTIONS {
        assert_that(&dir.opposite().opposite()).is_equal_to(dir);
    }
}

#[test]
fn test_direction_opposite_exhaustive() {
    // Test that every direction has a unique opposite
    let mut opposites = std::collections::HashSet::new();
    for &dir in &Direction::DIRECTIONS {
        let opposite = dir.opposite();
        assert_that(&opposites.insert(opposite)).is_true();
    }
    assert_that(&opposites).has_length(4);
}

#[test]
fn test_direction_as_usize_exhaustive() {
    // Test that as_usize() returns unique values for all directions
    let mut usizes = std::collections::HashSet::new();
    for &dir in &Direction::DIRECTIONS {
        let usize_val = dir.as_usize();
        assert_that(&usizes.insert(usize_val)).is_true();
    }
    assert_that(&usizes).has_length(4);
}

#[test]
fn test_direction_as_ivec2_exhaustive() {
    // Test that as_ivec2() returns unique values for all directions
    let mut ivec2s = std::collections::HashSet::new();
    for &dir in &Direction::DIRECTIONS {
        let ivec2_val = dir.as_ivec2();
        assert_that(&ivec2s.insert(ivec2_val)).is_true();
    }
    assert_that(&ivec2s).has_length(4);
}
