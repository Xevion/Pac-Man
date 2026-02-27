mod common;

use common::create_test_map;
use glam::Vec2;
use pacman::map::direction::Direction;
use pacman::systems::ghost::personality::{calculate_chase_target, TargetingContext};
use pacman::systems::ghost::GhostType;
use speculoos::prelude::*;

/// Helper to build a TargetingContext with known positions
fn make_ctx(
    pacman_node: u16,
    pacman_direction: Direction,
    pacman_position: Vec2,
    blinky_position: Vec2,
    self_node: u16,
    self_position: Vec2,
) -> TargetingContext {
    TargetingContext {
        pacman_node,
        pacman_direction,
        pacman_position,
        blinky_position,
        self_node,
        self_position,
    }
}
#[test]
fn blinky_targets_pacman_node() {
    let map = create_test_map();
    let pacman_node = 10u16;
    let ctx = make_ctx(
        pacman_node,
        Direction::Right,
        Vec2::new(100.0, 100.0),
        Vec2::ZERO,
        0,
        Vec2::ZERO,
    );
    let target = calculate_chase_target(GhostType::Blinky, &ctx, &map);
    assert_that(&target).is_equal_to(pacman_node);
}

#[test]
fn blinky_targets_pacman_node_regardless_of_direction() {
    let map = create_test_map();
    for dir in Direction::DIRECTIONS {
        let ctx = make_ctx(42, dir, Vec2::new(200.0, 200.0), Vec2::ZERO, 0, Vec2::ZERO);
        let target = calculate_chase_target(GhostType::Blinky, &ctx, &map);
        assert_that(&target).is_equal_to(42);
    }
}
#[test]
fn pinky_targets_ahead_right() {
    let map = create_test_map();
    let pac_pos = Vec2::new(100.0, 100.0);
    let ctx = make_ctx(10, Direction::Right, pac_pos, Vec2::ZERO, 0, Vec2::ZERO);
    let target = calculate_chase_target(GhostType::Pinky, &ctx, &map);

    // Target should be nearest node to (100 + 4*8, 100) = (132, 100)
    let expected_pos = pac_pos + Vec2::new(4.0 * 8.0, 0.0);
    let nearest = find_nearest_node_manual(&map, expected_pos);
    assert_that(&target).is_equal_to(nearest);
}

#[test]
fn pinky_targets_ahead_left() {
    let map = create_test_map();
    let pac_pos = Vec2::new(100.0, 100.0);
    let ctx = make_ctx(10, Direction::Left, pac_pos, Vec2::ZERO, 0, Vec2::ZERO);
    let target = calculate_chase_target(GhostType::Pinky, &ctx, &map);

    let expected_pos = pac_pos + Vec2::new(-4.0 * 8.0, 0.0);
    let nearest = find_nearest_node_manual(&map, expected_pos);
    assert_that(&target).is_equal_to(nearest);
}

#[test]
fn pinky_targets_ahead_down() {
    let map = create_test_map();
    let pac_pos = Vec2::new(100.0, 100.0);
    let ctx = make_ctx(10, Direction::Down, pac_pos, Vec2::ZERO, 0, Vec2::ZERO);
    let target = calculate_chase_target(GhostType::Pinky, &ctx, &map);

    let expected_pos = pac_pos + Vec2::new(0.0, 4.0 * 8.0);
    let nearest = find_nearest_node_manual(&map, expected_pos);
    assert_that(&target).is_equal_to(nearest);
}

#[test]
fn pinky_up_direction_overflow_bug() {
    let map = create_test_map();
    let pac_pos = Vec2::new(100.0, 100.0);
    let ctx = make_ctx(10, Direction::Up, pac_pos, Vec2::ZERO, 0, Vec2::ZERO);
    let target = calculate_chase_target(GhostType::Pinky, &ctx, &map);

    // Bug: when facing up, offset is (-4, -4) * TILE_SIZE = (-32, -32)
    let expected_pos = pac_pos + Vec2::new(-4.0 * 8.0, -4.0 * 8.0);
    let nearest = find_nearest_node_manual(&map, expected_pos);
    assert_that(&target).is_equal_to(nearest);
}
#[test]
fn inky_vector_doubling_right() {
    let map = create_test_map();
    let pac_pos = Vec2::new(100.0, 100.0);
    let blinky_pos = Vec2::new(80.0, 100.0);
    let ctx = make_ctx(10, Direction::Right, pac_pos, blinky_pos, 0, Vec2::ZERO);
    let target = calculate_chase_target(GhostType::Inky, &ctx, &map);

    // Intermediate: pac_pos + (2, 0) * 8 = (116, 100)
    // Vector from blinky to intermediate: (116-80, 100-100) = (36, 0)
    // Target: blinky + vector*2 = (80 + 72, 100) = (152, 100)
    let intermediate = pac_pos + Vec2::new(2.0 * 8.0, 0.0);
    let vector = intermediate - blinky_pos;
    let expected_pos = blinky_pos + vector * 2.0;
    let nearest = find_nearest_node_manual(&map, expected_pos);
    assert_that(&target).is_equal_to(nearest);
}

#[test]
fn inky_up_direction_also_has_overflow_bug() {
    let map = create_test_map();
    let pac_pos = Vec2::new(100.0, 100.0);
    let blinky_pos = Vec2::new(100.0, 80.0);
    let ctx = make_ctx(10, Direction::Up, pac_pos, blinky_pos, 0, Vec2::ZERO);
    let target = calculate_chase_target(GhostType::Inky, &ctx, &map);

    // Up bug: offset is (-2, -2) * 8 = (-16, -16)
    let intermediate = pac_pos + Vec2::new(-2.0 * 8.0, -2.0 * 8.0);
    let vector = intermediate - blinky_pos;
    let expected_pos = blinky_pos + vector * 2.0;
    let nearest = find_nearest_node_manual(&map, expected_pos);
    assert_that(&target).is_equal_to(nearest);
}
#[test]
fn clyde_chases_when_far() {
    let map = create_test_map();
    let pac_pos = Vec2::new(100.0, 100.0);
    // Place Clyde far away (>= 8 tiles = 64 pixels)
    let self_pos = Vec2::new(200.0, 100.0); // 100 pixels away
    let ctx = make_ctx(10, Direction::Right, pac_pos, Vec2::ZERO, 0, self_pos);
    let target = calculate_chase_target(GhostType::Clyde, &ctx, &map);
    // Should target pacman_node
    assert_that(&target).is_equal_to(10);
}

#[test]
fn clyde_scatters_when_close() {
    let map = create_test_map();
    let pac_pos = Vec2::new(100.0, 100.0);
    // Place Clyde close (< 8 tiles = 64 pixels)
    let self_pos = Vec2::new(140.0, 100.0); // 40 pixels away
    let self_node = 5u16;
    let ctx = make_ctx(10, Direction::Right, pac_pos, Vec2::ZERO, self_node, self_pos);
    let target = calculate_chase_target(GhostType::Clyde, &ctx, &map);
    // Should return self_node (signal for scatter)
    assert_that(&target).is_equal_to(self_node);
}

#[test]
fn clyde_threshold_exactly_8_tiles_chases() {
    let map = create_test_map();
    let pac_pos = Vec2::new(100.0, 100.0);
    // Exactly 8 tiles (64 pixels) away: distance_squared = 64*64 = 4096
    // threshold * threshold = 64 * 64 = 4096
    // distance_sq >= threshold*threshold -> chases
    let self_pos = Vec2::new(164.0, 100.0);
    let ctx = make_ctx(10, Direction::Right, pac_pos, Vec2::ZERO, 5, self_pos);
    let target = calculate_chase_target(GhostType::Clyde, &ctx, &map);
    assert_that(&target).is_equal_to(10); // pacman_node
}

#[test]
fn clyde_threshold_just_under_8_tiles_scatters() {
    let map = create_test_map();
    let pac_pos = Vec2::new(100.0, 100.0);
    // Just under 64 pixels away
    let self_pos = Vec2::new(163.9, 100.0);
    let self_node = 5u16;
    let ctx = make_ctx(10, Direction::Right, pac_pos, Vec2::ZERO, self_node, self_pos);
    let target = calculate_chase_target(GhostType::Clyde, &ctx, &map);
    assert_that(&target).is_equal_to(self_node); // scatter
}

/// Manual implementation of find_nearest_node for test validation
fn find_nearest_node_manual(map: &pacman::map::builder::Map, target_pos: Vec2) -> u16 {
    let mut best_node = 0u16;
    let mut best_dist = f32::MAX;

    for (i, node) in map.graph.nodes().enumerate() {
        let dist = node.position.distance_squared(target_pos);
        if dist < best_dist {
            best_dist = dist;
            best_node = i as u16;
        }
    }

    best_node
}
