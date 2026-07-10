//! Candump replay that emits formatted [`ClusterTelemetry`] for the dashboard.

use chrono::{Local, Timelike};
use sigma_instrumentation::{
    apply_telemetry, ClusterTelemetry, GaugeScale, SigmaDashboard, TelemetryPresenter,
};
use sigma_racer_telemetry::can::{decode_frame, parse_candump, CandumpFrame};
use sigma_racer_telemetry::VehicleState;
use slint::SharedString;
use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::map::to_cluster;

const GAUGE: GaugeScale = GaugeScale::DEFAULT;
const SAMPLE_LOG: &str =
    include_str!("../../../sigma-racer-cluster/testdata/sample-ride.log");

pub struct CandumpReplay {
    frames: RefCell<Vec<CandumpFrame>>,
    path_label: RefCell<String>,
    state: RefCell<VehicleState>,
    cursor: Cell<usize>,
    last_tick: Cell<Option<Instant>>,
    /// Simulated seconds into the log (advances by dt × rate).
    sim_t: Cell<f64>,
    rate: Cell<f32>,
    last_clock_min: Cell<i32>,
    /// Hold idle RPM + side stand down; freeze candump advance.
    parked: Cell<bool>,
}

impl Default for CandumpReplay {
    fn default() -> Self {
        let frames = parse_candump(SAMPLE_LOG);
        Self {
            frames: RefCell::new(frames),
            path_label: RefCell::new("(baked sample)".into()),
            state: RefCell::new(VehicleState::idle()),
            cursor: Cell::new(0),
            last_tick: Cell::new(None),
            sim_t: Cell::new(0.0),
            rate: Cell::new(1.0),
            last_clock_min: Cell::new(-1),
            parked: Cell::new(false),
        }
    }
}

impl CandumpReplay {
    pub fn path_label(&self) -> String {
        self.path_label.borrow().clone()
    }

    pub fn rate(&self) -> f32 {
        self.rate.get()
    }

    pub fn set_rate(&self, rate: f32) {
        self.rate.set(rate.clamp(0.25, 4.0));
    }

    pub fn parked(&self) -> bool {
        self.parked.get()
    }

    /// Latch park: idle RPM, side stand down, freeze replay. Toggle again to resume.
    pub fn toggle_park(&self) {
        if self.parked.get() {
            self.parked.set(false);
            return;
        }
        self.parked.set(true);
        *self.state.borrow_mut() = VehicleState::idle();
        self.state.borrow_mut().signals_live = true;
        self.last_tick.set(None);
    }

    pub fn load_path(&self, path: &Path) -> Result<(), String> {
        let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        let frames = parse_candump(&text);
        if frames.is_empty() {
            return Err("no usable candump frames".into());
        }
        let label = path
            .file_name()
            .and_then(|s| s.to_str())
            .map(|s| s.to_owned())
            .unwrap_or_else(|| path.display().to_string());
        self.replace_frames(frames, label);
        Ok(())
    }

    pub fn restart(&self) {
        let label = self.path_label.borrow().clone();
        let frames = self.frames.borrow().clone();
        self.replace_frames(frames, label);
    }

    fn replace_frames(&self, frames: Vec<CandumpFrame>, label: String) {
        *self.frames.borrow_mut() = frames;
        *self.path_label.borrow_mut() = label;
        *self.state.borrow_mut() = VehicleState::idle();
        self.cursor.set(0);
        self.sim_t.set(0.0);
        self.last_tick.set(None);
        self.parked.set(false);
    }

    fn stopped(&self) -> bool {
        self.state.borrow().speed < 0.5
    }

    pub fn is_stopped(&self) -> bool {
        self.stopped()
    }

    /// Advance replay by one UI tick and present formatted telemetry.
    pub fn step(&self, ui: &SigmaDashboard) {
        let now = Instant::now();
        let dt = match self.last_tick.get() {
            Some(prev) => now.duration_since(prev).as_secs_f64().min(0.05),
            None => 0.0,
        };
        self.last_tick.set(Some(now));

        if self.parked.get() {
            let mut state = self.state.borrow_mut();
            // Hold park: idle RPM, side stand down, zero motion.
            state.rpm = 1_200.0;
            state.speed = 0.0;
            state.gear = 0;
            state.throttle_pct = 0.0;
            state.side_stand = true;
            state.lean_angle = 0.0;
            state.gforce = 0.0;
            state.at_redline = false;
            state.redline_can = false;
            state.signals_live = true;
            state.refresh_derived();

            let mut msg = to_cluster(&state);
            msg.signals_live = true;
            apply_telemetry(ui, &msg, &GAUGE);
            ui.set_parked(true);

            let clock = Local::now();
            let minute = clock.minute() as i32;
            if minute != self.last_clock_min.get() {
                self.last_clock_min.set(minute);
                ui.set_clock(SharedString::from(clock.format("%H:%M").to_string()));
            }
            return;
        }

        ui.set_parked(false);

        let rate = f64::from(self.rate.get());
        let mut t = self.sim_t.get() + dt * rate;

        let frames = self.frames.borrow();
        if frames.is_empty() {
            let mut idle = ClusterTelemetry::idle();
            idle.signals_live = false;
            idle.present(ui, &GAUGE);
            return;
        }

        let mut cursor = self.cursor.get();
        let mut state = self.state.borrow_mut();

        if cursor >= frames.len() {
            cursor = 0;
            t = 0.0;
            *state = VehicleState::idle();
        }

        while cursor < frames.len() && frames[cursor].at <= t {
            let f = &frames[cursor];
            decode_frame(f.id, &f.data, &mut state);
            cursor += 1;
        }
        state.refresh_derived();
        state.signals_live = true;

        self.cursor.set(cursor);
        self.sim_t.set(t);

        let mut msg = to_cluster(&state);
        msg.signals_live = true;
        apply_telemetry(ui, &msg, &GAUGE);

        let clock = Local::now();
        let minute = clock.minute() as i32;
        if minute != self.last_clock_min.get() {
            self.last_clock_min.set(minute);
            ui.set_clock(SharedString::from(clock.format("%H:%M").to_string()));
        }
    }
}

/// Open a native file dialog and return the chosen path.
pub fn pick_candump_file() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("candump log", &["log", "txt", "candump"])
        .add_filter("All", &["*"])
        .set_title("Select candump -L log")
        .pick_file()
}
