use smallvec::SmallVec;
use std::collections::HashMap;

use crate::entity::{graph::NodeId, traversal::Position};

/// Trait for entities that can participate in collision detection.
pub trait Collidable {
    /// Returns the current position of this entity.
    fn position(&self) -> Position;

    /// Checks if this entity is colliding with another entity.
    #[allow(dead_code)]
    fn is_colliding_with(&self, other: &dyn Collidable) -> bool {
        positions_overlap(&self.position(), &other.position())
    }
}

/// System for tracking entities by their positions for efficient collision detection.
#[derive(Default)]
pub struct CollisionSystem {
    /// Maps node IDs to lists of entity IDs that are at that node
    node_entities: HashMap<NodeId, Vec<EntityId>>,
    /// Maps entity IDs to their current positions
    entity_positions: HashMap<EntityId, Position>,
    /// Next available entity ID
    next_id: EntityId,
}

/// Unique identifier for an entity in the collision system
pub type EntityId = u32;

impl CollisionSystem {
    /// Registers an entity with the collision system and returns its ID
    pub fn register_entity(&mut self, position: Position) -> EntityId {
        let id = self.next_id;
        self.next_id += 1;

        self.entity_positions.insert(id, position);
        self.update_node_entities(id, position);

        id
    }

    /// Updates an entity's position
    pub fn update_position(&mut self, entity_id: EntityId, new_position: Position) {
        if let Some(old_position) = self.entity_positions.get(&entity_id) {
            // Remove from old nodes
            self.remove_from_nodes(entity_id, *old_position);
        }

        // Update position and add to new nodes
        self.entity_positions.insert(entity_id, new_position);
        self.update_node_entities(entity_id, new_position);
    }

    /// Removes an entity from the collision system
    #[allow(dead_code)]
    pub fn remove_entity(&mut self, entity_id: EntityId) {
        if let Some(position) = self.entity_positions.remove(&entity_id) {
            self.remove_from_nodes(entity_id, position);
        }
    }

    /// Gets all entity IDs at a specific node
    pub fn entities_at_node(&self, node: NodeId) -> &[EntityId] {
        self.node_entities.get(&node).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Gets all entity IDs that could collide with an entity at the given position
    pub fn potential_collisions(&self, position: &Position) -> Vec<EntityId> {
        let mut collisions = Vec::new();
        let nodes = get_nodes(position);

        for node in nodes {
            collisions.extend(self.entities_at_node(node));
        }

        // Remove duplicates
        collisions.sort_unstable();
        collisions.dedup();
        collisions
    }

    /// Updates the node_entities map when an entity's position changes
    fn update_node_entities(&mut self, entity_id: EntityId, position: Position) {
        let nodes = get_nodes(&position);
        for node in nodes {
            self.node_entities.entry(node).or_default().push(entity_id);
        }
    }

    /// Removes an entity from all nodes it was previously at
    fn remove_from_nodes(&mut self, entity_id: EntityId, position: Position) {
        let nodes = get_nodes(&position);
        for node in nodes {
            if let Some(entities) = self.node_entities.get_mut(&node) {
                entities.retain(|&id| id != entity_id);
                if entities.is_empty() {
                    self.node_entities.remove(&node);
                }
            }
        }
    }
}

/// Checks if two positions overlap (entities are at the same location).
fn positions_overlap(a: &Position, b: &Position) -> bool {
    let a_nodes = get_nodes(a);
    let b_nodes = get_nodes(b);

    // Check if any nodes overlap
    a_nodes.iter().any(|a_node| b_nodes.contains(a_node))

    // TODO: More complex overlap detection, the above is a simple check, but it could become an early filter for more precise calculations later
}

/// Gets all nodes that an entity is currently at or between.
fn get_nodes(pos: &Position) -> SmallVec<[NodeId; 2]> {
    let mut nodes = SmallVec::new();
    match pos {
        Position::AtNode(node) => nodes.push(*node),
        Position::BetweenNodes { from, to, .. } => {
            nodes.push(*from);
            nodes.push(*to);
        }
    }
    nodes
}
