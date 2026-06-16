//! Tests for the attract-mode AI's direction chooser.
//!
//! The stub AI reuses the ghost intersection logic with Pac-Man's traversal flags
//! and no red-zone restriction. This verifies that parameterization: the chooser
//! must pick a Pac-Man-traversable, non-reversing edge that greedily steps toward
//! the target.

use pacman::map::direction::Direction;
use pacman::map::graph::TraversalFlags;
use pacman::systems::ghost::targeting::choose_direction_at_intersection;
use speculoos::prelude::*;

mod common;

#[test]
fn ai_chooser_picks_pacman_traversable_step_toward_target() {
    let map = common::create_test_map();
    let current = map.start_positions.pacman;
    let current_direction = Direction::Left;

    // Target the node farthest from the start, giving an unambiguous gradient to
    // greedily descend.
    let origin = map.graph.get_node(current).unwrap().position;
    let (target, _) = map
        .graph
        .nodes()
        .enumerate()
        .map(|(i, node)| (i as u16, node.position.distance_squared(origin)))
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .expect("the maze has nodes");

    let chosen = choose_direction_at_intersection(
        &map,
        None,
        TraversalFlags::PACMAN,
        current,
        target,
        current_direction,
        false,
        None,
    )
    .expect("a corridor node always offers a forward option");

    let intersection = &map.graph.adjacency_list[current as usize];

    // The chosen direction is a real, Pac-Man-traversable, non-reversing edge.
    let chosen_edge = intersection.get(chosen).expect("chosen direction has an edge");
    assert_that(&chosen_edge.traversal_flags.contains(TraversalFlags::PACMAN)).is_true();
    assert_that(&(chosen == current_direction.opposite())).is_false();

    // It is the greedy step: no other valid candidate lands nearer the target.
    let target_pos = map.graph.get_node(target).unwrap().position;
    let chosen_dist = map
        .graph
        .get_node(chosen_edge.target)
        .unwrap()
        .position
        .distance_squared(target_pos);

    for dir in Direction::DIRECTIONS {
        if dir == current_direction.opposite() {
            continue;
        }
        let Some(edge) = intersection.get(dir) else {
            continue;
        };
        if !edge.traversal_flags.contains(TraversalFlags::PACMAN) {
            continue;
        }
        let dist = map.graph.get_node(edge.target).unwrap().position.distance_squared(target_pos);
        assert_that(&(chosen_dist <= dist)).is_true();
    }
}

/// The `flags` parameter is load-bearing: at the ghost-house door (a GHOST-only
/// edge) the chooser must refuse that direction for Pac-Man even when it points
/// straight at the target, while the same call with GHOST flags takes it.
#[test]
fn ai_chooser_respects_pacman_flags_at_ghost_door() {
    let map = common::create_test_map();

    // Locate a ghost-only edge -- the maze has exactly the house door.
    let (src, ghost_only_dir, target) = map
        .graph
        .adjacency_list
        .iter()
        .enumerate()
        .find_map(|(i, intersection)| {
            Direction::DIRECTIONS.into_iter().find_map(|dir| {
                let edge = intersection.get(dir)?;
                let ghost_only =
                    edge.traversal_flags.contains(TraversalFlags::GHOST) && !edge.traversal_flags.contains(TraversalFlags::PACMAN);
                ghost_only.then_some((i as u16, dir, edge.target))
            })
        })
        .expect("the maze has a ghost-only house door");

    // Aim straight at the ghost-only edge's destination, with a heading that leaves
    // that direction eligible (not the reverse), so only the flags can exclude it.
    let pacman_choice =
        choose_direction_at_intersection(&map, None, TraversalFlags::PACMAN, src, target, ghost_only_dir, false, None);
    let ghost_choice =
        choose_direction_at_intersection(&map, None, TraversalFlags::GHOST, src, target, ghost_only_dir, false, None);

    assert_that(&(pacman_choice == Some(ghost_only_dir))).is_false();
    assert_that(&ghost_choice).is_equal_to(Some(ghost_only_dir));
}
