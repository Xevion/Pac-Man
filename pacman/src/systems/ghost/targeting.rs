//! Ghost targeting and intersection decision logic.

use crate::map::builder::Map;
use crate::map::direction::Direction;
use crate::map::graph::TraversalFlags;
use crate::systems::movement::NodeId;
use bevy_ecs::resource::Resource;
use glam::Vec2;
use rand::prelude::IndexedRandom;

/// Nodes where ghosts cannot turn upward (except when frightened).
/// Located at the 4 tiles directly above the ghost house entrances:
/// grid positions (12,14), (15,14), (12,26), (15,26).
#[derive(Resource, Debug)]
pub struct RedZoneNodes(pub [NodeId; 4]);

impl RedZoneNodes {
    pub fn from_map(map: &Map) -> Self {
        use glam::I8Vec2;

        let red_zone_coords = [
            I8Vec2::new(12, 14),
            I8Vec2::new(15, 14),
            I8Vec2::new(12, 26),
            I8Vec2::new(15, 26),
        ];

        let mut nodes = [0; 4];
        for (i, coord) in red_zone_coords.iter().enumerate() {
            if let Some(&node_id) = map.grid_to_node.get(coord) {
                nodes[i] = node_id;
            }
        }

        Self(nodes)
    }

    pub fn contains(&self, node: NodeId) -> bool {
        self.0.contains(&node)
    }
}

/// Determines the best direction for a ghost at an intersection
pub fn choose_direction_at_intersection(
    map: &Map,
    red_zones: &RedZoneNodes,
    current_node: NodeId,
    target_node: NodeId,
    current_direction: Direction,
    is_frightened: bool,
    rng: Option<&mut dyn rand::RngCore>,
) -> Option<Direction> {
    let intersection = &map.graph.adjacency_list[current_node as usize];
    let opposite = current_direction.opposite();

    // Collect valid directions (not opposite, traversable by ghosts)
    let mut candidates: Vec<(Direction, f32)> = Vec::with_capacity(3);

    for dir in Direction::DIRECTIONS {
        // Cannot reverse direction
        if dir == opposite {
            continue;
        }

        // Check if edge exists and is ghost-traversable
        let Some(edge) = intersection.get(dir) else {
            continue;
        };

        if !edge.traversal_flags.contains(TraversalFlags::GHOST) {
            continue;
        }

        // Red zone check: no upward turns unless frightened
        if dir == Direction::Up && !is_frightened && red_zones.contains(current_node) {
            continue;
        }

        // Calculate distance from next node to target
        let next_node = edge.target;
        let Some(next) = map.graph.get_node(next_node) else { continue };
        let Some(target) = map.graph.get_node(target_node) else {
            continue;
        };
        let distance = next.position.distance_squared(target.position);

        candidates.push((dir, distance));
    }

    if candidates.is_empty() {
        // Dead end - must reverse (shouldn't happen in normal maze)
        return intersection.get(opposite).map(|_| opposite);
    }

    if is_frightened {
        // Random choice when frightened (with direction priority for determinism on ties)
        if let Some(rng) = rng {
            return candidates.choose(rng).map(|(d, _)| *d);
        }
    }

    // Normal mode: choose direction with shortest distance to target
    // Tie-breaker priority: Up > Left > Down > Right
    candidates.sort_by(|(dir_a, dist_a), (dir_b, dist_b)| {
        dist_a
            .partial_cmp(dist_b)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| direction_priority(*dir_a).cmp(&direction_priority(*dir_b)))
    });

    candidates.first().map(|(d, _)| *d)
}

fn direction_priority(dir: Direction) -> u8 {
    match dir {
        Direction::Up => 0,
        Direction::Left => 1,
        Direction::Down => 2,
        Direction::Right => 3,
    }
}

/// Find the nearest graph node to a world position
pub fn find_nearest_node(map: &Map, target_pos: Vec2) -> NodeId {
    let mut best_node = 0;
    let mut best_dist = f32::MAX;

    for (i, node) in map.graph.nodes().enumerate() {
        let dist = node.position.distance_squared(target_pos);
        if dist < best_dist {
            best_dist = dist;
            best_node = i as NodeId;
        }
    }

    best_node
}
