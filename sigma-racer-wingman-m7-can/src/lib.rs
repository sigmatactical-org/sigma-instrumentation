//! Sigma Racer Wingman **M7 safety-bus** CAN dictionary and codec.
//!
//! This crate is the single source of truth for the CAN contract on the M7
//! safety bus: the message IDs, the `.dbc` database, and the frame⇄signal
//! codec. Both the Linux `vehicle-service` (`std`) and the future M7 firmware
//! (`no_std`) depend on it so the two compute domains can never disagree on
//! message IDs or signal scaling.
//!
//! Decoding and encoding are allocation-free (bounded `dbc-rs` buffers); only
//! [`parse`] builds the database and must be called once — lazily via [`dbc`]
//! under `std`, or once at boot on the M7.

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

pub use dbc_rs::{Dbc, DecodedSignal, Error};

/// The canonical M7 draft DBC source — the single source of truth.
pub const M7_DBC: &str = include_str!("../dbc/m7-draft.dbc");

/// Engine status (RPM, temperatures, redline flag).
pub const ENGINE_STATUS: u32 = 0x0A0;
/// Throttle position, current gear, ride mode, side-stand.
pub const THROTTLE_GEAR: u32 = 0x0C0;
/// Ground speed, lean angle, longitudinal acceleration.
pub const WHEEL_SPEED: u32 = 0x120;
/// Fuel, battery, bus load, ABS/TC flags, DTC count.
pub const CHASSIS_ELECTRICAL: u32 = 0x200;
/// Odometer and trip meters.
pub const TRIP_ODOMETER: u32 = 0x220;

/// Every message ID in the dictionary, in transmit order.
pub const MESSAGE_IDS: [u32; 5] = [
    ENGINE_STATUS,
    THROTTLE_GEAR,
    WHEEL_SPEED,
    CHASSIS_ELECTRICAL,
    TRIP_ODOMETER,
];

/// Rider performance / ride mode (`VAL_ 192 PerformanceMode`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PerformanceMode {
    Rain,
    #[default]
    Std,
    Sport,
    Track,
}

impl PerformanceMode {
    /// Map a raw DBC value onto a mode (unknown values fall back to `Sport`).
    pub fn from_raw(raw: u8) -> Self {
        match raw {
            0 => Self::Rain,
            1 => Self::Std,
            3 => Self::Track,
            _ => Self::Sport,
        }
    }

    /// The raw DBC value for this mode.
    pub fn to_raw(self) -> u8 {
        match self {
            Self::Rain => 0,
            Self::Std => 1,
            Self::Sport => 2,
            Self::Track => 3,
        }
    }

    /// The human-readable label (matches the DBC `VAL_` table).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Rain => "RAIN",
            Self::Std => "STD",
            Self::Sport => "SPORT",
            Self::Track => "TRACK",
        }
    }

    /// Parse a label back into a mode.
    pub fn from_label(label: &str) -> Option<Self> {
        match label {
            "RAIN" => Some(Self::Rain),
            "STD" => Some(Self::Std),
            "SPORT" => Some(Self::Sport),
            "TRACK" => Some(Self::Track),
            _ => None,
        }
    }
}

/// Neutral, `Copy` snapshot of the M7 signal dictionary in physical units.
///
/// This is the shared contract type: the codec decodes frames into it and
/// encodes frames from it, independent of any consumer's own state model.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct M7Signals {
    pub engine_rpm: f32,
    pub coolant_c: i16,
    pub oil_c: i16,
    pub redline: bool,
    pub throttle_pct: f32,
    pub gear: i8,
    pub performance_mode: PerformanceMode,
    pub side_stand: bool,
    pub ground_speed: f32,
    pub lean_angle: f32,
    pub long_accel: f32,
    pub fuel_pct: f32,
    pub battery_v: f32,
    pub can_load: u8,
    pub abs_active: bool,
    pub tc_active: bool,
    pub dtc_count: u8,
    pub odometer: f32,
    pub trip1: f32,
    pub trip2: f32,
}

impl Default for M7Signals {
    fn default() -> Self {
        Self {
            engine_rpm: 0.0,
            coolant_c: 0,
            oil_c: 0,
            redline: false,
            throttle_pct: 0.0,
            gear: 0,
            performance_mode: PerformanceMode::Std,
            side_stand: false,
            ground_speed: 0.0,
            lean_angle: 0.0,
            long_accel: 0.0,
            fuel_pct: 0.0,
            battery_v: 0.0,
            can_load: 0,
            abs_active: false,
            tc_active: false,
            dtc_count: 0,
            odometer: 0.0,
            trip1: 0.0,
            trip2: 0.0,
        }
    }
}

/// Parse the embedded M7 dictionary into a [`Dbc`].
///
/// Allocation (in `alloc` mode) happens here, not on the decode/encode path.
/// Call once and cache — see [`dbc`] under `std`.
pub fn parse() -> Result<Dbc, Error> {
    Dbc::parse(M7_DBC)
}

/// Decode one CAN frame into `out`, updating only the fields carried by this
/// message. Returns `false` if `id` is not in the dictionary or the frame does
/// not decode.
pub fn decode_into(dbc: &Dbc, id: u32, data: &[u8], out: &mut M7Signals) -> bool {
    let decoded = match dbc.decode(id, data, false) {
        Ok(signals) if !signals.is_empty() => signals,
        _ => return false,
    };
    for signal in decoded.iter() {
        apply(id, signal.name, signal.value, out);
    }
    true
}

fn apply(id: u32, name: &str, value: f64, out: &mut M7Signals) {
    match (id, name) {
        (ENGINE_STATUS, "EngineRPM") => out.engine_rpm = value as f32,
        (ENGINE_STATUS, "CoolantTemp") => out.coolant_c = round(value) as i16,
        (ENGINE_STATUS, "OilTemp") => out.oil_c = round(value) as i16,
        (ENGINE_STATUS, "Redline") => out.redline = value != 0.0,
        (THROTTLE_GEAR, "ThrottlePos") => out.throttle_pct = value as f32,
        (THROTTLE_GEAR, "CurrentGear") => out.gear = round(value) as i8,
        (THROTTLE_GEAR, "PerformanceMode") => {
            out.performance_mode = PerformanceMode::from_raw(round(value) as u8)
        }
        (THROTTLE_GEAR, "SideStand") => out.side_stand = value != 0.0,
        (WHEEL_SPEED, "GroundSpeed") => out.ground_speed = value as f32,
        (WHEEL_SPEED, "LeanAngle") => out.lean_angle = value as f32,
        (WHEEL_SPEED, "LongAccel") => out.long_accel = value as f32,
        (CHASSIS_ELECTRICAL, "FuelLevel") => out.fuel_pct = value as f32,
        (CHASSIS_ELECTRICAL, "BatteryVoltage") => out.battery_v = value as f32,
        (CHASSIS_ELECTRICAL, "CanBusLoad") => out.can_load = round(value) as u8,
        (CHASSIS_ELECTRICAL, "AbsActive") => out.abs_active = value != 0.0,
        (CHASSIS_ELECTRICAL, "TcActive") => out.tc_active = value != 0.0,
        (CHASSIS_ELECTRICAL, "DtcCount") => out.dtc_count = round(value) as u8,
        (TRIP_ODOMETER, "Odometer") => out.odometer = value as f32,
        (TRIP_ODOMETER, "Trip1") => out.trip1 = value as f32,
        (TRIP_ODOMETER, "Trip2") => out.trip2 = value as f32,
        _ => {}
    }
}

/// Round half-away-from-zero without `std` float intrinsics (`f64::round` is
/// unavailable under `no_std`). Adequate for the small, bounded signal values
/// in this dictionary.
#[inline]
fn round(value: f64) -> i64 {
    if value >= 0.0 {
        (value + 0.5) as i64
    } else {
        (value - 0.5) as i64
    }
}

/// Encode the full dictionary into its five CAN frames `(id, 8-byte payload)`.
///
/// Returns an error if any value is outside its DBC-defined range.
pub fn encode_frames(dbc: &Dbc, s: &M7Signals) -> Result<[(u32, [u8; 8]); 5], Error> {
    Ok([
        encode_one(dbc, ENGINE_STATUS, &engine_signals(s))?,
        encode_one(dbc, THROTTLE_GEAR, &throttle_signals(s))?,
        encode_one(dbc, WHEEL_SPEED, &wheel_signals(s))?,
        encode_one(dbc, CHASSIS_ELECTRICAL, &chassis_signals(s))?,
        encode_one(dbc, TRIP_ODOMETER, &trip_signals(s))?,
    ])
}

fn encode_one(dbc: &Dbc, id: u32, signals: &[(&str, f64)]) -> Result<(u32, [u8; 8]), Error> {
    let payload = dbc.encode(id, signals, false)?;
    let mut frame = [0u8; 8];
    let len = payload.len().min(8);
    frame[..len].copy_from_slice(&payload.as_slice()[..len]);
    Ok((id, frame))
}

fn engine_signals(s: &M7Signals) -> [(&'static str, f64); 4] {
    [
        ("EngineRPM", f64::from(s.engine_rpm)),
        ("CoolantTemp", f64::from(s.coolant_c)),
        ("OilTemp", f64::from(s.oil_c)),
        ("Redline", f64::from(u8::from(s.redline))),
    ]
}

fn throttle_signals(s: &M7Signals) -> [(&'static str, f64); 4] {
    [
        ("ThrottlePos", f64::from(s.throttle_pct)),
        ("CurrentGear", f64::from(s.gear.max(0))),
        ("PerformanceMode", f64::from(s.performance_mode.to_raw())),
        ("SideStand", f64::from(u8::from(s.side_stand))),
    ]
}

fn wheel_signals(s: &M7Signals) -> [(&'static str, f64); 3] {
    [
        ("GroundSpeed", f64::from(s.ground_speed)),
        ("LeanAngle", f64::from(s.lean_angle)),
        ("LongAccel", f64::from(s.long_accel)),
    ]
}

fn chassis_signals(s: &M7Signals) -> [(&'static str, f64); 6] {
    [
        ("FuelLevel", f64::from(s.fuel_pct)),
        ("BatteryVoltage", f64::from(s.battery_v)),
        ("CanBusLoad", f64::from(s.can_load)),
        ("AbsActive", f64::from(u8::from(s.abs_active))),
        ("TcActive", f64::from(u8::from(s.tc_active))),
        ("DtcCount", f64::from(s.dtc_count)),
    ]
}

fn trip_signals(s: &M7Signals) -> [(&'static str, f64); 3] {
    [
        ("Odometer", f64::from(s.odometer)),
        ("Trip1", f64::from(s.trip1)),
        ("Trip2", f64::from(s.trip2)),
    ]
}

#[cfg(feature = "std")]
mod cached {
    use super::{parse, Dbc};
    use std::sync::OnceLock;

    static PARSED: OnceLock<Dbc> = OnceLock::new();

    /// Lazily-parsed, cached M7 database (`std` only, thread-safe).
    pub fn dbc() -> &'static Dbc {
        PARSED.get_or_init(|| parse().expect("m7-draft.dbc must parse"))
    }
}

#[cfg(feature = "std")]
pub use cached::dbc;

/// Decode one frame into `out` using the cached database (`std` convenience).
#[cfg(feature = "std")]
pub fn decode(id: u32, data: &[u8], out: &mut M7Signals) -> bool {
    decode_into(dbc(), id, data, out)
}

/// Encode all frames using the cached database (`std` convenience).
#[cfg(feature = "std")]
pub fn encode_all(s: &M7Signals) -> Result<[(u32, [u8; 8]); 5], Error> {
    encode_frames(dbc(), s)
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;

    #[test]
    fn dbc_parses_five_messages() {
        assert_eq!(dbc().messages().len(), 5);
    }

    #[test]
    fn performance_mode_round_trips() {
        for mode in [
            PerformanceMode::Rain,
            PerformanceMode::Std,
            PerformanceMode::Sport,
            PerformanceMode::Track,
        ] {
            assert_eq!(PerformanceMode::from_raw(mode.to_raw()), mode);
            assert_eq!(PerformanceMode::from_label(mode.as_str()), Some(mode));
        }
    }

    #[test]
    fn signals_round_trip_through_can() {
        let sample = M7Signals {
            engine_rpm: 6_000.0,
            coolant_c: 90,
            oil_c: 100,
            redline: false,
            throttle_pct: 42.0,
            gear: 3,
            performance_mode: PerformanceMode::Track,
            side_stand: false,
            ground_speed: 128.0,
            lean_angle: 25.0,
            long_accel: 0.5,
            fuel_pct: 60.0,
            battery_v: 13.5,
            can_load: 12,
            abs_active: true,
            tc_active: false,
            dtc_count: 2,
            odometer: 1_234.5,
            trip1: 100.0,
            trip2: 42.0,
        };

        let frames = encode_all(&sample).expect("encode");
        let mut out = M7Signals::default();
        for (id, payload) in frames {
            assert!(decode(id, &payload, &mut out), "decode 0x{id:03X}");
        }

        assert!((out.engine_rpm - sample.engine_rpm).abs() < 1.0);
        assert_eq!(out.coolant_c, sample.coolant_c);
        assert_eq!(out.oil_c, sample.oil_c);
        assert_eq!(out.gear, sample.gear);
        assert_eq!(out.performance_mode, sample.performance_mode);
        assert_eq!(out.side_stand, sample.side_stand);
        assert_eq!(out.abs_active, sample.abs_active);
        assert_eq!(out.tc_active, sample.tc_active);
        assert_eq!(out.dtc_count, sample.dtc_count);
        assert!((out.ground_speed - sample.ground_speed).abs() < 0.1);
        assert!((out.throttle_pct - sample.throttle_pct).abs() < 0.1);
        assert!((out.odometer - sample.odometer).abs() < 0.1);
    }
}
