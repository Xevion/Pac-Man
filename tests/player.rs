use pacman::map::direction::Direction;
use pacman::map::graph::{Edge, TraversalFlags};
use pacman::systems::components::EntityType;
use pacman::systems::player::can_traverse;

#[test]
fn test_can_traverse_player_on_all_edges() {
    let edge = Edge {
        target: 1,
        distance: 10.0,
        direction: Direction::Up,
        traversal_flags: TraversalFlags::ALL,
    };

    assert!(can_traverse(EntityType::Player, edge));
}

#[test]
fn test_can_traverse_player_on_pacman_only_edges() {
    let edge = Edge {
        target: 1,
        distance: 10.0,
        direction: Direction::Right,
        traversal_flags: TraversalFlags::PACMAN,
    };

    assert!(can_traverse(EntityType::Player, edge));
}

#[test]
fn test_can_traverse_player_blocked_on_ghost_only_edges() {
    let edge = Edge {
        target: 1,
        distance: 10.0,
        direction: Direction::Left,
        traversal_flags: TraversalFlags::GHOST,
    };

    assert!(!can_traverse(EntityType::Player, edge));
}

#[test]
fn test_can_traverse_ghost_on_all_edges() {
    let edge = Edge {
        target: 2,
        distance: 15.0,
        direction: Direction::Down,
        traversal_flags: TraversalFlags::ALL,
    };

    assert!(can_traverse(EntityType::Ghost, edge));
}

#[test]
fn test_can_traverse_ghost_on_ghost_only_edges() {
    let edge = Edge {
        target: 2,
        distance: 15.0,
        direction: Direction::Up,
        traversal_flags: TraversalFlags::GHOST,
    };

    assert!(can_traverse(EntityType::Ghost, edge));
}

#[test]
fn test_can_traverse_ghost_blocked_on_pacman_only_edges() {
    let edge = Edge {
        target: 2,
        distance: 15.0,
        direction: Direction::Right,
        traversal_flags: TraversalFlags::PACMAN,
    };

    assert!(!can_traverse(EntityType::Ghost, edge));
}

#[test]
fn test_can_traverse_static_entities_flags() {
    let edge = Edge {
        target: 3,
        distance: 8.0,
        direction: Direction::Left,
        traversal_flags: TraversalFlags::ALL,
    };

    // Static entities have empty traversal flags but can still "traverse"
    // in the sense that empty flags are contained in any flag set
    // This is the expected behavior since empty ⊆ any set
    assert!(can_traverse(EntityType::Pellet, edge));
    assert!(can_traverse(EntityType::PowerPellet, edge));
}

#[test]
fn test_entity_type_traversal_flags() {
    assert_eq!(EntityType::Player.traversal_flags(), TraversalFlags::PACMAN);
    assert_eq!(EntityType::Ghost.traversal_flags(), TraversalFlags::GHOST);
    assert_eq!(EntityType::Pellet.traversal_flags(), TraversalFlags::empty());
    assert_eq!(EntityType::PowerPellet.traversal_flags(), TraversalFlags::empty());
}
