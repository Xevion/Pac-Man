use pacman::entity::collision::{Collidable, CollisionSystem};
use pacman::entity::traversal::Position;

struct MockCollidable {
    pos: Position,
}

impl Collidable for MockCollidable {
    fn position(&self) -> Position {
        self.pos
    }
}

#[test]
fn test_is_colliding_with() {
    let entity1 = MockCollidable {
        pos: Position::AtNode(1),
    };
    let entity2 = MockCollidable {
        pos: Position::AtNode(1),
    };
    let entity3 = MockCollidable {
        pos: Position::AtNode(2),
    };
    let entity4 = MockCollidable {
        pos: Position::BetweenNodes {
            from: 1,
            to: 2,
            traversed: 0.5,
        },
    };

    assert!(entity1.is_colliding_with(&entity2));
    assert!(!entity1.is_colliding_with(&entity3));
    assert!(entity1.is_colliding_with(&entity4));
    assert!(entity3.is_colliding_with(&entity4));
}

#[test]
fn test_collision_system_register_and_query() {
    let mut collision_system = CollisionSystem::default();

    let pos1 = Position::AtNode(1);
    let entity1 = collision_system.register_entity(pos1);

    let pos2 = Position::BetweenNodes {
        from: 1,
        to: 2,
        traversed: 0.5,
    };
    let entity2 = collision_system.register_entity(pos2);

    let pos3 = Position::AtNode(3);
    let entity3 = collision_system.register_entity(pos3);

    // Test entities_at_node
    assert_eq!(collision_system.entities_at_node(1), &[entity1, entity2]);
    assert_eq!(collision_system.entities_at_node(2), &[entity2]);
    assert_eq!(collision_system.entities_at_node(3), &[entity3]);
    assert_eq!(collision_system.entities_at_node(4), &[] as &[u32]);

    // Test potential_collisions
    let mut collisions1 = collision_system.potential_collisions(&pos1);
    collisions1.sort_unstable();
    assert_eq!(collisions1, vec![entity1, entity2]);

    let mut collisions2 = collision_system.potential_collisions(&pos2);
    collisions2.sort_unstable();
    assert_eq!(collisions2, vec![entity1, entity2]);

    let mut collisions3 = collision_system.potential_collisions(&pos3);
    collisions3.sort_unstable();
    assert_eq!(collisions3, vec![entity3]);
}

#[test]
fn test_collision_system_update() {
    let mut collision_system = CollisionSystem::default();

    let entity1 = collision_system.register_entity(Position::AtNode(1));

    assert_eq!(collision_system.entities_at_node(1), &[entity1]);
    assert_eq!(collision_system.entities_at_node(2), &[] as &[u32]);

    collision_system.update_position(entity1, Position::AtNode(2));

    assert_eq!(collision_system.entities_at_node(1), &[] as &[u32]);
    assert_eq!(collision_system.entities_at_node(2), &[entity1]);

    collision_system.update_position(
        entity1,
        Position::BetweenNodes {
            from: 2,
            to: 3,
            traversed: 0.1,
        },
    );

    assert_eq!(collision_system.entities_at_node(1), &[] as &[u32]);
    assert_eq!(collision_system.entities_at_node(2), &[entity1]);
    assert_eq!(collision_system.entities_at_node(3), &[entity1]);
}

#[test]
fn test_collision_system_remove() {
    let mut collision_system = CollisionSystem::default();

    let entity1 = collision_system.register_entity(Position::AtNode(1));
    let entity2 = collision_system.register_entity(Position::AtNode(1));

    assert_eq!(collision_system.entities_at_node(1), &[entity1, entity2]);

    collision_system.remove_entity(entity1);

    assert_eq!(collision_system.entities_at_node(1), &[entity2]);

    collision_system.remove_entity(entity2);
    assert_eq!(collision_system.entities_at_node(1), &[] as &[u32]);
}
