//! Pac-Man game library crate.
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

#[cfg_attr(coverage_nightly, coverage(off))]
pub mod app;
#[cfg_attr(coverage_nightly, coverage(off))]
pub mod audio;
#[cfg_attr(coverage_nightly, coverage(off))]
pub mod error;
#[cfg_attr(coverage_nightly, coverage(off))]
pub mod events;
#[cfg_attr(coverage_nightly, coverage(off))]
pub mod formatter;
#[cfg_attr(coverage_nightly, coverage(off))]
pub mod platform;

pub mod asset;
pub mod constants;
pub mod game;
pub mod map;
pub mod systems;
pub mod texture;
