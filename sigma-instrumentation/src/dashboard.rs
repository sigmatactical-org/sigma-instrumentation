//! Static gauge artwork and readout helpers bound to a dashboard instance.

use slint::{ModelRc, SharedString, VecModel};
use std::rc::Rc;

use crate::{gauge, SigmaDashboard, Tick};

/// Install computed RPM dial paths, ticks, and numerals on the dashboard.
pub fn init_gauge_art(ui: &SigmaDashboard) {
    ui.set_track_path(gauge::track_path());
    ui.set_redline_path(gauge::redline_path());
    ui.set_ticks_major(gauge::ticks_major());
    ui.set_ticks_minor(gauge::ticks_minor());
    ui.set_ticks_redline(gauge::ticks_redline());
    ui.set_labels(build_numerals());
}

/// Update speed value and digit cells (0–999 km/h).
pub fn set_speed_readout(ui: &SigmaDashboard, speed: i32) {
    let (h, t, o) = speed_digits(speed);
    ui.set_speed(speed);
    ui.set_d_hundreds(h);
    ui.set_d_tens(t);
    ui.set_d_ones(o);
}

pub fn speed_digits(speed: i32) -> (SharedString, SharedString, SharedString) {
    let s = speed.clamp(0, 999);
    let h = s / 100;
    let t = (s / 10) % 10;
    let o = s % 10;
    let hs = if s >= 100 {
        SharedString::from(format!("{h}"))
    } else {
        SharedString::from("")
    };
    let ts = if s == 0 {
        SharedString::from("0")
    } else if s >= 10 {
        SharedString::from(format!("{t}"))
    } else {
        SharedString::from("")
    };
    let os = if s == 0 {
        SharedString::from("")
    } else {
        SharedString::from(format!("{o}"))
    };
    (hs, ts, os)
}

fn build_numerals() -> ModelRc<Tick> {
    let rows: Vec<Tick> = gauge::numerals()
        .into_iter()
        .map(|n| Tick {
            x: n.x,
            y: n.y,
            label: SharedString::from(n.label),
            redline: n.redline,
        })
        .collect();
    ModelRc::new(Rc::new(VecModel::from(rows)))
}
