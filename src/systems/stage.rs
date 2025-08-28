use bevy_ecs::{
    prelude::{Commands, Entity, Query, With},
    system::ResMut,
};

use crate::systems::components::{Frozen, GhostCollider, PlayerControlled, StartupSequence};

/// Handles startup sequence transitions and component management
pub fn startup_stage_system(
    mut startup: ResMut<StartupSequence>,
    mut commands: Commands,
    mut player_query: Query<Entity, With<PlayerControlled>>,
    mut ghost_query: Query<Entity, With<GhostCollider>>,
) {
    if let Some((from, to)) = startup.tick() {
        match (from, to) {
            (StartupSequence::TextOnly { .. }, StartupSequence::CharactersVisible { .. }) => {
                // TODO: Add TextOnly tag component to hide entities
                // TODO: Add CharactersVisible tag component to show entities
                // TODO: Remove TextOnly tag component
            }
            (StartupSequence::CharactersVisible { .. }, StartupSequence::GameActive) => {
                // Remove Frozen tag from all entities
                for entity in player_query.iter_mut().chain(ghost_query.iter_mut()) {
                    commands.entity(entity).remove::<Frozen>();
                }
                // TODO: Add GameActive tag component
                // TODO: Remove CharactersVisible tag component
            }
            _ => {}
        }
    }
}
