use std::collections::HashMap;

use crate::platform;
use crate::systems::{DirectionalAnimation, Frozen, LinearAnimation, Looping};
use crate::{
    map::{
        builder::Map,
        direction::Direction,
        graph::{Edge, TraversalFlags},
    },
    systems::{
        components::DeltaTime,
        movement::{Position, Velocity},
    },
};
use bevy_ecs::component::Component;
use bevy_ecs::resource::Resource;
use tracing::{debug, trace, warn};

use bevy_ecs::query::Without;
use bevy_ecs::system::{Commands, Query, Res};
use rand::seq::IndexedRandom;
use smallvec::SmallVec;

/// Tag component for eaten ghosts
#[derive(Component, Debug, Clone, Copy)]
pub struct Eaten;

/// Tag component for Pac-Man during his death animation.
/// This is mainly because the Frozen tag would stop both movement and animation, while the Dying tag can signal that the animation should continue despite being frozen.
#[derive(Component, Debug, Clone, Copy)]
pub struct Dying;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ghost {
    Blinky,
    Pinky,
    Inky,
    Clyde,
}

impl Ghost {
    /// Returns the ghost type name for atlas lookups.
    pub fn as_str(self) -> &'static str {
        match self {
            Ghost::Blinky => "blinky",
            Ghost::Pinky => "pinky",
            Ghost::Inky => "inky",
            Ghost::Clyde => "clyde",
        }
    }

    /// Returns the base movement speed for this ghost type.
    pub fn base_speed(self) -> f32 {
        match self {
            Ghost::Blinky => 1.0,
            Ghost::Pinky => 0.95,
            Ghost::Inky => 0.9,
            Ghost::Clyde => 0.85,
        }
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub enum GhostState {
    /// Normal ghost behavior - chasing Pac-Man
    Normal,
    /// Frightened state after power pellet - ghost can be eaten
    Frightened {
        remaining_ticks: u32,
        flash: bool,
        remaining_flash_ticks: u32,
    },
    /// Eyes state - ghost has been eaten and is returning to ghost house
    Eyes,
}

impl GhostState {
    /// Creates a new frightened state with the specified duration
    pub fn new_frightened(total_ticks: u32, flash_start_ticks: u32) -> Self {
        Self::Frightened {
            remaining_ticks: total_ticks,
            flash: false,
            remaining_flash_ticks: flash_start_ticks, // Time until flashing starts
        }
    }

    /// Ticks the ghost state, returning true if the state changed.
    pub fn tick(&mut self) -> bool {
        if let GhostState::Frightened {
            remaining_ticks,
            flash,
            remaining_flash_ticks,
        } = self
        {
            // Transition out of frightened state
            if *remaining_ticks == 0 {
                *self = GhostState::Normal;
                return true;
            }

            *remaining_ticks -= 1;

            if *remaining_flash_ticks > 0 {
                *remaining_flash_ticks = remaining_flash_ticks.saturating_sub(1);
                if *remaining_flash_ticks == 0 {
                    *flash = true;
                    true
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Returns the appropriate animation state for this ghost state
    pub fn animation_state(&self) -> GhostAnimation {
        match self {
            GhostState::Normal => GhostAnimation::Normal,
            GhostState::Eyes => GhostAnimation::Eyes,
            GhostState::Frightened { flash: false, .. } => GhostAnimation::Frightened { flash: false },
            GhostState::Frightened { flash: true, .. } => GhostAnimation::Frightened { flash: true },
        }
    }
}

/// Enumeration of different ghost animation states.
/// Note that this is used in micromap which has a fixed size based on the number of variants,
/// so extending this should be done with caution, and will require updating the micromap's capacity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GhostAnimation {
    /// Normal ghost appearance with directional movement animations
    Normal,
    /// Blue ghost appearance when vulnerable (power pellet active)
    Frightened { flash: bool },
    /// Eyes-only animation when ghost has been consumed by Pac-Man (Eaten state)
    Eyes,
}

/// Global resource containing pre-loaded animation sets for all ghost types.
///
/// This resource is initialized once during game startup and provides O(1) access
/// to animation sets for each ghost type. The animation system uses this resource
/// to efficiently switch between different ghost states without runtime asset loading.
///
/// The HashMap is keyed by `Ghost` enum variants (Blinky, Pinky, Inky, Clyde) and
/// contains the normal directional animation for each ghost type.
#[derive(Resource)]
pub struct GhostAnimations {
    pub normal: HashMap<Ghost, DirectionalAnimation>,
    pub eyes: DirectionalAnimation,
    pub frightened: LinearAnimation,
    pub frightened_flashing: LinearAnimation,
}

impl GhostAnimations {
    /// Creates a new GhostAnimations resource with the provided data.
    pub fn new(
        normal: HashMap<Ghost, DirectionalAnimation>,
        eyes: DirectionalAnimation,
        frightened: LinearAnimation,
        frightened_flashing: LinearAnimation,
    ) -> Self {
        Self {
            normal,
            eyes,
            frightened,
            frightened_flashing,
        }
    }

    /// Gets the normal directional animation for the specified ghost type.
    pub fn get_normal(&self, ghost_type: &Ghost) -> Option<&DirectionalAnimation> {
        self.normal.get(ghost_type)
    }

    /// Gets the eyes animation (shared across all ghosts).
    pub fn eyes(&self) -> &DirectionalAnimation {
        &self.eyes
    }

    /// Gets the frightened animations (shared across all ghosts).
    pub fn frightened(&self, flash: bool) -> &LinearAnimation {
        if flash {
            &self.frightened_flashing
        } else {
            &self.frightened
        }
    }
}

/// Autonomous ghost AI system implementing randomized movement with backtracking avoidance.
pub fn ghost_movement_system(
    map: Res<Map>,
    delta_time: Res<DeltaTime>,
    mut ghosts: Query<(&Ghost, &mut Velocity, &mut Position), Without<Frozen>>,
) {
    for (_ghost, mut velocity, mut position) in ghosts.iter_mut() {
        let mut distance = velocity.speed * 60.0 * delta_time.seconds;
        loop {
            match *position {
                Position::Stopped { node: current_node } => {
                    let intersection = &map.graph.adjacency_list[current_node as usize];
                    let opposite = velocity.direction.opposite();

                    let mut non_opposite_options: SmallVec<[Edge; 3]> = SmallVec::new();

                    // Collect all available directions that ghosts can traverse
                    for edge in Direction::DIRECTIONS.iter().flat_map(|d| intersection.get(*d)) {
                        if edge.traversal_flags.contains(TraversalFlags::GHOST) && edge.direction != opposite {
                            non_opposite_options.push(edge);
                        }
                    }

                    let new_edge: Edge = if non_opposite_options.is_empty() {
                        if let Some(edge) = intersection.get(opposite) {
                            trace!(node = current_node, ghost = ?_ghost, direction = ?opposite, "Ghost forced to reverse direction");
                            edge
                        } else {
                            warn!(node = current_node, ghost = ?_ghost, "Ghost stuck with no available directions");
                            break;
                        }
                    } else {
                        *non_opposite_options.choose(&mut platform::rng()).unwrap()
                    };

                    velocity.direction = new_edge.direction;
                    *position = Position::Moving {
                        from: current_node,
                        to: new_edge.target,
                        remaining_distance: new_edge.distance,
                    };
                }
                Position::Moving { .. } => {
                    if let Some(overflow) = position.tick(distance) {
                        distance = overflow;
                    } else {
                        break;
                    }
                }
            }
        }
    }
}

/// System that handles eaten ghost behavior and respawn logic.
///
/// When a ghost is eaten by Pac-Man, it enters an "eaten" state where:
/// 1. It displays eyes-only animation
/// 2. It moves directly back to the ghost house at increased speed
/// 3. Once it reaches the ghost house center, it respawns as a normal ghost
///
/// This system runs after the main movement system to override eaten ghost movement.
pub fn eaten_ghost_system(
    map: Res<Map>,
    delta_time: Res<DeltaTime>,
    mut eaten_ghosts: Query<(&Ghost, &mut Position, &mut Velocity, &mut GhostState)>,
) {
    for (ghost_type, mut position, mut velocity, mut ghost_state) in eaten_ghosts.iter_mut() {
        // Only process ghosts that are in Eyes state
        if !matches!(*ghost_state, GhostState::Eyes) {
            continue;
        }
        // Set higher speed for eaten ghosts returning to ghost house
        let original_speed = velocity.speed;
        velocity.speed = ghost_type.base_speed() * 2.0; // Move twice as fast when eaten

        // Calculate direction towards ghost house center (using Clyde's start position)
        let ghost_house_center = map.start_positions.clyde;

        match *position {
            Position::Stopped { node: current_node } => {
                // Find path to ghost house center and start moving
                if let Some(direction) = find_direction_to_target(&map, current_node, ghost_house_center) {
                    velocity.direction = direction;
                    *position = Position::Moving {
                        from: current_node,
                        to: map.graph.adjacency_list[current_node as usize].get(direction).unwrap().target,
                        remaining_distance: map.graph.adjacency_list[current_node as usize]
                            .get(direction)
                            .unwrap()
                            .distance,
                    };
                }
            }
            Position::Moving { to, .. } => {
                let distance = velocity.speed * 60.0 * delta_time.seconds;
                if let Some(_overflow) = position.tick(distance) {
                    // Reached target node, check if we're at ghost house center
                    if to == ghost_house_center {
                        // Respawn the ghost - set state back to normal
                        debug!(ghost = ?ghost_type, "Eaten ghost reached ghost house, respawning as normal");
                        *ghost_state = GhostState::Normal;
                        // Reset to stopped at ghost house center
                        *position = Position::Stopped {
                            node: ghost_house_center,
                        };
                    } else {
                        // Continue pathfinding to ghost house
                        if let Some(next_direction) = find_direction_to_target(&map, to, ghost_house_center) {
                            velocity.direction = next_direction;
                            *position = Position::Moving {
                                from: to,
                                to: map.graph.adjacency_list[to as usize].get(next_direction).unwrap().target,
                                remaining_distance: map.graph.adjacency_list[to as usize].get(next_direction).unwrap().distance,
                            };
                        }
                    }
                }
            }
        }

        // Restore original speed
        velocity.speed = original_speed;
    }
}

/// Helper function to find the direction from a node towards a target node.
/// Uses simple greedy pathfinding - prefers straight lines when possible.
fn find_direction_to_target(
    map: &Map,
    from_node: crate::systems::movement::NodeId,
    target_node: crate::systems::movement::NodeId,
) -> Option<Direction> {
    let from_pos = map.graph.get_node(from_node).unwrap().position;
    let target_pos = map.graph.get_node(target_node).unwrap().position;

    let dx = target_pos.x as i32 - from_pos.x as i32;
    let dy = target_pos.y as i32 - from_pos.y as i32;

    // Prefer horizontal movement first, then vertical
    let preferred_dirs = if dx.abs() > dy.abs() {
        if dx > 0 {
            [Direction::Right, Direction::Up, Direction::Down, Direction::Left]
        } else {
            [Direction::Left, Direction::Up, Direction::Down, Direction::Right]
        }
    } else if dy > 0 {
        [Direction::Down, Direction::Left, Direction::Right, Direction::Up]
    } else {
        [Direction::Up, Direction::Left, Direction::Right, Direction::Down]
    };

    // Return first available direction towards target
    for direction in preferred_dirs {
        if let Some(edge) = map.graph.adjacency_list[from_node as usize].get(direction) {
            if edge.traversal_flags.contains(TraversalFlags::GHOST) {
                return Some(direction);
            }
        }
    }

    None
}

/// Component to track the last animation state for efficient change detection
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct LastAnimationState(pub GhostAnimation);

/// Unified system that manages ghost state transitions and animations with component swapping
pub fn ghost_state_system(
    mut commands: Commands,
    animations: Res<GhostAnimations>,
    mut ghosts: Query<(bevy_ecs::entity::Entity, &Ghost, &mut GhostState, &mut LastAnimationState)>,
) {
    for (entity, ghost_type, mut ghost_state, mut last_animation_state) in ghosts.iter_mut() {
        // Tick the ghost state to handle internal transitions (like flashing)
        let _ = ghost_state.tick();

        // Only update animation if the animation state actually changed
        let current_animation_state = ghost_state.animation_state();
        if last_animation_state.0 != current_animation_state {
            trace!(ghost = ?ghost_type, old_state = ?last_animation_state.0, new_state = ?current_animation_state, "Ghost animation state changed");
            match current_animation_state {
                GhostAnimation::Frightened { flash } => {
                    // Remove DirectionalAnimation, add LinearAnimation with Looping component
                    commands
                        .entity(entity)
                        .remove::<DirectionalAnimation>()
                        .insert(animations.frightened(flash).clone())
                        .insert(Looping);
                }
                GhostAnimation::Normal => {
                    // Remove LinearAnimation and Looping, add DirectionalAnimation
                    commands
                        .entity(entity)
                        .remove::<(LinearAnimation, Looping)>()
                        .insert(animations.get_normal(ghost_type).unwrap().clone());
                }
                GhostAnimation::Eyes => {
                    // Remove LinearAnimation and Looping, add DirectionalAnimation (eyes animation)
                    trace!(ghost = ?ghost_type, "Switching to eyes animation for eaten ghost");
                    commands
                        .entity(entity)
                        .remove::<(LinearAnimation, Looping)>()
                        .insert(animations.eyes().clone());
                }
            }
            last_animation_state.0 = current_animation_state;
        }
    }
}
