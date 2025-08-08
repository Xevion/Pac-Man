use glam::{IVec2, UVec2};
use pacman::helpers::centered_with_size;

#[test]
fn test_centered_with_size_basic() {
    let rect = centered_with_size(IVec2::new(100, 100), UVec2::new(50, 30));
    assert_eq!(rect.origin(), (75, 85));
    assert_eq!(rect.size(), (50, 30));
}

#[test]
fn test_centered_with_size_odd_dimensions() {
    let rect = centered_with_size(IVec2::new(50, 50), UVec2::new(51, 31));
    assert_eq!(rect.origin(), (25, 35));
    assert_eq!(rect.size(), (51, 31));
}

#[test]
fn test_centered_with_size_zero_position() {
    let rect = centered_with_size(IVec2::new(0, 0), UVec2::new(100, 100));
    assert_eq!(rect.origin(), (-50, -50));
    assert_eq!(rect.size(), (100, 100));
}

#[test]
fn test_centered_with_size_negative_position() {
    let rect = centered_with_size(IVec2::new(-100, -50), UVec2::new(80, 40));
    assert_eq!(rect.origin(), (-140, -70));
    assert_eq!(rect.size(), (80, 40));
}

#[test]
fn test_centered_with_size_large_dimensions() {
    let rect = centered_with_size(IVec2::new(1000, 1000), UVec2::new(1000, 1000));
    assert_eq!(rect.origin(), (500, 500));
    assert_eq!(rect.size(), (1000, 1000));
}
