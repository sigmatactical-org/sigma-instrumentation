//! Reusable Sigma instrument cluster UI — Slint dashboard, gauge geometry, themes.
//!
//! Product binaries (`sigma-racer`, `testbed`) depend on this crate for the shared
//! dashboard surface and helpers.

slint::include_modules!();

pub mod dashboard;
pub mod display;
pub mod gauge;
pub mod theme;
pub mod windows;

pub use dashboard::{init_gauge_art, set_speed_readout, speed_digits};
pub use display::{configure_window, DisplayConfig};
