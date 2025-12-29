use std::mem::discriminant;
use tracing::{debug, info};

use crate::constants;
use crate::events::StageTransition;
use crate::map::direction::Direction;
use crate::systems::{EntityType, ItemCollider, SpawnTrigger, Velocity};
use crate::{
    map::builder::Map,
    systems::{
        AudioEvent, Blinking, DirectionalAnimation, Dying, Frozen, Ghost, GhostCollider, GhostState, LinearAnimation, Looping,
        NodeId, PlayerControlled, Position, Visibility,
    },
};
use bevy_ecs::{
    entity::Entity,
    event::{EventReader, EventWriter},
    query::{With, Without},
    resource::Resource,
    system::{Commands, Query, Res, ResMut, Single},
};

use crate::events::{GameCommand, GameEvent};
#[cfg(not(target_os = "emscripten"))]
use bevy_ecs::system::NonSendMut;
#[cfg(not(target_os = "emscripten"))]
use sdl2::render::Canvas;
#[cfg(not(target_os = "emscripten"))]
use sdl2::video::{FullscreenType, Window};

#[derive(Resource, Clone)]
pub struct PlayerAnimation(pub DirectionalAnimation);

#[derive(Resource, Clone)]
pub struct PlayerDeathAnimation(pub LinearAnimation);

/// Tracks whether the beginning sound has been played for the current startup sequence
#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct IntroPlayed(pub bool);

/// A resource to track the overall stage of the game from a high-level perspective.
#[derive(Resource, Debug, PartialEq, Eq, Clone, Copy)]
pub enum GameStage {
    /// Waiting for user interaction before starting (Emscripten only).
    /// Game is rendered but audio/gameplay are paused until the user clicks or presses a key.
    WaitingForInteraction,
    Starting(StartupSequence),
    /// The main gameplay loop is active.
    Playing,
    /// Short freeze after Pac-Man eats a ghost to display bonus score
    GhostEatenPause {
        remaining_ticks: u32,
        ghost_entity: Entity,
        ghost_type: Ghost,
        node: NodeId,
    },
    /// The player has died and the death sequence is in progress. At the end, the player will return to the startup sequence or game over.
    PlayerDying(DyingSequence),
    /// The game has ended.
    GameOver,
}

#[derive(Resource, Debug, PartialEq, Eq, Clone, Copy)]
pub enum PauseState {
    Inactive,
    Active { remaining_ticks: Option<u32> },
}

impl Default for PauseState {
    fn default() -> Self {
        Self::Inactive
    }
}

impl PauseState {
    pub fn active(&self) -> bool {
        matches!(
            self,
            PauseState::Active { remaining_ticks: None }
                | PauseState::Active {
                    remaining_ticks: Some(1..=u32::MAX)
                }
        )
    }

    /// Ticks the pause state
    /// # Returns
    /// `true` if the state changed significantly (e.g. from active to inactive or vice versa)
    pub fn tick(&mut self) -> bool {
        match self {
            // Permanent states
            PauseState::Active { remaining_ticks: None } | PauseState::Inactive => false,
            // Last tick of the active state
            PauseState::Active {
                remaining_ticks: Some(1),
            } => {
                *self = PauseState::Inactive;
                true
            }
            // Active state with remaining ticks
            PauseState::Active {
                remaining_ticks: Some(ticks),
            } => {
                *self = PauseState::Active {
                    remaining_ticks: Some(*ticks - 1),
                };
                false
            }
        }
    }
}

pub fn handle_pause_command(
    mut events: EventReader<GameEvent>,
    mut pause_state: ResMut<PauseState>,
    mut audio_events: EventWriter<AudioEvent>,
) {
    for event in events.read() {
        match event {
            GameEvent::Command(GameCommand::TogglePause) => {
                *pause_state = match *pause_state {
                    PauseState::Active { .. } => {
                        info!("Game resumed");
                        audio_events.write(AudioEvent::Resume);
                        PauseState::Inactive
                    }
                    PauseState::Inactive => {
                        info!("Game paused");
                        audio_events.write(AudioEvent::Pause);
                        PauseState::Active { remaining_ticks: None }
                    }
                }
            }
            GameEvent::Command(GameCommand::SingleTick) => {
                // Single tick should not function while the game is playing
                if matches!(*pause_state, PauseState::Active { remaining_ticks: None }) {
                    return;
                }

                *pause_state = PauseState::Active {
                    remaining_ticks: Some(1),
                };
                audio_events.write(AudioEvent::Resume);
            }
            _ => {}
        }
    }
}

pub fn manage_pause_state_system(mut pause_state: ResMut<PauseState>, mut audio_events: EventWriter<AudioEvent>) {
    let changed = pause_state.tick();

    // If the pause state changed, send the appropriate audio event
    if changed {
        // Since the pause state can never go from inactive to active, the only way to get here is if the game is now paused...
        audio_events.write(AudioEvent::Pause);
    }
}

#[cfg(not(target_os = "emscripten"))]
pub fn handle_fullscreen_command(mut events: EventReader<GameEvent>, mut canvas: NonSendMut<&mut Canvas<Window>>) {
    for event in events.read() {
        if let GameEvent::Command(GameCommand::ToggleFullscreen) = event {
            let window = canvas.window_mut();
            let current = window.fullscreen_state();
            let target = match current {
                FullscreenType::Off => FullscreenType::Desktop,
                _ => FullscreenType::Off,
            };

            if let Err(e) = window.set_fullscreen(target) {
                tracing::warn!(error = ?e, "Failed to toggle fullscreen");
            } else {
                let on = matches!(target, FullscreenType::Desktop | FullscreenType::True);
                info!(fullscreen = on, "Toggled fullscreen");
            }
        }
    }
}

pub trait TooSimilar {
    fn too_similar(&self, other: &Self) -> bool;
}

impl TooSimilar for GameStage {
    fn too_similar(&self, other: &Self) -> bool {
        discriminant(self) == discriminant(other) && {
            // These states are very simple, so they're 'too similar' automatically
            if matches!(self, GameStage::Playing | GameStage::GameOver | GameStage::WaitingForInteraction) {
                return true;
            }

            // Since the discriminant is the same but the values are different, it's the interior value that is somehow different
            match (self, other) {
                // These states are similar if their interior values are similar as well
                (GameStage::Starting(startup), GameStage::Starting(other)) => startup.too_similar(other),
                (GameStage::PlayerDying(dying), GameStage::PlayerDying(other)) => dying.too_similar(other),
                (
                    GameStage::GhostEatenPause {
                        ghost_entity,
                        ghost_type,
                        node,
                        ..
                    },
                    GameStage::GhostEatenPause {
                        ghost_entity: other_ghost_entity,
                        ghost_type: other_ghost_type,
                        node: other_node,
                        ..
                    },
                ) => ghost_entity == other_ghost_entity && ghost_type == other_ghost_type && node == other_node,
                // Already handled, but kept to properly exhaust the match
                (GameStage::Playing, _) | (GameStage::GameOver, _) | (GameStage::WaitingForInteraction, _) => unreachable!(),
                _ => unreachable!(),
            }
        }
    }
}

/// A resource that manages the multi-stage startup sequence of the game.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum StartupSequence {
    /// Stage 1: Text-only stage
    /// - Player & ghosts are hidden
    /// - READY! and PLAYER ONE text are shown
    /// - Energizers do not blink
    TextOnly {
        /// Remaining ticks in this stage
        remaining_ticks: u32,
    },
    /// Stage 2: Characters visible stage
    /// - PLAYER ONE text is hidden, READY! text remains
    /// - Ghosts and Pac-Man are now shown
    CharactersVisible {
        /// Remaining ticks in this stage
        remaining_ticks: u32,
    },
}

impl Default for GameStage {
    fn default() -> Self {
        Self::Playing
    }
}

impl TooSimilar for StartupSequence {
    fn too_similar(&self, other: &Self) -> bool {
        discriminant(self) == discriminant(other)
    }
}

/// The state machine for the multi-stage death sequence.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DyingSequence {
    /// Initial stage: entities are frozen, waiting for a delay.
    Frozen { remaining_ticks: u32 },
    /// Second stage: Pac-Man's death animation is playing.
    Animating { remaining_ticks: u32 },
    /// Third stage: Pac-Man is now gone, waiting a moment before the level restarts.
    Hidden { remaining_ticks: u32 },
}

impl TooSimilar for DyingSequence {
    fn too_similar(&self, other: &Self) -> bool {
        discriminant(self) == discriminant(other)
    }
}

/// A resource to store the number of player lives.
#[derive(Resource, Debug)]
pub struct PlayerLives(pub u8);

impl Default for PlayerLives {
    fn default() -> Self {
        Self(3)
    }
}

/// Handles startup sequence transitions and component management
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn stage_system(
    mut game_state: ResMut<GameStage>,
    player_death_animation: Res<PlayerDeathAnimation>,
    player_animation: Res<PlayerAnimation>,
    mut player_lives: ResMut<PlayerLives>,
    map: Res<Map>,
    mut commands: Commands,
    mut audio_events: EventWriter<AudioEvent>,
    mut stage_event_reader: EventReader<StageTransition>,
    mut blinking_query: Query<Entity, With<Blinking>>,
    player: Single<(Entity, &mut Position), With<PlayerControlled>>,
    mut item_query: Query<(Entity, &EntityType), With<ItemCollider>>,
    mut ghost_query: Query<(Entity, &Ghost, &mut Position, &mut GhostState), (With<GhostCollider>, Without<PlayerControlled>)>,
    mut intro_played: ResMut<IntroPlayed>,
) {
    let old_state = *game_state;
    let mut new_state_opt: Option<GameStage> = None;

    // Handle stage transition requests before normal ticking
    for event in stage_event_reader.read() {
        let StageTransition::GhostEatenPause {
            ghost_entity,
            ghost_type,
        } = *event;
        let pac_node = player.1.current_node();

        debug!(ghost = ?ghost_type, node = pac_node, "Ghost eaten, entering pause state");
        new_state_opt = Some(GameStage::GhostEatenPause {
            remaining_ticks: 30,
            ghost_entity,
            ghost_type,
            node: pac_node,
        });
    }

    let new_state: GameStage = new_state_opt.unwrap_or_else(|| match *game_state {
        GameStage::WaitingForInteraction => {
            // Stay in this state until JS calls start_game()
            *game_state
        }
        GameStage::Playing => {
            // This is the default state, do nothing
            *game_state
        }
        GameStage::GhostEatenPause {
            remaining_ticks,
            ghost_entity,
            ghost_type,
            node,
        } => {
            if remaining_ticks > 0 {
                GameStage::GhostEatenPause {
                    remaining_ticks: remaining_ticks.saturating_sub(1),
                    ghost_entity,
                    ghost_type,
                    node,
                }
            } else {
                debug!("Ghost eaten pause ended, resuming gameplay");
                GameStage::Playing
            }
        }
        GameStage::Starting(sequence) => match sequence {
            StartupSequence::TextOnly { remaining_ticks } => {
                // Play the beginning sound once at the start of TextOnly stage
                if !intro_played.0 {
                    audio_events.write(AudioEvent::PlaySound(crate::audio::Sound::Beginning));
                    intro_played.0 = true;
                }
                if remaining_ticks > 0 {
                    GameStage::Starting(StartupSequence::TextOnly {
                        remaining_ticks: remaining_ticks.saturating_sub(1),
                    })
                } else {
                    GameStage::Starting(StartupSequence::CharactersVisible { remaining_ticks: 60 })
                }
            }
            StartupSequence::CharactersVisible { remaining_ticks } => {
                if remaining_ticks > 0 {
                    GameStage::Starting(StartupSequence::CharactersVisible {
                        remaining_ticks: remaining_ticks.saturating_sub(1),
                    })
                } else {
                    info!("Startup sequence completed, beginning gameplay");
                    GameStage::Playing
                }
            }
        },
        GameStage::PlayerDying(sequence) => match sequence {
            DyingSequence::Frozen { remaining_ticks } => {
                if remaining_ticks > 0 {
                    GameStage::PlayerDying(DyingSequence::Frozen {
                        remaining_ticks: remaining_ticks.saturating_sub(1),
                    })
                } else {
                    let death_animation = &player_death_animation.0;
                    let remaining_ticks = (death_animation.tiles.len() * death_animation.frame_duration as usize) as u32;
                    debug!(animation_frames = remaining_ticks, "Starting player death animation");
                    GameStage::PlayerDying(DyingSequence::Animating { remaining_ticks })
                }
            }
            DyingSequence::Animating { remaining_ticks } => {
                if remaining_ticks > 0 {
                    GameStage::PlayerDying(DyingSequence::Animating {
                        remaining_ticks: remaining_ticks.saturating_sub(1),
                    })
                } else {
                    GameStage::PlayerDying(DyingSequence::Hidden { remaining_ticks: 60 })
                }
            }
            DyingSequence::Hidden { remaining_ticks } => {
                if remaining_ticks > 0 {
                    GameStage::PlayerDying(DyingSequence::Hidden {
                        remaining_ticks: remaining_ticks.saturating_sub(1),
                    })
                } else {
                    player_lives.0 = player_lives.0.saturating_sub(1);

                    if player_lives.0 > 0 {
                        info!(remaining_lives = player_lives.0, "Player died, returning to startup sequence");
                        GameStage::Starting(StartupSequence::CharactersVisible { remaining_ticks: 60 })
                    } else {
                        info!("All lives lost, game over");
                        GameStage::GameOver
                    }
                }
            }
        },
        GameStage::GameOver => *game_state,
    });

    if old_state == new_state {
        return;
    }

    if !old_state.too_similar(&new_state) {
        debug!(old_state = ?old_state, new_state = ?new_state, "Game stage transition");
    }

    match (old_state, new_state) {
        (GameStage::Playing, GameStage::GhostEatenPause { ghost_entity, node, .. }) => {
            // Freeze the player & non-eaten ghosts
            commands.entity(player.0).insert(Frozen);
            commands.entity(ghost_entity).insert(Frozen);
            for (entity, _, _, state) in ghost_query.iter_mut() {
                // Only freeze ghosts that are not currently eaten
                if *state != GhostState::Eyes {
                    debug!(ghost = ?entity, "Freezing ghost");
                    commands.entity(entity).insert(Frozen);
                }
            }

            // Hide the player & eaten ghost
            commands.entity(player.0).insert(Visibility::hidden());
            commands.entity(ghost_entity).insert(Visibility::hidden());

            // Spawn bonus points entity at Pac-Man's position
            commands.trigger(SpawnTrigger::Bonus {
                position: Position::Stopped { node },
                // TODO: Doubling score value for each consecutive ghost eaten
                value: 200,
                ttl: 30,
            });
        }
        (GameStage::GhostEatenPause { ghost_entity, .. }, GameStage::Playing) => {
            // Unfreeze and reveal the player & all ghosts
            commands.entity(player.0).remove::<Frozen>().insert(Visibility::visible());
            for (entity, _, _, _) in ghost_query.iter_mut() {
                commands.entity(entity).remove::<Frozen>().insert(Visibility::visible());
            }

            // Reveal the eaten ghost and switch it to Eyes state
            commands.entity(ghost_entity).insert(GhostState::Eyes);
        }
        (_, GameStage::PlayerDying(DyingSequence::Frozen { .. })) => {
            // Freeze the player & ghosts
            commands.entity(player.0).insert(Frozen);
            for (entity, _, _, _) in ghost_query.iter_mut() {
                commands.entity(entity).insert(Frozen);
            }
        }
        (GameStage::PlayerDying(DyingSequence::Frozen { .. }), GameStage::PlayerDying(DyingSequence::Animating { .. })) => {
            // Hide the ghosts
            for (entity, _, _, _) in ghost_query.iter_mut() {
                commands.entity(entity).insert(Visibility::hidden());
            }

            // Start Pac-Man's death animation
            commands
                .entity(player.0)
                .remove::<DirectionalAnimation>()
                .insert((Dying, player_death_animation.0.clone()));

            // Play the death sound
            audio_events.write(AudioEvent::PlaySound(crate::audio::Sound::PacmanDeath));
        }
        (_, GameStage::PlayerDying(DyingSequence::Hidden { .. })) => {
            // Pac-Man's death animation is complete, so he should be hidden just like the ghosts.
            // Then, we reset them all back to their original positions and states.

            // Freeze the blinking power pellets, force them to be visible (if they were hidden by blinking)
            for entity in blinking_query.iter_mut() {
                commands.entity(entity).insert(Frozen).insert(Visibility::visible());
            }

            // Delete any fruit entities
            for (entity, _) in item_query
                .iter_mut()
                .filter(|(_, entity_type)| matches!(entity_type, EntityType::Fruit(_)))
            {
                commands.entity(entity).despawn();
            }

            // Reset the player animation
            commands
                .entity(player.0)
                .remove::<(Dying, LinearAnimation, Looping)>()
                .insert((
                    Velocity {
                        speed: constants::mechanics::PLAYER_SPEED,
                        direction: Direction::Left,
                    },
                    Position::Stopped {
                        node: map.start_positions.pacman,
                    },
                    player_animation.0.clone(),
                    Visibility::hidden(),
                    Frozen,
                ));

            // Reset ghost positions and state
            for (ghost_entity, ghost, _, _) in ghost_query.iter_mut() {
                commands.entity(ghost_entity).insert((
                    GhostState::Normal,
                    Position::Stopped {
                        node: match ghost {
                            Ghost::Blinky => map.start_positions.blinky,
                            Ghost::Pinky => map.start_positions.pinky,
                            Ghost::Inky => map.start_positions.inky,
                            Ghost::Clyde => map.start_positions.clyde,
                        },
                    },
                    Frozen,
                    Visibility::hidden(),
                ));
            }
        }
        (_, GameStage::Starting(StartupSequence::CharactersVisible { .. })) => {
            // Unhide the player & ghosts
            commands.entity(player.0).insert(Visibility::visible());
            for (entity, _, _, _) in ghost_query.iter_mut() {
                commands.entity(entity).insert(Visibility::visible());
            }
        }
        (GameStage::Starting(StartupSequence::CharactersVisible { .. }), GameStage::Playing) => {
            // Unfreeze the player & ghosts & blinking
            commands.entity(player.0).remove::<Frozen>();
            for (entity, _, _, _) in ghost_query.iter_mut() {
                commands.entity(entity).remove::<Frozen>();
            }
            for entity in blinking_query.iter_mut() {
                commands.entity(entity).remove::<Frozen>();
            }
            // Reset intro flag for the next round
            intro_played.0 = false;
        }
        (_, GameStage::GameOver) => {
            // Freeze blinking
            for entity in blinking_query.iter_mut() {
                commands.entity(entity).insert(Frozen);
            }
        }
        _ => {}
    }

    *game_state = new_state;
}
