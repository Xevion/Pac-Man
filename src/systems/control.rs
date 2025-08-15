use bevy_ecs::{
    event::{EventReader, EventWriter},
    query::With,
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
    mut players: Query<&mut Velocity, With<PlayerControlled>>,
    mut errors: EventWriter<GameError>,
) {
    // Get the player's velocity (handling to ensure there is only one player)
    let mut velocity = match players.single_mut() {
        Ok(velocity) => velocity,
        Err(e) => {
            errors.write(GameError::InvalidState(format!("No/multiple entities queried for player system: {}", e)).into());
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
