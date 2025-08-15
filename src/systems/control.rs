use bevy_ecs::{
    event::{EventReader, EventWriter},
    system::{Query, ResMut},
};

use crate::{
    error::GameError,
    events::GameEvent,
    systems::{
        components::{GlobalState, PlayerControlled, Velocity},
        input::GameCommand,
    },
};

// Handles
pub fn player_system(
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
                    velocity.next_direction = Some((*direction, 90));
                }
                GameCommand::Exit => {
                    state.exit = true;
                }
                _ => {}
            },
        }
    }
}
