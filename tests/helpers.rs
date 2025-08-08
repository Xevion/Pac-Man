use glam::{IVec2, UVec2};
use pacman::helpers::centered_with_size;

#[test]
fn test_centered_with_size() {
    let test_cases = [
        ((100, 100), (50, 30), (75, 85)),
        ((50, 50), (51, 31), (25, 35)),
        ((0, 0), (100, 100), (-50, -50)),
        ((-100, -50), (80, 40), (-140, -70)),
        ((1000, 1000), (1000, 1000), (500, 500)),
    ];

    for ((pos_x, pos_y), (size_x, size_y), (expected_x, expected_y)) in test_cases {
        let rect = centered_with_size(IVec2::new(pos_x, pos_y), UVec2::new(size_x, size_y));
        assert_eq!(rect.origin(), (expected_x, expected_y));
        assert_eq!(rect.size(), (size_x, size_y));
    }
}
