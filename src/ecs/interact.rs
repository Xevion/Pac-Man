use bevy_ecs::{
    event::{EventReader, EventWriter},
    query::With,
    system::{Query, ResMut},
};

use crate::{
    ecs::{GlobalState, PlayerControlled, Velocity},
    error::GameError,
    game::events::GameEvent,
    input::commands::GameCommand,
};

// Handles
pub fn interact_system(
    mut events: EventReader<GameEvent>,
    mut state: ResMut<GlobalState>,
    mut players: Query<(&PlayerControlled, &mut Velocity)>,
    mut errors: EventWriter<GameError>,
) {
    // Get the player's velocity (handling to ensure there is only one player)
    let mut velocity = match players.single_mut() {
        Ok((_, velocity)) => velocity,
        Err(e) => {
            errors.write(GameError::InvalidState(format!("Player not found: {}", e)).into());
            return;
        }
    };

    // Handle events
    for event in events.read() {
        match event {
            GameEvent::Command(command) => match command {
                GameCommand::MovePlayer(direction) => {
                    velocity.direction = *direction;
                }
                GameCommand::Exit => {
                    state.exit = true;
                }
                _ => {}
            },
        }
    }
}
