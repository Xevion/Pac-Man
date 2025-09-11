use bevy_ecs::{
    component::Component,
    query::{Has, Or, With, Without},
    system::{Query, Res},
};

use crate::{
    systems::{DeltaTime, Dying, Frozen, LinearAnimation, Looping, Position, Renderable, Velocity},
    texture::animated::DirectionalTiles,
};

/// Directional animation component with shared timing across all directions
#[derive(Component, Clone)]
pub struct DirectionalAnimation {
    pub moving_tiles: DirectionalTiles,
    pub stopped_tiles: DirectionalTiles,
    pub current_frame: usize,
    pub time_bank: u16,
    pub frame_duration: u16,
}

impl DirectionalAnimation {
    /// Creates a new directional animation with the given tiles and frame duration
    pub fn new(moving_tiles: DirectionalTiles, stopped_tiles: DirectionalTiles, frame_duration: u16) -> Self {
        Self {
            moving_tiles,
            stopped_tiles,
            current_frame: 0,
            time_bank: 0,
            frame_duration,
        }
    }
}

/// Updates directional animated entities with synchronized timing across directions.
///
/// This runs before the render system to update sprites based on current direction and movement state.
/// All directions share the same frame timing to ensure perfect synchronization.
pub fn directional_render_system(
    dt: Res<DeltaTime>,
    mut query: Query<(&Position, &Velocity, &mut DirectionalAnimation, &mut Renderable, Has<Frozen>)>,
) {
    let ticks = (dt.seconds * 60.0).round() as u16; // Convert from seconds to ticks at 60 ticks/sec

    for (position, velocity, mut anim, mut renderable, frozen) in query.iter_mut() {
        let stopped = matches!(position, Position::Stopped { .. });

        // Only tick animation when moving to preserve stopped frame
        if !stopped && !frozen {
            // Tick shared animation state
            anim.time_bank += ticks;
            while anim.time_bank >= anim.frame_duration {
                anim.time_bank -= anim.frame_duration;
                anim.current_frame += 1;
            }
        }

        // Get tiles for current direction and movement state
        let tiles = if stopped {
            anim.stopped_tiles.get(velocity.direction)
        } else {
            anim.moving_tiles.get(velocity.direction)
        };

        if !tiles.is_empty() {
            let new_tile = tiles.get_tile(anim.current_frame);
            if renderable.sprite != new_tile {
                renderable.sprite = new_tile;
            }
        }
    }
}

/// System that updates `Renderable` sprites for entities with `LinearAnimation`.
#[allow(clippy::type_complexity)]
pub fn linear_render_system(
    dt: Res<DeltaTime>,
    mut query: Query<(&mut LinearAnimation, &mut Renderable, Has<Looping>), Or<(Without<Frozen>, With<Dying>)>>,
) {
    for (mut anim, mut renderable, looping) in query.iter_mut() {
        if anim.finished {
            continue;
        }

        anim.time_bank += dt.ticks as u16;
        let frames_to_advance = (anim.time_bank / anim.frame_duration) as usize;

        if frames_to_advance == 0 {
            continue;
        }

        let total_frames = anim.tiles.len();

        if !looping && anim.current_frame + frames_to_advance >= total_frames {
            anim.finished = true;
            anim.current_frame = total_frames - 1;
        } else {
            anim.current_frame += frames_to_advance;
        }

        anim.time_bank %= anim.frame_duration;
        renderable.sprite = anim.tiles.get_tile(anim.current_frame);
    }
}
