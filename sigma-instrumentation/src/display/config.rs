//! Display sizing presets.

pub const PANEL_WIDTH: u32 = 800;
pub const PANEL_HEIGHT: u32 = 480;
/// Cluster face (480) + harness strip (~96).
const TESTBED_HEIGHT: u32 = 576;

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

    /// Desktop / virt testbed with the config harness under the cluster face.
    pub const fn testbed() -> Self {
        Self {
            fixed_size: Some((PANEL_WIDTH, TESTBED_HEIGHT)),
            default_kiosk: false,
        }
    }

    /// Virt panel size but tall enough for the harness strip.
    pub const fn testbed_virt_panel() -> Self {
        Self {
            fixed_size: Some((PANEL_WIDTH, TESTBED_HEIGHT)),
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
            // Always the cluster panel size — paired with physical pin in kiosk.
            fixed_size: Some((PANEL_WIDTH, PANEL_HEIGHT)),
            default_kiosk,
        }
    }
}
