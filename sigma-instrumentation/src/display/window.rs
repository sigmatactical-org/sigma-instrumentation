//! Window size and fullscreen policy.

use slint::platform::WindowEvent;
use slint::{ComponentHandle, PhysicalSize};

use crate::SigmaDashboard;

use super::config::{DisplayConfig, PANEL_HEIGHT, PANEL_WIDTH};

fn panel_size_from_env() -> Option<(u32, u32)> {
    let (w, h) = (
        std::env::var("SIGMA_PANEL_WIDTH").ok()?,
        std::env::var("SIGMA_PANEL_HEIGHT").ok()?,
    );
    let (w, h) = (w.parse().ok()?, h.parse().ok()?);
    (w > 0 && h > 0).then_some((w, h))
}

fn kiosk_enabled(config: &DisplayConfig) -> bool {
    std::env::var("SLINT_FULLSCREEN")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(config.default_kiosk)
}

/// Call **before** [`SigmaDashboard::new`]. Starts at 1:1; [`ensure_panel_geometry`]
/// then letterboxes the 800×480 design into whatever surface we actually get.
pub fn force_panel_scale_factor() {
    // SAFETY: single-threaded `main` before other threads / the winit loop.
    unsafe {
        std::env::set_var("SLINT_SCALE_FACTOR", "1");
    }
}

/// Letterbox the fixed 800×480 design into the current window surface.
///
/// Uses scale = min(pw/800, ph/480) so the full dial + speed always fit.
/// A height-only fit (cover) is what cropped the right side on 1024×768 outputs.
pub fn ensure_panel_geometry(ui: &SigmaDashboard, kiosk: bool) {
    if kiosk {
        ui.window().set_fullscreen(true);
        ui.window()
            .set_position(slint::PhysicalPosition::new(0, 0));
    }

    let phys = ui.window().size();
    let pw = (phys.width as f32).max(1.0);
    let ph = (phys.height as f32).max(1.0);

    let scale = (pw / PANEL_WIDTH as f32)
        .min(ph / PANEL_HEIGHT as f32)
        .clamp(0.25, 8.0);

    ui.window()
        .dispatch_event(WindowEvent::ScaleFactorChanged { scale_factor: scale });

    if !kiosk {
        // Windowed: exact design × scale.
        ui.window().set_size(PhysicalSize::new(
            (PANEL_WIDTH as f32 * scale).round().max(1.0) as u32,
            (PANEL_HEIGHT as f32 * scale).round().max(1.0) as u32,
        ));
    }

    let logical = ui.window().size();
    eprintln!(
        "cluster: surface {}×{}px scale={scale:.3} → logical ~{}×{} (design {}×{})",
        phys.width,
        phys.height,
        (logical.width as f32 / scale).round() as u32,
        (logical.height as f32 / scale).round() as u32,
        PANEL_WIDTH,
        PANEL_HEIGHT,
    );
}

/// Apply window size / fullscreen policy from config and environment.
pub fn configure_window(ui: &SigmaDashboard, config: DisplayConfig) {
    let kiosk = kiosk_enabled(&config);

    // Desktop testbed: let preferred-width/height drive the window.
    if config.fixed_size.is_some()
        && !kiosk
        && std::env::var("SIGMA_PIN_WINDOW").ok().as_deref() != Some("1")
        && panel_size_from_env().is_none()
    {
        return;
    }

    if kiosk || config.fixed_size.is_some() || panel_size_from_env().is_some() {
        ensure_panel_geometry(ui, kiosk);
    }
}
