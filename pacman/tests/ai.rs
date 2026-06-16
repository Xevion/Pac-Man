//! Tests for the attract-mode AI's direction chooser.
//!
//! The stub AI reuses the ghost intersection logic with Pac-Man's traversal flags
//! and no red-zone restriction. This verifies that parameterization: the chooser
//! must pick a Pac-Man-traversable, non-reversing edge that greedily steps toward
//! the target.

use bevy_ecs::event::{EventRegistry, Events};
use bevy_ecs::world::World;

use pacman::events::{GameCommand, GameEvent};
use pacman::map::direction::Direction;
use pacman::map::graph::TraversalFlags;
use pacman::systems::ai::ai_player_system;
use pacman::systems::collision::ItemCollider;
use pacman::systems::common::Frozen;
use pacman::systems::ghost::targeting::choose_direction_at_intersection;
use pacman::systems::movement::{Position, Velocity};
use pacman::systems::player::PlayerControlled;
use pacman::systems::profiling::profile;
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
                let ghost_only = edge.traversal_flags.contains(TraversalFlags::GHOST)
                    && !edge.traversal_flags.contains(TraversalFlags::PACMAN);
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

/// Builds a minimal world holding just what `ai_player_system` reads, then runs the
/// system through the real `profile()` wrapper -- the same path the schedule uses,
/// which calls `System::run` directly and bypasses Bevy's param validation. Running
/// it through a plain `Schedule` instead would let validation silently skip an empty
/// `Single`, hiding exactly the regression these tests guard against.
fn run_ai_player_system(world: &mut World) {
    let mut system = profile("playercontrols", ai_player_system);
    system(world);
}

/// Regression: attract entered with the player mid-intro-freeze. The player exists and
/// carries Position/Velocity but is `Frozen`, so the AI's `Without<Frozen>` query is
/// empty. A bare `Single` panicked here ("expected exactly one matching entity") the
/// instant attract activated; `Option<Single>` must make it a no-op instead.
#[test]
fn ai_player_system_skips_frozen_player_without_panicking() {
    let mut world = World::new();
    EventRegistry::register_event::<GameEvent>(&mut world);
    world.insert_resource(common::create_test_map());

    world.spawn((
        PlayerControlled,
        Position::Stopped { node: 0 },
        Velocity {
            speed: 0.0,
            direction: Direction::Left,
        },
        Frozen,
    ));

    run_ai_player_system(&mut world);
}

/// Regression: the AI must also tolerate no player entity at all (e.g. a frame before
/// the scene's spawn lands), rather than panicking on the empty `Single`.
#[test]
fn ai_player_system_skips_absent_player_without_panicking() {
    let mut world = World::new();
    EventRegistry::register_event::<GameEvent>(&mut world);
    world.insert_resource(common::create_test_map());

    run_ai_player_system(&mut world);
}

/// With a movable player and a remaining pellet, the AI still produces a `MovePlayer`
/// command through the full system + `profile()` path -- proving the `Option<Single>`
/// guard didn't turn the system into a silent no-op.
#[test]
fn ai_player_system_emits_move_toward_pellet() {
    let mut world = World::new();
    EventRegistry::register_event::<GameEvent>(&mut world);

    let map = common::create_test_map();
    let start = map.start_positions.pacman;
    // Place the only pellet at the node farthest from the start, so `nearest_pellet_node`
    // is unambiguous and the chooser has a clear forward step (same gradient the chooser
    // test relies on).
    let origin = map.graph.get_node(start).unwrap().position;
    let (target, _) = map
        .graph
        .nodes()
        .enumerate()
        .map(|(i, node)| (i as u16, node.position.distance_squared(origin)))
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .expect("the maze has nodes");
    world.insert_resource(map);

    world.spawn((
        PlayerControlled,
        Position::Stopped { node: start },
        Velocity {
            speed: 0.0,
            direction: Direction::Left,
        },
    ));
    world.spawn((ItemCollider, Position::Stopped { node: target }));

    run_ai_player_system(&mut world);

    let emitted: Vec<GameEvent> = world.resource_mut::<Events<GameEvent>>().drain().collect();
    let moved = emitted
        .iter()
        .any(|e| matches!(e, GameEvent::Command(GameCommand::MovePlayer(_))));
    assert_that(&moved).is_true();
}
