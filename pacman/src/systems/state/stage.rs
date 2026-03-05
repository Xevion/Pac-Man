use tracing::{debug, info};

use crate::constants;
use crate::events::StageTransition;
use crate::map::builder::Map;
use crate::map::direction::Direction;
use crate::systems::animation::{Blinking, DirectionalAnimation, Dying, LinearAnimation, Looping};
use crate::systems::audio::AudioEvent;
use crate::systems::collision::{GhostCollider, ItemCollider};
use crate::systems::common::{EntityType, Frozen};
use crate::systems::ghost::{GhostState, GhostType};
use crate::systems::item::SpawnTrigger;
use crate::systems::movement::{NodeId, Position, Velocity};
use crate::systems::player::PlayerControlled;
use crate::systems::render::Visibility;
use bevy_ecs::{
    entity::Entity,
    event::{EventReader, EventWriter},
    query::{With, Without},
    resource::Resource,
    system::{Commands, Query, Res, ResMut, Single, SystemParam},
};

use super::{IntroPlayed, PlayerAnimation, PlayerDeathAnimation, PlayerLives, TooSimilar};

/// A resource to track the overall stage of the game from a high-level perspective.
#[derive(Resource, Debug, PartialEq, Eq, Clone, Copy)]
pub enum GameStage {
    /// Waiting for user interaction before starting (Emscripten only).
    /// Game is rendered but audio/gameplay are paused until the user clicks or presses a key.
    #[cfg(target_os = "emscripten")]
    WaitingForInteraction,
    Starting(StartupSequence),
    /// The main gameplay loop is active.
    Playing,
    /// Short freeze after Pac-Man eats a ghost to display bonus score
    GhostEatenPause {
        remaining_ticks: u32,
        ghost_entity: Entity,
        ghost_type: GhostType,
        node: NodeId,
    },
    /// The player has died and the death sequence is in progress. At the end, the player will return to the startup sequence or game over.
    PlayerDying(DyingSequence),
    /// The game has ended.
    GameOver,
}

impl Default for GameStage {
    fn default() -> Self {
        Self::Playing
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

/// Grouped resources for the stage system.
#[derive(SystemParam)]
pub struct StageResources<'w, 's> {
    pub game_state: ResMut<'w, GameStage>,
    pub player_death_animation: Res<'w, PlayerDeathAnimation>,
    pub player_animation: Res<'w, PlayerAnimation>,
    pub player_lives: ResMut<'w, PlayerLives>,
    pub map: Res<'w, Map>,
    pub intro_played: ResMut<'w, IntroPlayed>,
    pub commands: Commands<'w, 's>,
    pub audio_events: EventWriter<'w, AudioEvent>,
    pub stage_event_reader: EventReader<'w, 's, StageTransition>,
}

/// Handles startup sequence transitions and component management
#[allow(clippy::type_complexity)]
pub fn stage_system(
    mut res: StageResources,
    mut blinking_query: Query<Entity, With<Blinking>>,
    player: Single<(Entity, &mut Position), With<PlayerControlled>>,
    mut item_query: Query<(Entity, &EntityType), With<ItemCollider>>,
    mut ghost_query: Query<
        (Entity, &GhostType, &mut Position, &mut GhostState),
        (With<GhostCollider>, Without<PlayerControlled>),
    >,
) {
    let old_state = *res.game_state;
    let mut new_state_opt: Option<GameStage> = None;

    // Handle stage transition requests before normal ticking
    for event in res.stage_event_reader.read() {
        let StageTransition::GhostEatenPause {
            ghost_entity,
            ghost_type,
        } = *event;
        let pac_node = player.1.current_node();

        debug!(ghost = ?ghost_type, node = pac_node, "Ghost eaten, entering pause state");
        new_state_opt = Some(GameStage::GhostEatenPause {
            remaining_ticks: constants::mechanics::GHOST_EATEN_PAUSE_TICKS,
            ghost_entity,
            ghost_type,
            node: pac_node,
        });
    }

    let new_state: GameStage = new_state_opt.unwrap_or_else(|| match *res.game_state {
        #[cfg(target_os = "emscripten")]
        GameStage::WaitingForInteraction => {
            // Stay in this state until JS calls start_game()
            *res.game_state
        }
        GameStage::Playing => {
            // This is the default state, do nothing
            *res.game_state
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
                if !res.intro_played.0 {
                    res.audio_events.write(AudioEvent::PlaySound(crate::audio::Sound::Beginning));
                    res.intro_played.0 = true;
                }
                if remaining_ticks > 0 {
                    GameStage::Starting(StartupSequence::TextOnly {
                        remaining_ticks: remaining_ticks.saturating_sub(1),
                    })
                } else {
                    GameStage::Starting(StartupSequence::CharactersVisible {
                        remaining_ticks: constants::startup::CHARACTERS_VISIBLE_TICKS,
                    })
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
                    let death_animation = &res.player_death_animation.0;
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
                    GameStage::PlayerDying(DyingSequence::Hidden {
                        remaining_ticks: constants::mechanics::DEATH_HIDDEN_TICKS,
                    })
                }
            }
            DyingSequence::Hidden { remaining_ticks } => {
                if remaining_ticks > 0 {
                    GameStage::PlayerDying(DyingSequence::Hidden {
                        remaining_ticks: remaining_ticks.saturating_sub(1),
                    })
                } else {
                    res.player_lives.lose_life();

                    if res.player_lives.is_alive() {
                        info!(
                            remaining_lives = res.player_lives.remaining(),
                            "Player died, returning to startup sequence"
                        );
                        GameStage::Starting(StartupSequence::CharactersVisible {
                            remaining_ticks: constants::startup::CHARACTERS_VISIBLE_TICKS,
                        })
                    } else {
                        info!("All lives lost, game over");
                        GameStage::GameOver
                    }
                }
            }
        },
        GameStage::GameOver => *res.game_state,
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
            res.commands.entity(player.0).insert(Frozen);
            res.commands.entity(ghost_entity).insert(Frozen);
            for (entity, _, _, state) in ghost_query.iter_mut() {
                // Only freeze ghosts that are not currently eaten
                if *state != GhostState::Eyes {
                    debug!(ghost = ?entity, "Freezing ghost");
                    res.commands.entity(entity).insert(Frozen);
                }
            }

            // Hide the player & eaten ghost
            res.commands.entity(player.0).insert(Visibility::hidden());
            res.commands.entity(ghost_entity).insert(Visibility::hidden());

            // Spawn bonus points entity at Pac-Man's position
            res.commands.trigger(SpawnTrigger::Bonus {
                position: Position::Stopped { node },
                // TODO: Doubling score value for each consecutive ghost eaten
                value: constants::mechanics::GHOST_EATEN_SCORE,
                ttl: constants::mechanics::GHOST_EATEN_PAUSE_TICKS,
            });
        }
        (GameStage::GhostEatenPause { ghost_entity, .. }, GameStage::Playing) => {
            // Unfreeze and reveal the player & all ghosts
            res.commands.entity(player.0).remove::<Frozen>().insert(Visibility::visible());
            for (entity, _, _, _) in ghost_query.iter_mut() {
                res.commands.entity(entity).remove::<Frozen>().insert(Visibility::visible());
            }

            // Reveal the eaten ghost and switch it to Eyes state
            res.commands.entity(ghost_entity).insert(GhostState::Eyes);
        }
        (_, GameStage::PlayerDying(DyingSequence::Frozen { .. })) => {
            // Freeze the player & ghosts
            res.commands.entity(player.0).insert(Frozen);
            for (entity, _, _, _) in ghost_query.iter_mut() {
                res.commands.entity(entity).insert(Frozen);
            }
        }
        (GameStage::PlayerDying(DyingSequence::Frozen { .. }), GameStage::PlayerDying(DyingSequence::Animating { .. })) => {
            // Hide the ghosts
            for (entity, _, _, _) in ghost_query.iter_mut() {
                res.commands.entity(entity).insert(Visibility::hidden());
            }

            // Start Pac-Man's death animation
            res.commands
                .entity(player.0)
                .remove::<DirectionalAnimation>()
                .insert((Dying, res.player_death_animation.0.clone()));

            // Play the death sound
            res.audio_events
                .write(AudioEvent::PlaySound(crate::audio::Sound::PacmanDeath));
        }
        (_, GameStage::PlayerDying(DyingSequence::Hidden { .. })) => {
            // Pac-Man's death animation is complete, so he should be hidden just like the ghosts.
            // Then, we reset them all back to their original positions and states.

            // Freeze the blinking power pellets, force them to be visible (if they were hidden by blinking)
            for entity in blinking_query.iter_mut() {
                res.commands.entity(entity).insert(Frozen).insert(Visibility::visible());
            }

            // Delete any fruit entities
            for (entity, _) in item_query
                .iter_mut()
                .filter(|(_, entity_type)| matches!(entity_type, EntityType::Fruit(_)))
            {
                res.commands.entity(entity).despawn();
            }

            // Reset the player animation
            res.commands
                .entity(player.0)
                .remove::<(Dying, LinearAnimation, Looping)>()
                .insert((
                    Velocity {
                        speed: constants::mechanics::PLAYER_SPEED,
                        direction: Direction::Left,
                    },
                    Position::Stopped {
                        node: res.map.start_positions.pacman,
                    },
                    res.player_animation.0.clone(),
                    Visibility::hidden(),
                    Frozen,
                ));

            // Reset ghost positions and state
            for (ghost_entity, ghost, _, _) in ghost_query.iter_mut() {
                // Blinky starts active outside the house, others start in house
                let ghost_state = if *ghost == GhostType::Blinky {
                    GhostState::Active { frightened: None }
                } else {
                    GhostState::InHouse {
                        position: crate::systems::ghost::state::HousePosition::Center,
                        bounce: crate::systems::ghost::state::BounceDirection::Up,
                    }
                };

                res.commands.entity(ghost_entity).insert((
                    ghost_state,
                    Position::Stopped {
                        node: match ghost {
                            GhostType::Blinky => res.map.start_positions.blinky,
                            GhostType::Pinky => res.map.start_positions.pinky,
                            GhostType::Inky => res.map.start_positions.inky,
                            GhostType::Clyde => res.map.start_positions.clyde,
                        },
                    },
                    Frozen,
                    Visibility::hidden(),
                ));
            }
        }
        (_, GameStage::Starting(StartupSequence::CharactersVisible { .. })) => {
            // Unhide the player & ghosts
            res.commands.entity(player.0).insert(Visibility::visible());
            for (entity, _, _, _) in ghost_query.iter_mut() {
                res.commands.entity(entity).insert(Visibility::visible());
            }
        }
        (GameStage::Starting(StartupSequence::CharactersVisible { .. }), GameStage::Playing) => {
            // Unfreeze the player & ghosts & blinking
            res.commands.entity(player.0).remove::<Frozen>();
            for (entity, _, _, _) in ghost_query.iter_mut() {
                res.commands.entity(entity).remove::<Frozen>();
            }
            for entity in blinking_query.iter_mut() {
                res.commands.entity(entity).remove::<Frozen>();
            }
            // Reset intro flag for the next round
            res.intro_played.0 = false;
        }
        (_, GameStage::GameOver) => {
            // Freeze blinking
            for entity in blinking_query.iter_mut() {
                res.commands.entity(entity).insert(Frozen);
            }
        }
        _ => {}
    }

    *res.game_state = new_state;
}
