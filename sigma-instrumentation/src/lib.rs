//! Reusable Sigma instrument cluster UI — Slint dashboard, gauge geometry, themes.
//!
//! Product binaries (`sigma-racer-cluster`, `testbed`) depend on this crate for the shared
//! dashboard surface and helpers. Vehicle producers push [`ClusterTelemetry`]; this crate
//! never decodes CAN.

slint::include_modules!();

pub mod buttons;
pub mod camera;
pub mod connectivity;
pub mod dashboard;
pub mod display;
pub mod gauge;
pub mod heading;
pub mod telemetry;
pub mod theme;
pub mod updates;
pub mod windows;

pub use dashboard::{init_gauge_art, set_needle_paths, set_speed_readout, speed_digits, start_signal_blink};
pub use display::{
    configure_window, ensure_panel_geometry, force_panel_scale_factor, DisplayConfig,
};
pub use gauge::GaugeScale;
pub use heading::heading_label;
pub use telemetry::{apply_telemetry, ClusterTelemetry, TelemetryPresenter};
pub use theme::DisplayMode;
pub use updates::{start as start_updates_client, UpdatesConfig};
