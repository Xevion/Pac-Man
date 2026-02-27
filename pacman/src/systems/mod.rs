//! This module contains all the systems in the game.

// These modules are excluded from coverage.
#[cfg_attr(coverage_nightly, coverage(off))]
pub mod audio;
#[cfg_attr(coverage_nightly, coverage(off))]
pub mod debug;
#[cfg_attr(coverage_nightly, coverage(off))]
pub mod profiling;
#[cfg_attr(coverage_nightly, coverage(off))]
pub mod render;

pub mod animation;
pub mod collision;
pub mod common;
pub mod ghost;
pub mod hud;
pub mod input;
pub mod item;
pub mod lifetime;
pub mod movement;
pub mod player;
pub mod state;
