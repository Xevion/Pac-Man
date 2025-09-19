#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

#[cfg_attr(coverage_nightly, coverage(off))]
pub mod config;
#[cfg_attr(coverage_nightly, coverage(off))]
pub mod errors;
#[cfg_attr(coverage_nightly, coverage(off))]
pub mod formatter;

pub mod app;
pub mod auth;
pub mod data;
pub mod image;
pub mod logging;
pub mod routes;
pub mod session;
