//! M7 draft CAN encode/decode via dbc-rs (schemas/can/m7-draft.dbc).

use crate::m7_dbc::m7_dbc;
use crate::state::VehicleState;
use dbc_rs::Dbc;

const ENGINE_STATUS: u32 = 0x0A0;
const THROTTLE_GEAR: u32 = 0x0C0;
const WHEEL_SPEED: u32 = 0x120;
const CHASSIS_ELECTRICAL: u32 = 0x200;
const TRIP_ODOMETER: u32 = 0x220;

pub fn decode_frame(id: u32, data: &[u8], state: &mut VehicleState) -> bool {
    let dbc = m7_dbc();
    let decoded = match dbc.decode(id, data, false) {
        Ok(signals) if !signals.is_empty() => signals,
        _ => return false,
    };

    for signal in decoded.iter() {
        apply_signal(id, signal.name, signal.value, signal.description, state);
    }
    state.refresh_derived();
    true
}

fn apply_signal(
    msg_id: u32,
    name: &str,
    value: f64,
    description: Option<&str>,
    state: &mut VehicleState,
) {
    match (msg_id, name) {
        (ENGINE_STATUS, "EngineRPM") => state.rpm = value as f32,
        (ENGINE_STATUS, "CoolantTemp") => state.coolant_c = value.round() as i16,
        (ENGINE_STATUS, "OilTemp") => state.oil_c = value.round() as i16,
        (ENGINE_STATUS, "Redline") => state.at_redline = value != 0.0,
        (THROTTLE_GEAR, "CurrentGear") => state.gear = value.round() as i8,
        (THROTTLE_GEAR, "PerformanceMode") => {
            state.riding_mode = performance_mode_label(value, description);
        }
        (THROTTLE_GEAR, "SideStand") => state.side_stand = value != 0.0,
        (WHEEL_SPEED, "GroundSpeed") => state.speed = value as f32,
        (WHEEL_SPEED, "LeanAngle") => state.lean_angle = value as f32,
        (WHEEL_SPEED, "LongAccel") => state.gforce = value as f32,
        (CHASSIS_ELECTRICAL, "FuelLevel") => state.fuel_pct = value as f32,
        (CHASSIS_ELECTRICAL, "BatteryVoltage") => state.battery_v = value as f32,
        (CHASSIS_ELECTRICAL, "CanBusLoad") => state.can_load = value.round() as u8,
        (CHASSIS_ELECTRICAL, "AbsActive") => state.abs_active = value != 0.0,
        (CHASSIS_ELECTRICAL, "TcActive") => state.tc_active = value != 0.0,
        (CHASSIS_ELECTRICAL, "DtcCount") => state.dtc = value.round() as u8,
        (TRIP_ODOMETER, "Odometer") => state.odometer = value as f32,
        (TRIP_ODOMETER, "Trip1") => state.trip1 = value as f32,
        (TRIP_ODOMETER, "Trip2") => state.trip2 = value as f32,
        _ => {}
    }
}

fn performance_mode_label(value: f64, description: Option<&str>) -> String {
    if let Some(label) = description {
        if matches!(label, "RAIN" | "STD" | "SPORT" | "TRACK") {
            return label.into();
        }
    }
    match value.round() as u8 {
        0 => "RAIN".into(),
        1 => "STD".into(),
        3 => "TRACK".into(),
        _ => "SPORT".into(),
    }
}

/// Encode current state as simulated CAN frames (for `sim` source).
pub fn encode_sim_frames(state: &VehicleState) -> Vec<(u32, [u8; 8])> {
    let dbc = m7_dbc();
    vec![
        encode_message(dbc, ENGINE_STATUS, &engine_signals(state)),
        encode_message(dbc, THROTTLE_GEAR, &throttle_signals(state)),
        encode_message(dbc, WHEEL_SPEED, &wheel_signals(state)),
        encode_message(dbc, CHASSIS_ELECTRICAL, &chassis_signals(state)),
        encode_message(dbc, TRIP_ODOMETER, &trip_signals(state)),
    ]
}

fn encode_message(dbc: &Dbc, id: u32, signals: &[(&str, f64)]) -> (u32, [u8; 8]) {
    let payload = dbc
        .encode(id, signals, false)
        .unwrap_or_else(|err| panic!("encode 0x{id:03X}: {err}"));
    let mut frame = [0u8; 8];
    let len = payload.len().min(8);
    frame[..len].copy_from_slice(&payload.as_slice()[..len]);
    (id, frame)
}

fn engine_signals(state: &VehicleState) -> [(&str, f64); 4] {
    [
        ("EngineRPM", f64::from(state.rpm)),
        ("CoolantTemp", f64::from(state.coolant_c)),
        ("OilTemp", f64::from(state.oil_c)),
        ("Redline", f64::from(u8::from(state.at_redline))),
    ]
}

fn throttle_signals(state: &VehicleState) -> [(&str, f64); 3] {
    let mode = match state.riding_mode.as_str() {
        "RAIN" => 0.0,
        "STD" => 1.0,
        "TRACK" => 3.0,
        _ => 2.0,
    };
    [
        ("CurrentGear", f64::from(state.gear.max(0))),
        ("PerformanceMode", mode),
        ("SideStand", f64::from(u8::from(state.side_stand))),
    ]
}

fn wheel_signals(state: &VehicleState) -> [(&str, f64); 3] {
    [
        ("GroundSpeed", f64::from(state.speed)),
        ("LeanAngle", f64::from(state.lean_angle)),
        ("LongAccel", f64::from(state.gforce)),
    ]
}

fn chassis_signals(state: &VehicleState) -> [(&str, f64); 6] {
    [
        ("FuelLevel", f64::from(state.fuel_pct)),
        ("BatteryVoltage", f64::from(state.battery_v)),
        ("CanBusLoad", f64::from(state.can_load)),
        ("AbsActive", f64::from(u8::from(state.abs_active))),
        ("TcActive", f64::from(u8::from(state.tc_active))),
        ("DtcCount", f64::from(state.dtc)),
    ]
}

fn trip_signals(state: &VehicleState) -> [(&str, f64); 3] {
    [
        ("Odometer", f64::from(state.odometer)),
        ("Trip1", f64::from(state.trip1)),
        ("Trip2", f64::from(state.trip2)),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_idle() {
        let idle = VehicleState::idle();
        let mut decoded = VehicleState::idle();
        decoded.speed = 0.0;
        for (id, payload) in encode_sim_frames(&idle) {
            decode_frame(id, &payload, &mut decoded);
        }
        assert!((decoded.rpm - idle.rpm).abs() < 1.0);
        assert_eq!(decoded.gear, idle.gear);
        assert_eq!(decoded.side_stand, idle.side_stand);
        assert_eq!(decoded.riding_mode, idle.riding_mode);
    }

    #[test]
    fn dbc_parses() {
        assert_eq!(m7_dbc().messages().len(), 5);
    }
}
