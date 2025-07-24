//! This module contains helper functions that are used throughout the game.

/// Checks if two grid positions are adjacent to each other
///
/// # Arguments
/// * `a` - First position as (x, y) coordinates
/// * `b` - Second position as (x, y) coordinates
/// * `diagonal` - Whether to consider diagonal adjacency (true) or only orthogonal (false)
///
/// # Returns
/// * `true` if positions are adjacent according to the diagonal parameter
/// * `false` otherwise
pub fn is_adjacent(a: (u32, u32), b: (u32, u32), diagonal: bool) -> bool {
    let (ax, ay) = a;
    let (bx, by) = b;
    let dx = ax.abs_diff(bx);
    let dy = ay.abs_diff(by);
    if diagonal {
        dx <= 1 && dy <= 1 && (dx != 0 || dy != 0)
    } else {
        (dx == 1 && dy == 0) || (dx == 0 && dy == 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orthogonal_adjacency() {
        // Test orthogonal adjacency (diagonal = false)

        // Same position should not be adjacent
        assert!(!is_adjacent((0, 0), (0, 0), false));

        // Adjacent positions should be true
        assert!(is_adjacent((0, 0), (1, 0), false)); // Right
        assert!(is_adjacent((0, 0), (0, 1), false)); // Down
        assert!(is_adjacent((1, 1), (0, 1), false)); // Left
        assert!(is_adjacent((1, 1), (1, 0), false)); // Up

        // Diagonal positions should be false
        assert!(!is_adjacent((0, 0), (1, 1), false));
        assert!(!is_adjacent((0, 1), (1, 0), false));

        // Positions more than 1 step away should be false
        assert!(!is_adjacent((0, 0), (2, 0), false));
        assert!(!is_adjacent((0, 0), (0, 2), false));
        assert!(!is_adjacent((0, 0), (2, 2), false));
    }

    #[test]
    fn test_diagonal_adjacency() {
        // Test diagonal adjacency (diagonal = true)

        // Same position should not be adjacent
        assert!(!is_adjacent((0, 0), (0, 0), true));

        // Orthogonal adjacent positions should be true
        assert!(is_adjacent((0, 0), (1, 0), true)); // Right
        assert!(is_adjacent((0, 0), (0, 1), true)); // Down
        assert!(is_adjacent((1, 1), (0, 1), true)); // Left
        assert!(is_adjacent((1, 1), (1, 0), true)); // Up

        // Diagonal adjacent positions should be true
        assert!(is_adjacent((0, 0), (1, 1), true)); // Down-right
        assert!(is_adjacent((1, 0), (0, 1), true)); // Down-left
        assert!(is_adjacent((0, 1), (1, 0), true)); // Up-right
        assert!(is_adjacent((1, 1), (0, 0), true)); // Up-left

        // Positions more than 1 step away should be false
        assert!(!is_adjacent((0, 0), (2, 0), true));
        assert!(!is_adjacent((0, 0), (0, 2), true));
        assert!(!is_adjacent((0, 0), (2, 2), true));
        assert!(!is_adjacent((0, 0), (1, 2), true));
    }

    #[test]
    fn test_edge_cases() {
        // Test with larger coordinates
        assert!(is_adjacent((100, 100), (101, 100), false));
        assert!(is_adjacent((100, 100), (100, 101), false));
        assert!(!is_adjacent((100, 100), (102, 100), false));

        assert!(is_adjacent((100, 100), (101, 101), true));
        assert!(!is_adjacent((100, 100), (102, 102), true));

        // Test with zero coordinates
        assert!(is_adjacent((0, 0), (1, 0), false));
        assert!(is_adjacent((0, 0), (0, 1), false));
        assert!(is_adjacent((0, 0), (1, 1), true));
    }

    #[test]
    fn test_commutative_property() {
        // The function should work the same regardless of parameter order
        assert_eq!(is_adjacent((1, 2), (2, 2), false), is_adjacent((2, 2), (1, 2), false));

        assert_eq!(is_adjacent((1, 2), (2, 3), true), is_adjacent((2, 3), (1, 2), true));
    }
}
