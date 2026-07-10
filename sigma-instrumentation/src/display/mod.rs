//! Display sizing — fixed panel vs kiosk fullscreen.

mod config;
mod window;

pub use config::DisplayConfig;
pub use window::{configure_window, ensure_panel_geometry, force_panel_scale_factor};
