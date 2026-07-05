//! Display sizing — fixed panel vs kiosk fullscreen.

use slint::ComponentHandle;

use crate::SigmaDashboard;

const PANEL_WIDTH: u32 = 800;
const PANEL_HEIGHT: u32 = 480;

/// How the dashboard window should be presented on startup.
#[derive(Clone, Copy, Debug)]
pub struct DisplayConfig {
    /// Fixed window size (e.g. 800×480 virt panel). When set, fullscreen is skipped.
    pub fixed_size: Option<(u32, u32)>,
    /// Default kiosk/fullscreen when `SLINT_FULLSCREEN` is unset.
    pub default_kiosk: bool,
}

impl DisplayConfig {
    pub const fn virt_panel() -> Self {
        Self {
            fixed_size: Some((PANEL_WIDTH, PANEL_HEIGHT)),
            default_kiosk: false,
        }
    }

    pub const fn desktop() -> Self {
        Self {
            fixed_size: None,
            default_kiosk: false,
        }
    }

    pub const fn embedded(default_kiosk: bool) -> Self {
        Self {
            fixed_size: None,
            default_kiosk,
        }
    }
}

fn panel_size_from_env() -> Option<(u32, u32)> {
    let (w, h) = (
        std::env::var("SIGMA_PANEL_WIDTH").ok()?,
        std::env::var("SIGMA_PANEL_HEIGHT").ok()?,
    );
    let (w, h) = (w.parse().ok()?, h.parse().ok()?);
    (w > 0 && h > 0).then_some((w, h))
}

fn pin_panel_window(ui: &SigmaDashboard, w: u32, h: u32) {
    ui.window()
        .set_size(slint::PhysicalSize::new(w, h));
    ui.window()
        .set_position(slint::PhysicalPosition::new(0, 0));
}

/// Apply window size / fullscreen policy from config and environment.
pub fn configure_window(ui: &SigmaDashboard, config: DisplayConfig) {
    if let Some((w, h)) = panel_size_from_env().or(config.fixed_size) {
        pin_panel_window(ui, w, h);
        return;
    }

    let kiosk = std::env::var("SLINT_FULLSCREEN")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(config.default_kiosk);

    if kiosk {
        ui.window().set_fullscreen(true);
        ui.window()
            .set_position(slint::PhysicalPosition::new(0, 0));
    }
}
