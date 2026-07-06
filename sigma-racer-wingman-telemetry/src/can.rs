//! Bridge between the shared M7 CAN codec and the telemetry `VehicleState`.
//!
//! The CAN contract itself — message IDs, the `.dbc`, and the frame⇄signal
//! codec — lives in [`sigma_racer_wingman_m7_can`]. This module only maps that
//! crate's neutral [`M7Signals`] onto the `std`-side [`VehicleState`].

use crate::state::VehicleState;
use sigma_racer_wingman_m7_can::{self as m7, M7Signals, PerformanceMode};

/// Decode one CAN frame into `state`. Returns `false` if the frame is not part
/// of the M7 dictionary or fails to decode.
pub fn decode_frame(id: u32, data: &[u8], state: &mut VehicleState) -> bool {
    // Seed from the current state so fields not carried by this message keep
    // their values, then let the decoder overwrite only this frame's signals.
    let mut signals = to_signals(state);
    if !m7::decode(id, data, &mut signals) {
        return false;
    }
    from_signals(&signals, state);
    state.refresh_derived();
    true
}

/// Encode the current state as the full set of simulated CAN frames (used by
/// the `sim` source). Fails soft to no frames if a value is out of range.
pub fn encode_sim_frames(state: &VehicleState) -> Vec<(u32, [u8; 8])> {
    m7::encode_all(&to_signals(state))
        .map(|frames| frames.to_vec())
        .unwrap_or_default()
}

fn to_signals(state: &VehicleState) -> M7Signals {
    M7Signals {
        engine_rpm: state.rpm,
        coolant_c: state.coolant_c,
        oil_c: state.oil_c,
        redline: state.at_redline,
        // `VehicleState` has no throttle field yet; the contract carries it.
        throttle_pct: 0.0,
        gear: state.gear,
        performance_mode: PerformanceMode::from_label(&state.riding_mode).unwrap_or_default(),
        side_stand: state.side_stand,
        ground_speed: state.speed,
        lean_angle: state.lean_angle,
        long_accel: state.gforce,
        fuel_pct: state.fuel_pct,
        battery_v: state.battery_v,
        can_load: state.can_load,
        abs_active: state.abs_active,
        tc_active: state.tc_active,
        dtc_count: state.dtc,
        odometer: state.odometer,
        trip1: state.trip1,
        trip2: state.trip2,
    }
}

fn from_signals(s: &M7Signals, state: &mut VehicleState) {
    state.rpm = s.engine_rpm;
    state.coolant_c = s.coolant_c;
    state.oil_c = s.oil_c;
    state.at_redline = s.redline;
    state.gear = s.gear;
    state.riding_mode = s.performance_mode.as_str().into();
    state.side_stand = s.side_stand;
    state.speed = s.ground_speed;
    state.lean_angle = s.lean_angle;
    state.gforce = s.long_accel;
    state.fuel_pct = s.fuel_pct;
    state.battery_v = s.battery_v;
    state.can_load = s.can_load;
    state.abs_active = s.abs_active;
    state.tc_active = s.tc_active;
    state.dtc = s.dtc_count;
    state.odometer = s.odometer;
    state.trip1 = s.trip1;
    state.trip2 = s.trip2;
    // `throttle_pct` intentionally dropped — no `VehicleState` field for it.
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::m7_dbc::m7_dbc;

    #[test]
    fn dbc_parses() {
        assert_eq!(m7_dbc().messages().len(), 5);
    }

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
}
