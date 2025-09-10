use crate::texture::animated::TileSequence;
use bevy_ecs::component::Component;
use bevy_ecs::resource::Resource;

/// Tag component to mark animations that should loop when they reach the end
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Looping;

/// Linear animation component for non-directional animations (frightened ghosts)
#[derive(Component, Resource, Clone)]
pub struct LinearAnimation {
    pub tiles: TileSequence,
    pub current_frame: usize,
    pub time_bank: u16,
    pub frame_duration: u16,
    pub finished: bool,
}

impl LinearAnimation {
    /// Creates a new linear animation with the given tiles and frame duration
    pub fn new(tiles: TileSequence, frame_duration: u16) -> Self {
        Self {
            tiles,
            current_frame: 0,
            time_bank: 0,
            frame_duration,
            finished: false,
        }
    }
}
