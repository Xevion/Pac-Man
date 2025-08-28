use bevy_ecs::{bundle::Bundle, component::Component, resource::Resource};
use bitflags::bitflags;

use crate::{
    map::graph::TraversalFlags,
    systems::movement::{BufferedDirection, Position, Velocity},
    texture::{animated::AnimatedTexture, sprite::AtlasTile},
};

/// A tag component for entities that are controlled by the player.
#[derive(Default, Component)]
pub struct PlayerControlled;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
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

    /// Returns the ghost's color for debug rendering.
    #[allow(dead_code)]
    pub fn debug_color(&self) -> sdl2::pixels::Color {
        match self {
            Ghost::Blinky => sdl2::pixels::Color::RGB(255, 0, 0),    // Red
            Ghost::Pinky => sdl2::pixels::Color::RGB(255, 182, 255), // Pink
            Ghost::Inky => sdl2::pixels::Color::RGB(0, 255, 255),    // Cyan
            Ghost::Clyde => sdl2::pixels::Color::RGB(255, 182, 85),  // Orange
        }
    }
}

/// A tag component denoting the type of entity.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityType {
    Player,
    Ghost,
    Pellet,
    PowerPellet,
}

impl EntityType {
    /// Returns the traversal flags for this entity type.
    pub fn traversal_flags(&self) -> TraversalFlags {
        match self {
            EntityType::Player => TraversalFlags::PACMAN,
            EntityType::Ghost => TraversalFlags::GHOST,
            _ => TraversalFlags::empty(), // Static entities don't traverse
        }
    }
    pub fn score_value(&self) -> Option<u32> {
        match self {
            EntityType::Pellet => Some(10),
            EntityType::PowerPellet => Some(50),
            _ => None,
        }
    }

    pub fn is_collectible(&self) -> bool {
        matches!(self, EntityType::Pellet | EntityType::PowerPellet)
    }
}

/// A component for entities that have a sprite, with a layer for ordering.
///
/// This is intended to be modified by other entities allowing animation.
#[derive(Component)]
pub struct Renderable {
    pub sprite: AtlasTile,
    pub layer: u8,
    pub visible: bool,
}

/// A component for entities that have a directional animated texture.
#[derive(Component)]
pub struct DirectionalAnimated {
    pub textures: [Option<AnimatedTexture>; 4],
    pub stopped_textures: [Option<AnimatedTexture>; 4],
}

bitflags! {
    #[derive(Component, Default, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct CollisionLayer: u8 {
        const PACMAN = 1 << 0;
        const GHOST = 1 << 1;
        const ITEM = 1 << 2;
    }
}

#[derive(Component)]
pub struct Collider {
    pub size: f32,
}

impl Collider {
    /// Checks if this collider collides with another collider at the given distance.
    pub fn collides_with(&self, other_size: f32, distance: f32) -> bool {
        let collision_distance = (self.size + other_size) / 2.0;
        distance < collision_distance
    }
}

/// Marker components for collision filtering optimization
#[derive(Component)]
pub struct PacmanCollider;

#[derive(Component)]
pub struct GhostCollider;

#[derive(Component)]
pub struct ItemCollider;

#[derive(Bundle)]
pub struct PlayerBundle {
    pub player: PlayerControlled,
    pub position: Position,
    pub velocity: Velocity,
    pub buffered_direction: BufferedDirection,
    pub sprite: Renderable,
    pub directional_animated: DirectionalAnimated,
    pub entity_type: EntityType,
    pub collider: Collider,
    pub pacman_collider: PacmanCollider,
}

#[derive(Bundle)]
pub struct ItemBundle {
    pub position: Position,
    pub sprite: Renderable,
    pub entity_type: EntityType,
    pub collider: Collider,
    pub item_collider: ItemCollider,
}

#[derive(Bundle)]
pub struct GhostBundle {
    pub ghost: Ghost,
    pub position: Position,
    pub velocity: Velocity,
    pub sprite: Renderable,
    pub directional_animated: DirectionalAnimated,
    pub entity_type: EntityType,
    pub collider: Collider,
    pub ghost_collider: GhostCollider,
}

#[derive(Resource)]
pub struct GlobalState {
    pub exit: bool,
}

#[derive(Resource)]
pub struct ScoreResource(pub u32);

#[derive(Resource)]
pub struct DeltaTime(pub f32);

/// Resource for tracking audio state
#[derive(Resource, Debug, Clone, Default)]
pub struct AudioState {
    /// Whether audio is currently muted
    pub muted: bool,
    /// Current sound index for cycling through eat sounds
    pub sound_index: usize,
}

/// Lifecycle state for the player entity.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerLifecycle {
    Spawning,
    Alive,
    Dying,
    Respawning,
}

impl PlayerLifecycle {
    /// Returns true when gameplay input and movement should be active
    pub fn is_interactive(self) -> bool {
        matches!(self, PlayerLifecycle::Alive)
    }
}

impl Default for PlayerLifecycle {
    fn default() -> Self {
        PlayerLifecycle::Spawning
    }
}

/// Whether player input should be processed.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlState {
    InputEnabled,
    InputLocked,
}

impl Default for ControlState {
    fn default() -> Self {
        Self::InputLocked
    }
}

/// Combat-related state for Pac-Man. Tick-based energizer logic.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatState {
    Normal,
    Energized {
        /// Remaining energizer duration in ticks (frames)
        remaining_ticks: u32,
        /// Ticks until flashing begins (counts down to 0, then flashing is active)
        flash_countdown_ticks: u32,
    },
}

impl Default for CombatState {
    fn default() -> Self {
        CombatState::Normal
    }
}

impl CombatState {
    pub fn is_energized(&self) -> bool {
        matches!(self, CombatState::Energized { .. })
    }

    pub fn is_flashing(&self) -> bool {
        matches!(self, CombatState::Energized { flash_countdown_ticks, .. } if *flash_countdown_ticks == 0)
    }

    pub fn deactivate_energizer(&mut self) {
        *self = CombatState::Normal;
    }

    /// Activate energizer using tick-based durations.
    pub fn activate_energizer_ticks(&mut self, total_ticks: u32, flash_lead_ticks: u32) {
        let flash_countdown_ticks = total_ticks.saturating_sub(flash_lead_ticks);
        *self = CombatState::Energized {
            remaining_ticks: total_ticks,
            flash_countdown_ticks,
        };
    }

    /// Advance one frame. When ticks reach zero, returns to Normal.
    pub fn tick_frame(&mut self) {
        if let CombatState::Energized {
            remaining_ticks,
            flash_countdown_ticks,
        } = self
        {
            if *remaining_ticks > 0 {
                *remaining_ticks -= 1;
                if *flash_countdown_ticks > 0 {
                    *flash_countdown_ticks -= 1;
                }
            }
            if *remaining_ticks == 0 {
                *self = CombatState::Normal;
            }
        }
    }
}

/// Movement modifiers that can affect Pac-Man's speed or handling.
#[derive(Component, Debug, Clone, Copy)]
pub struct MovementModifiers {
    /// Multiplier applied to base speed (e.g., tunnels)
    pub speed_multiplier: f32,
    /// True when currently in a tunnel slowdown region
    pub tunnel_slowdown_active: bool,
}

impl Default for MovementModifiers {
    fn default() -> Self {
        Self {
            speed_multiplier: 1.0,
            tunnel_slowdown_active: false,
        }
    }
}

/// Level-dependent timing configuration
#[derive(Resource, Debug, Clone, Copy)]
pub struct LevelTiming {
    /// Duration of energizer effect in seconds
    pub energizer_duration: f32,
    /// Freeze duration at spawn/ready in seconds
    pub spawn_freeze_duration: f32,
    /// When to start flashing relative to energizer end (seconds)
    pub energizer_flash_threshold: f32,
}

impl Default for LevelTiming {
    fn default() -> Self {
        Self {
            energizer_duration: 6.0,
            spawn_freeze_duration: 1.5,
            energizer_flash_threshold: 2.0,
        }
    }
}

impl LevelTiming {
    /// Returns timing configuration for a given level.
    pub fn for_level(_level: u32) -> Self {
        // Placeholder: tune per the Pac-Man Dossier tables
        Self::default()
    }
}

/// Tag component for entities that should be frozen during startup
#[derive(Component, Debug, Clone, Copy)]
pub struct Frozen;

/// Convenience bundle for attaching the hybrid FSM to the player entity
#[derive(Bundle, Default)]
pub struct PlayerStateBundle {
    pub lifecycle: PlayerLifecycle,
    pub control: ControlState,
    pub combat: CombatState,
    pub movement_modifiers: MovementModifiers,
}

#[derive(Resource, Debug, Clone, Copy)]
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
    /// Stage 3: Game begins
    /// - Final state, game is fully active
    GameActive,
}

impl StartupSequence {
    /// Creates a new StartupSequence with the specified duration in ticks
    pub fn new(text_only_ticks: u32, _characters_visible_ticks: u32) -> Self {
        Self::TextOnly {
            remaining_ticks: text_only_ticks,
        }
    }

    /// Returns true if the timer is still active (not in GameActive state)
    pub fn is_active(&self) -> bool {
        !matches!(self, StartupSequence::GameActive)
    }

    /// Returns true if we're in the game active stage
    pub fn is_game_active(&self) -> bool {
        matches!(self, StartupSequence::GameActive)
    }

    /// Ticks the timer by one frame, returning transition information if state changes
    pub fn tick(&mut self) -> Option<(StartupSequence, StartupSequence)> {
        match self {
            StartupSequence::TextOnly { remaining_ticks } => {
                if *remaining_ticks > 0 {
                    *remaining_ticks -= 1;
                    None
                } else {
                    let from = *self;
                    *self = StartupSequence::CharactersVisible {
                        remaining_ticks: 60, // 1 second at 60 FPS
                    };
                    Some((from, *self))
                }
            }
            StartupSequence::CharactersVisible { remaining_ticks } => {
                if *remaining_ticks > 0 {
                    *remaining_ticks -= 1;
                    None
                } else {
                    let from = *self;
                    *self = StartupSequence::GameActive;
                    Some((from, *self))
                }
            }
            StartupSequence::GameActive => None,
        }
    }
}
