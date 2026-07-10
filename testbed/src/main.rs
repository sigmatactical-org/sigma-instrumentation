//! Interactive testbed — candump replay into the real cluster UI + harness.

mod connectivity_sim;
mod map;
mod replay;
mod snapshot;

use connectivity_sim::NavState;
use replay::{pick_candump_file, CandumpReplay};
use sigma_instrumentation::{
    configure_window, init_gauge_art, set_speed_readout, start_signal_blink, start_updates_client,
    theme, windows, DisplayConfig, GaugeScale, SigmaDashboard, UpdatesConfig,
};
use slint::{ComponentHandle, Model, SharedString};
use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

fn main() -> Result<(), slint::PlatformError> {
    let ui = SigmaDashboard::new()?;

    let preset = theme::init_from_env(&ui);
    let mode = Rc::new(Cell::new(preset.display_mode()));

    configure_window(&ui, DisplayConfig::testbed());
    init_gauge_art(&ui, &GaugeScale::DEFAULT);
    set_speed_readout(&ui, 0);

    let labels = ui.get_labels();
    eprintln!(
        "testbed: labels={} speed={} scale={}",
        labels.row_count(),
        ui.get_speed(),
        ui.window().scale_factor()
    );
    if let Some(t) = labels.row_data(0) {
        eprintln!("testbed: label[0]='{}' at ({:.1},{:.1})", t.label, t.x, t.y);
    }
    if let Some(t) = labels.row_data(labels.row_count().saturating_sub(1)) {
        eprintln!(
            "testbed: label[{}]='{}' at ({:.1},{:.1})",
            labels.row_count().saturating_sub(1),
            t.label,
            t.x,
            t.y
        );
    }

    ui.set_harness_visible(true);
    ui.set_display_mode_label(SharedString::from(mode.get().as_str()));
    ui.set_replay_rate(1.0);

    let replay = Rc::new(CandumpReplay::default());
    let nav = Rc::new(NavState::default());
    connectivity_sim::init(&ui, &nav);
    ui.set_candump_path(SharedString::from(replay.path_label()));

    {
        let replay = replay.clone();
        ui.on_rpm_up(move || replay.restart());
    }
    {
        let replay = replay.clone();
        ui.on_rpm_down(move || replay.set_rate((replay.rate() * 0.5).max(0.25)));
    }
    {
        let replay = replay.clone();
        let nav = nav.clone();
        let ui_weak = ui.as_weak();
        ui.on_nav_next(move || {
            if let Some(ui) = ui_weak.upgrade() {
                nav.nav_next(&ui, replay.is_stopped());
            }
        });
    }
    {
        let replay = replay.clone();
        let nav = nav.clone();
        let ui_weak = ui.as_weak();
        ui.on_nav_prev(move || {
            if let Some(ui) = ui_weak.upgrade() {
                nav.nav_prev(&ui, replay.is_stopped());
            }
        });
    }
    {
        let nav = nav.clone();
        let ui_weak = ui.as_weak();
        ui.on_nav_back(move || {
            if let Some(ui) = ui_weak.upgrade() {
                nav.nav_back(&ui);
            }
        });
    }
    {
        let nav = nav.clone();
        let ui_weak = ui.as_weak();
        ui.on_nav_select(move || {
            if let Some(ui) = ui_weak.upgrade() {
                nav.nav_select(&ui);
            }
        });
    }

    {
        let replay = replay.clone();
        let ui_weak = ui.as_weak();
        ui.on_pick_candump(move || {
            let Some(path) = pick_candump_file() else {
                return;
            };
            if let Err(err) = replay.load_path(&path) {
                eprintln!("testbed: load candump: {err}");
                return;
            }
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_candump_path(SharedString::from(replay.path_label()));
            }
        });
    }

    {
        let mode = mode.clone();
        let ui_weak = ui.as_weak();
        ui.on_cycle_display_mode(move || {
            let next = mode.get().cycle();
            mode.set(next);
            if let Some(ui) = ui_weak.upgrade() {
                theme::apply_mode(&ui, next);
                ui.set_display_mode_label(SharedString::from(next.as_str()));
            }
        });
    }

    {
        let replay = replay.clone();
        ui.on_replay_rate_changed(move |rate| {
            replay.set_rate(rate);
        });
    }

    {
        let ui_weak = ui.as_weak();
        ui.on_toggle_turn_left(move || {
            if let Some(ui) = ui_weak.upgrade() {
                let on = !ui.get_turn_left();
                ui.set_turn_left(on);
                if on {
                    ui.set_turn_right(false);
                }
                if on || ui.get_turn_right() {
                    ui.set_signal_lit(true);
                }
            }
        });
    }
    {
        let ui_weak = ui.as_weak();
        ui.on_toggle_turn_right(move || {
            if let Some(ui) = ui_weak.upgrade() {
                let on = !ui.get_turn_right();
                ui.set_turn_right(on);
                if on {
                    ui.set_turn_left(false);
                }
                if on || ui.get_turn_left() {
                    ui.set_signal_lit(true);
                }
            }
        });
    }
    {
        let ui_weak = ui.as_weak();
        ui.on_toggle_hazard(move || {
            if let Some(ui) = ui_weak.upgrade() {
                let both = ui.get_turn_left() && ui.get_turn_right();
                ui.set_turn_left(!both);
                ui.set_turn_right(!both);
                ui.set_signal_lit(!both);
            }
        });
    }

    {
        let replay = replay.clone();
        let ui_weak = ui.as_weak();
        ui.on_park(move || {
            replay.toggle_park();
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_parked(replay.parked());
            }
        });
    }

    let _signal_blink = start_signal_blink(&ui);
    start_updates_client(&ui, UpdatesConfig::from_env());

    // Dump a real painted frame, then exit.
    if let Ok(path) = std::env::var("SIGMA_SNAPSHOT") {
        let snap_ui = ui.as_weak();
        let snap_replay = replay.clone();
        let snap_nav = nav.clone();
        let frames = Rc::new(Cell::new(0u32));
        let snap_timer = slint::Timer::default();
        snap_timer.start(
            slint::TimerMode::Repeated,
            Duration::from_millis(50),
            move || {
                let Some(ui) = snap_ui.upgrade() else {
                    return;
                };
                snap_replay.step(&ui);
                snap_nav.paint(&ui);
                let n = frames.get() + 1;
                frames.set(n);
                if n >= 10 {
                    let _ = snapshot::dump(&ui, &path);
                    slint::quit_event_loop().ok();
                }
            },
        );
        std::mem::forget(snap_timer);
        return ui.run();
    }

    let timer = slint::Timer::default();
    let tick_replay = replay.clone();
    let tick_nav = nav.clone();
    let tick_ui = ui.as_weak();
    timer.start(
        slint::TimerMode::Repeated,
        Duration::from_millis(16),
        move || {
            if let Some(ui) = tick_ui.upgrade() {
                tick_replay.step(&ui);
                ui.set_current_window(tick_nav.window.get());
                ui.set_nav_blocked_hint(
                    !tick_replay.is_stopped() && tick_nav.window.get() > windows::PANEL_MAX,
                );
            }
        },
    );

    ui.run()
}
