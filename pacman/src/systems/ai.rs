//! Stub attract-mode AI for Pac-Man.
//!
//! Steers Pac-Man toward the nearest remaining pellet so the attract scene plays
//! itself. It emits the very same `GameCommand::MovePlayer` a human would, so
//! `player_control_system` stays the single, source-agnostic consumer; the only
//! thing distinguishing AI play from human play is which producer is enabled (see
//! [`InputSource`](crate::systems::input::InputSource)).

use bevy_ecs::{
    event::EventWriter,
    query::{With, Without},
    system::{Query, Res, Single},
};

use crate::events::{GameCommand, GameEvent};
use crate::map::builder::Map;
use crate::map::graph::TraversalFlags;
use crate::systems::collision::ItemCollider;
use crate::systems::common::Frozen;
use crate::systems::ghost::targeting::choose_direction_at_intersection;
use crate::systems::movement::{NodeId, Position, Velocity};
use crate::systems::player::PlayerControlled;

/// Drives Pac-Man toward the nearest remaining pellet. Runs only under
/// `InputSource::Ai` (the attract scene); skips while the player is frozen (intro)
/// or absent, so there is nothing to steer until play actually begins.
#[allow(clippy::type_complexity)]
pub fn ai_player_system(
    map: Res<Map>,
    player: Option<Single<(&Position, &Velocity), (With<PlayerControlled>, Without<Frozen>)>>,
    pellets: Query<&Position, With<ItemCollider>>,
    mut writer: EventWriter<GameEvent>,
) {
    // Nothing to steer until there is exactly one movable player. The player spawns
    // `Frozen` and stays so through the intro sequence (and again during death and the
    // ghost-eaten pause), so the query is routinely empty -- the AI just waits. This
    // must be `Option<Single>`, not `Single`: the schedule runs every system through
    // `profile()`, which calls `System::run` directly and bypasses Bevy's param
    // validation, so a bare `Single` panics on the empty frame instead of being skipped.
    let Some(player) = player else {
        return;
    };
    let (position, velocity) = player.into_inner();
    let current_node = position.current_node();

    // With no pellets left there is nothing to chase; let momentum carry on.
    let Some(target_node) = nearest_pellet_node(&map, current_node, &pellets) else {
        return;
    };

    // Reuse the ghost intersection logic with Pac-Man's traversal flags and no
    // red-zone restriction: a greedy, deterministic shortest-step-to-target chooser.
    if let Some(direction) = choose_direction_at_intersection(
        &map,
        None,
        TraversalFlags::PACMAN,
        current_node,
        target_node,
        velocity.direction,
        false,
        None,
    ) {
        writer.write(GameEvent::Command(GameCommand::MovePlayer(direction)));
    }
}

/// Returns the graph node of the pellet nearest `from` by straight-line distance,
/// or `None` when no pellets remain.
fn nearest_pellet_node(map: &Map, from: NodeId, pellets: &Query<&Position, With<ItemCollider>>) -> Option<NodeId> {
    let origin = map.graph.get_node(from)?.position;
    pellets
        .iter()
        .map(Position::current_node)
        .filter_map(|node| map.graph.get_node(node).map(|n| (node, n.position.distance_squared(origin))))
        .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(node, _)| node)
}
