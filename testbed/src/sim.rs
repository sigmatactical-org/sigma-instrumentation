//! Yamaha XSR900 GP ride simulation for the testbed.

use chrono::Local;
use sigma_instrumentation::{gauge, windows, SigmaDashboard};
use slint::SharedString;
use std::cell::Cell;
use std::rc::Rc;
use std::time::Instant;

// Yamaha XSR900 GP (890 cc CP3) — red zone 11 250 r/min, ~235 km/h top speed
const IDLE_RPM: f32 = 1_200.0;
const REV_LIMIT_RPM: f32 = 11_500.0;
const MAX_SPEED_KMH: f32 = 235.0;
const SHIFT_PAUSE_S: f32 = 0.10;
const TOP_SPEED_HOLD_S: f32 = 1.2;
const SETTLE_S: f32 = 1.5;
const NAV_BLOCKED_HINT_S: f32 = 2.0;
const SLOW_UI_INTERVAL_S: f32 = 1.0;
const DEFAULT_FUEL_PCT: f32 = 0.62;
const DRAG_PLATEAU_ACCEL: f32 = 0.08;
const VMAX_UPSHIFT_RATIO: f32 = 0.87;

const SHIFT_UP_BY_GEAR: [f32; 7] = [0.0, 11_300.0, 11_100.0, 10_350.0, 10_100.0, 9_400.0, 8_500.0];
const SHIFT_DOWN_BY_GEAR: [f32; 7] = [0.0, 3_800.0, 4_200.0, 4_800.0, 5_500.0, 6_500.0, 7_800.0];

const PRIMARY_RATIO: f32 = 1.681;
const FINAL_DRIVE: f32 = 2.813;
const GEARBOX: [f32; 6] = [2.667, 2.000, 1.619, 1.381, 1.190, 1.037];
const WHEEL_CIRC_M: f32 = 1.88;

#[derive(Clone, Copy, PartialEq, Eq)]
enum DemoPhase {
    Launch,
    AccelRun,
    TopSpeedHold,
    DecelRun,
    Settle,
}

pub struct RideSimulation {
    speed_kmh: Cell<f32>,
    rpm: Cell<f32>,
    gear: Cell<i32>,
    phase: Cell<DemoPhase>,
    shift_pause: Cell<f32>,
    hold_timer: Cell<f32>,
    hold_start_speed: Cell<f32>,
    settle_timer: Cell<f32>,
    last_tick: Cell<Option<Instant>>,
    fuel_pct: Cell<f32>,
    prev_speed: Cell<f32>,
    gforce: Cell<f32>,
    demo_t: Cell<f32>,
    window: Cell<i32>,
    nav_blocked_timer: Cell<f32>,
    last_pushed_rpm: Cell<f32>,
    last_pushed_speed: Cell<i32>,
    slow_ui_accum: Cell<f32>,
}

impl Default for RideSimulation {
    fn default() -> Self {
        Self {
            speed_kmh: Cell::new(0.0),
            rpm: Cell::new(IDLE_RPM),
            gear: Cell::new(0),
            phase: Cell::new(DemoPhase::Launch),
            shift_pause: Cell::new(0.0),
            hold_timer: Cell::new(0.0),
            hold_start_speed: Cell::new(0.0),
            settle_timer: Cell::new(0.0),
            last_tick: Cell::new(None),
            fuel_pct: Cell::new(DEFAULT_FUEL_PCT),
            prev_speed: Cell::new(0.0),
            gforce: Cell::new(0.0),
            demo_t: Cell::new(0.0),
            window: Cell::new(0),
            nav_blocked_timer: Cell::new(0.0),
            last_pushed_rpm: Cell::new(f32::NAN),
            last_pushed_speed: Cell::new(-1),
            slow_ui_accum: Cell::new(SLOW_UI_INTERVAL_S),
        }
    }
}

impl RideSimulation {
    pub fn restart_run(&self) {
        self.speed_kmh.set(0.0);
        self.rpm.set(IDLE_RPM);
        self.gear.set(0);
        self.phase.set(DemoPhase::Launch);
        self.shift_pause.set(0.0);
        self.hold_timer.set(0.0);
        self.hold_start_speed.set(0.0);
        self.settle_timer.set(0.0);
        self.prev_speed.set(0.0);
        self.gforce.set(0.0);
        self.fuel_pct.set(DEFAULT_FUEL_PCT);
        self.last_pushed_rpm.set(f32::NAN);
        self.last_pushed_speed.set(-1);
    }

    pub fn force_decel(&self) {
        self.phase.set(DemoPhase::DecelRun);
    }

    pub fn nav_next(&self) {
        let cur = self.window.get();
        let next = if self.stopped() {
            (cur + 1).rem_euclid(windows::COUNT)
        } else {
            (cur.clamp(0, windows::PANEL_MAX) + 1).rem_euclid(windows::PANEL_MAX + 1)
        };
        self.window.set(next);
    }

    pub fn nav_prev(&self) {
        let cur = self.window.get();
        let prev = if self.stopped() {
            (cur - 1).rem_euclid(windows::COUNT)
        } else {
            (cur.clamp(0, windows::PANEL_MAX) - 1).rem_euclid(windows::PANEL_MAX + 1)
        };
        self.window.set(prev);
    }

    pub fn nav_home(&self) {
        self.window.set(0);
    }

    pub fn nav_select(&self, idx: i32) {
        if !(0..windows::COUNT).contains(&idx) {
            return;
        }
        if idx > windows::PANEL_MAX && !self.stopped() {
            self.nav_blocked_timer.set(NAV_BLOCKED_HINT_S);
            return;
        }
        self.window.set(idx);
    }

    pub fn step(&self, ui: &SigmaDashboard) {
        let now = Instant::now();
        let dt = self
            .last_tick
            .get()
            .map(|t| now.duration_since(t).as_secs_f32())
            .unwrap_or(0.016)
            .clamp(0.0, 0.1);
        self.last_tick.set(Some(now));
        self.demo_t.set(self.demo_t.get() + dt);
        self.tick_nav_blocked(dt);

        if self.shift_pause.get() > 0.0 {
            let pause = (self.shift_pause.get() - dt).max(0.0);
            self.shift_pause.set(pause);
            self.push_ui(ui, dt);
            return;
        }

        let phase = self.phase.get();
        let mut speed = self.speed_kmh.get();
        let mut gear = self.gear.get();
        let mut rpm = self.rpm.get();

        match phase {
            DemoPhase::Launch => {
                gear = 1;
                rpm = 2_800.0;
                self.phase.set(DemoPhase::AccelRun);
            }
            DemoPhase::AccelRun => {
                if gear == 0 {
                    gear = 1;
                }
                let accel = acceleration_ms2(speed, gear, rpm, true);
                speed = (speed + accel * dt * 3.6).clamp(0.0, MAX_SPEED_KMH);
                rpm = rpm_from_speed(speed, gear);

                if should_upshift(speed, gear, rpm, accel) {
                    gear += 1;
                    rpm = rpm_from_speed(speed, gear);
                    self.shift_pause.set(SHIFT_PAUSE_S);
                }

                if gear >= 6 {
                    let accel_now = acceleration_ms2(speed, gear, rpm, true);
                    if speed >= MAX_SPEED_KMH - 2.0 || accel_now <= 0.05 {
                        self.begin_top_speed_hold(speed);
                    }
                }
            }
            DemoPhase::TopSpeedHold => {
                gear = 6;
                let start = self.hold_start_speed.get();
                let elapsed = TOP_SPEED_HOLD_S - self.hold_timer.get();
                let t = (elapsed / TOP_SPEED_HOLD_S).clamp(0.0, 1.0);
                let ease = t * t * (3.0 - 2.0 * t);
                speed = start + (MAX_SPEED_KMH - start) * ease;
                rpm = rpm_from_speed(speed, gear);
                let mut hold = self.hold_timer.get() - dt;
                if hold <= 0.0 {
                    speed = MAX_SPEED_KMH;
                    rpm = rpm_from_speed(speed, gear);
                    hold = 0.0;
                    self.phase.set(DemoPhase::DecelRun);
                }
                self.hold_timer.set(hold);
            }
            DemoPhase::DecelRun => {
                if gear == 0 {
                    gear = 1;
                }
                let decel = acceleration_ms2(speed, gear, rpm, false);
                speed = (speed + decel * dt * 3.6).max(0.0);
                rpm = rpm_from_speed(speed, gear);

                if should_downshift(gear, rpm) {
                    gear -= 1;
                    if gear == 0 {
                        rpm = IDLE_RPM;
                    } else {
                        rpm = rpm_from_speed(speed, gear);
                    }
                    self.shift_pause.set(SHIFT_PAUSE_S);
                }

                if speed <= 0.05 {
                    speed = 0.0;
                    gear = 0;
                    rpm = IDLE_RPM;
                    self.settle_timer.set(SETTLE_S);
                    self.phase.set(DemoPhase::Settle);
                }
            }
            DemoPhase::Settle => {
                let mut t = self.settle_timer.get() - dt;
                if t <= 0.0 {
                    self.phase.set(DemoPhase::Launch);
                    t = 0.0;
                }
                self.settle_timer.set(t);
                speed = 0.0;
                gear = 0;
                rpm = IDLE_RPM;
            }
        }

        rpm = rpm.clamp(IDLE_RPM, REV_LIMIT_RPM);

        let dv = (speed - self.prev_speed.get()) / 3.6;
        let g = if dt > 0.0 { (dv / dt) / 9.81 } else { 0.0 };
        let g_smooth = self.gforce.get() * 0.8 + g * 0.2;
        self.gforce.set(g_smooth);
        self.prev_speed.set(speed);

        if speed > 0.0 {
            let dist_km = speed * dt / 3600.0;
            self.fuel_pct
                .set((self.fuel_pct.get() - dist_km / 300.0).max(0.0));
        }

        self.speed_kmh.set(speed);
        self.gear.set(gear);
        self.rpm.set(rpm);

        if !self.stopped() && self.window.get() > windows::PANEL_MAX {
            self.window.set(0);
        }

        self.push_ui(ui, dt);
    }

    fn begin_top_speed_hold(&self, speed: f32) {
        self.hold_start_speed.set(speed);
        self.hold_timer.set(TOP_SPEED_HOLD_S);
        self.phase.set(DemoPhase::TopSpeedHold);
    }

    fn stopped(&self) -> bool {
        self.speed_kmh.get() == 0.0
    }

    fn tick_nav_blocked(&self, dt: f32) {
        let t = self.nav_blocked_timer.get();
        if t > 0.0 {
            self.nav_blocked_timer.set((t - dt).max(0.0));
        }
    }

    fn push_ui(&self, ui: &SigmaDashboard, dt: f32) {
        let rpm = self.rpm.get();
        let speed = self.speed_kmh.get().round() as i32;
        let gear = self.gear.get().clamp(0, 6);
        let t = self.demo_t.get();

        ui.set_rpm(rpm);
        ui.set_speed(speed);
        ui.set_gear(gear);
        ui.set_at_redline(rpm >= gauge::REDLINE);
        ui.set_side_stand(speed == 0 && gear == 0);
        ui.set_current_window(self.window.get());
        ui.set_nav_blocked_hint(self.nav_blocked_timer.get() > 0.0);

        let prev_rpm = self.last_pushed_rpm.get();
        if prev_rpm.is_nan() || (rpm - prev_rpm).abs() > 0.01 {
            ui.set_swept_path(gauge::swept_path(rpm));
            let (nl, ns, nr, no) = gauge::needle_paths(rpm);
            ui.set_needle_left(nl);
            ui.set_needle_spine(ns);
            ui.set_needle_right(nr);
            ui.set_needle_outline(no);
            self.last_pushed_rpm.set(rpm);
        }

        if speed != self.last_pushed_speed.get() {
            let (h, te, o) = speed_digits(speed);
            ui.set_d_hundreds(h);
            ui.set_d_tens(te);
            ui.set_d_ones(o);
            self.last_pushed_speed.set(speed);
        }

        ui.set_fuel_pct(self.fuel_pct.get());
        ui.set_lean_angle(if speed > 20 {
            22.0 * (t * 0.7).sin()
        } else {
            0.0
        });
        ui.set_gforce(self.gforce.get());

        let mut slow_accum = self.slow_ui_accum.get() + dt;
        if slow_accum >= SLOW_UI_INTERVAL_S {
            slow_accum %= SLOW_UI_INTERVAL_S;
            self.push_slow_ui(ui, rpm, t);
        }
        self.slow_ui_accum.set(slow_accum);
    }

    fn push_slow_ui(&self, ui: &SigmaDashboard, rpm: f32, t: f32) {
        ui.set_clock(SharedString::from(
            Local::now().format("%H:%M").to_string(),
        ));

        let warm = (t / 25.0).clamp(0.0, 1.0);
        let coolant = 40.0 + warm * 46.0 + (rpm / gauge::MAX_RPM) * 6.0;
        ui.set_coolant_c(coolant.round() as i32);
        ui.set_oil_c((coolant + 10.0).round() as i32);

        let battery = if rpm < 1_500.0 { 13.1 } else { 13.9 };
        ui.set_battery_v(battery);
        ui.set_can_load((20.0 + (rpm / gauge::MAX_RPM) * 34.0).round() as i32);

        let heading = (t * 8.0).rem_euclid(360.0);
        ui.set_heading(heading);
        ui.set_heading_label(SharedString::from(sigma_instrumentation::heading_label(heading)));
        ui.set_elevation((667.0 + 30.0 * (t * 0.05).sin()).round() as i32);
    }
}

fn total_ratio(gear: i32) -> f32 {
    debug_assert!(
        (1..=6).contains(&gear),
        "total_ratio: gear must be 1..=6, got {gear}"
    );
    if !(1..=6).contains(&gear) {
        return 1.0;
    }
    PRIMARY_RATIO * GEARBOX[(gear - 1) as usize] * FINAL_DRIVE
}

fn rpm_from_speed(speed_kmh: f32, gear: i32) -> f32 {
    if gear <= 0 {
        return IDLE_RPM;
    }
    let wheel_rpm = speed_kmh / 3.6 / WHEEL_CIRC_M * 60.0;
    (wheel_rpm * total_ratio(gear)).clamp(IDLE_RPM, REV_LIMIT_RPM)
}

fn gear_vmax(gear: i32) -> f32 {
    if gear <= 0 {
        return 0.0;
    }
    REV_LIMIT_RPM / total_ratio(gear) * WHEEL_CIRC_M / 60.0 * 3.6
}

fn shift_up_rpm(gear: i32) -> f32 {
    SHIFT_UP_BY_GEAR[gear.clamp(0, 6) as usize]
}

fn shift_down_rpm(gear: i32) -> f32 {
    SHIFT_DOWN_BY_GEAR[gear.clamp(0, 6) as usize]
}

fn should_upshift(speed_kmh: f32, gear: i32, rpm: f32, accel: f32) -> bool {
    if gear >= 6 {
        return false;
    }
    let target = shift_up_rpm(gear);
    if rpm >= target {
        return true;
    }
    if accel <= DRAG_PLATEAU_ACCEL && rpm > 6_000.0 {
        return true;
    }
    if rpm >= target * 0.96 && accel < 0.25 {
        return true;
    }
    speed_kmh >= gear_vmax(gear) * VMAX_UPSHIFT_RATIO
}

fn should_downshift(gear: i32, rpm: f32) -> bool {
    gear > 1 && rpm < shift_down_rpm(gear)
}

fn torque_factor(rpm: f32) -> f32 {
    let low = (rpm / 3_500.0).clamp(0.0, 1.0);
    let mid = 1.0 - ((rpm - 10_000.0) / 4_000.0).clamp(0.0, 1.0).powi(2) * 0.35;
    let high = 1.0 - ((rpm - 11_000.0) / 350.0).clamp(0.0, 1.0).powi(2);
    (low * mid * high).clamp(0.12, 1.0)
}

fn acceleration_ms2(speed_kmh: f32, gear: i32, rpm: f32, throttle: bool) -> f32 {
    if gear <= 0 {
        return if throttle { 0.0 } else { -0.4 };
    }

    let v = speed_kmh / 3.6;
    let v_max = gear_vmax(gear).max(1.0);
    let v_ratio = (speed_kmh / v_max).clamp(0.0, 1.0);
    let drag = 0.28 + 0.000_42 * v * v;
    let redline = gauge::REDLINE;

    if throttle {
        let a_peak = 12.5 / (gear as f32).sqrt();
        let torque = torque_factor(rpm);
        let pull = if gear >= 6 {
            let rpm_room = (1.0 - (rpm / redline).powf(1.5)).max(0.08);
            a_peak * torque * rpm_room
        } else {
            a_peak * torque * (1.0 - v_ratio.powf(1.45))
        };
        pull - drag
    } else {
        let engine_brake = 3.0 * (gear as f32).powf(0.55) * (rpm / redline).powf(0.7);
        let aero_brake = 0.000_42 * v * v;
        -(engine_brake + drag * 0.5 + aero_brake)
    }
}

fn speed_digits(speed: i32) -> (SharedString, SharedString, SharedString) {
    let s = speed.clamp(0, 999);
    let h = s / 100;
    let t = (s / 10) % 10;
    let o = s % 10;
    let hs = if s >= 100 {
        SharedString::from(format!("{h}"))
    } else {
        SharedString::from("")
    };
    let ts = if s >= 10 {
        SharedString::from(format!("{t}"))
    } else {
        SharedString::from("")
    };
    let os = SharedString::from(format!("{o}"));
    (hs, ts, os)
}
