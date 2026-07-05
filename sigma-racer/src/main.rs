//! Sigma Racer — production instrument cluster for Wingman / i.MX 8M Plus.
//!
//! Live vehicle data replaces the idle loop via CAN-FD from the M7 safety core.

mod telemetry;
mod vehicle;

use chrono::Local;
use sigma_instrumentation::{
    configure_window, init_gauge_art, theme, DisplayConfig, SigmaDashboard,
};
use slint::ComponentHandle;
use slint::SharedString;
use std::time::Duration;
use telemetry::attach as attach_telemetry;

const BOOT_SPLASH_MS: u64 = 1200;

fn main() -> Result<(), slint::PlatformError> {
    let ui = SigmaDashboard::new()?;

    ui.set_boot_visible(true);
    theme::init_from_env(&ui);
    configure_window(
        &ui,
        DisplayConfig::embedded(cfg!(sigma_racer_wingman_embedded)),
    );
    init_gauge_art(&ui);

    ui.set_current_window(0);
    push_idle(&ui);
    attach_telemetry(&ui);

    let ui_weak = ui.as_weak();
    slint::Timer::single_shot(Duration::from_millis(BOOT_SPLASH_MS), move || {
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_boot_visible(false);
        }
    });

    let ui_weak = ui.as_weak();
    let timer = slint::Timer::default();
    timer.start(slint::TimerMode::Repeated, Duration::from_secs(1), move || {
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_clock(SharedString::from(
                Local::now().format("%H:%M").to_string(),
            ));
        }
    });

    ui.run()
}

/// Fallback values until the first Snapshot arrives from vehicle.service.
fn push_idle(ui: &SigmaDashboard) {
    use sigma_instrumentation::{gauge, set_speed_readout};
    use vehicle::XSR900_GP;

    let profile = XSR900_GP;

    ui.set_rpm(profile.idle_rpm);
    set_speed_readout(ui, 0);
    ui.set_gear(0);
    ui.set_at_redline(false);
    ui.set_side_stand(true);
    ui.set_swept_path(gauge::swept_path(profile.idle_rpm));
    let (nl, ns, nr, no) = gauge::needle_paths(profile.idle_rpm);
    ui.set_needle_left(nl);
    ui.set_needle_spine(ns);
    ui.set_needle_right(nr);
    ui.set_needle_outline(no);
    ui.set_fuel_pct(0.62);
    ui.set_lean_angle(0.0);
    ui.set_gforce(0.0);
    ui.set_coolant_c(42);
    ui.set_oil_c(52);
    ui.set_battery_v(13.1);
    ui.set_can_load(8);
    ui.set_heading(0.0);
    ui.set_heading_label(SharedString::from("N"));
    ui.set_elevation(667);
    ui.set_nav_blocked_hint(false);
}
