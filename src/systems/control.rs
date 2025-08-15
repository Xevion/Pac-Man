use bevy_ecs::{
    event::{EventReader, EventWriter},
    prelude::ResMut,
    query::With,
    system::Query,
};

use crate::{
    error::GameError,
    events::{GameCommand, GameEvent},
    systems::components::{GlobalState, PlayerControlled},
    systems::movement::Movable,
};

// Handles player input and control
pub fn player_system(
    mut events: EventReader<GameEvent>,
    mut state: ResMut<GlobalState>,
    mut players: Query<&mut Movable, With<PlayerControlled>>,
    mut errors: EventWriter<GameError>,
) {
    // Get the player's movable component (ensuring there is only one player)
    let mut movable = match players.single_mut() {
        Ok(movable) => movable,
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
                    movable.requested_direction = Some(*direction);
                }
                GameCommand::Exit => {
                    state.exit = true;
                }
                _ => {}
            },
            GameEvent::Collision(a, b) => {
                tracing::info!("Collision between {:?} and {:?}", a, b);
            }
        }
    }
}
