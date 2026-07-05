//! Tier-1 vehicle state keyed by VSS paths.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

pub const REDLINE_RPM: f32 = 11_250.0;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VehicleState {
    pub speed: f32,
    pub rpm: f32,
    pub gear: i8,
    pub at_redline: bool,
    pub side_stand: bool,
    pub riding_mode: String,
    pub fuel_pct: f32,
    pub coolant_c: i16,
    pub oil_c: i16,
    pub odometer: f32,
    pub trip1: f32,
    pub trip2: f32,
    pub lean_angle: f32,
    pub gforce: f32,
    pub battery_v: f32,
    pub can_load: u8,
    pub dtc: u8,
    pub abs_active: bool,
    pub tc_active: bool,
}

impl VehicleState {
    pub fn idle() -> Self {
        Self {
            speed: 0.0,
            rpm: 1_200.0,
            gear: 0,
            at_redline: false,
            side_stand: true,
            riding_mode: "SPORT".into(),
            fuel_pct: 62.0,
            coolant_c: 42,
            oil_c: 52,
            odometer: 1_245.0,
            trip1: 137.4,
            trip2: 42.1,
            lean_angle: 0.0,
            gforce: 0.0,
            battery_v: 13.1,
            can_load: 8,
            dtc: 0,
            abs_active: false,
            tc_active: false,
        }
    }

    pub fn refresh_derived(&mut self) {
        self.at_redline = self.rpm >= REDLINE_RPM;
    }

    pub fn to_vss_map(&self) -> HashMap<String, Value> {
        HashMap::from([
            (
                "Vehicle.Speed".into(),
                json!(self.speed.round() as i64),
            ),
            (
                "Vehicle.Powertrain.CombustionEngine.Speed".into(),
                json!(self.rpm.round() as i64),
            ),
            (
                "Vehicle.Powertrain.Transmission.CurrentGear".into(),
                json!(self.gear),
            ),
            (
                "Vehicle.Powertrain.CombustionEngine.IsRedline".into(),
                json!(self.at_redline),
            ),
            (
                "Vehicle.Body.IsSideStandEngaged".into(),
                json!(self.side_stand),
            ),
            (
                "Vehicle.Powertrain.Transmission.PerformanceMode".into(),
                json!(self.riding_mode),
            ),
            ("Vehicle.FuelSystem.Level".into(), json!(self.fuel_pct)),
            (
                "Vehicle.OBD.CoolantTemperature".into(),
                json!(self.coolant_c),
            ),
            ("Vehicle.OBD.OilTemperature".into(), json!(self.oil_c)),
            ("Vehicle.TraveledDistance".into(), json!(self.odometer)),
            ("Vehicle.TripMeter1".into(), json!(self.trip1)),
            ("Vehicle.TripMeter2".into(), json!(self.trip2)),
            (
                "Vehicle.Acceleration.Lateral".into(),
                json!(self.lean_angle),
            ),
            (
                "Vehicle.Acceleration.Longitudinal".into(),
                json!(self.gforce),
            ),
            (
                "Vehicle.ElectricalSystem.Battery.Voltage".into(),
                json!(self.battery_v),
            ),
            (
                "Vehicle.Cabin.Infotainment.CanBusLoad".into(),
                json!(self.can_load),
            ),
            ("Vehicle.OBD.DTCCount".into(), json!(self.dtc)),
            ("Vehicle.ADAS.ABS.IsActive".into(), json!(self.abs_active)),
            ("Vehicle.ADAS.TCS.IsActive".into(), json!(self.tc_active)),
        ])
    }

    pub fn apply_vss(&mut self, path: &str, value: &Value) {
        match path {
            "Vehicle.Speed" => self.speed = json_f32(value),
            "Vehicle.Powertrain.CombustionEngine.Speed" => self.rpm = json_f32(value),
            "Vehicle.Powertrain.Transmission.CurrentGear" => self.gear = json_i8(value),
            "Vehicle.Powertrain.CombustionEngine.IsRedline" => {
                self.at_redline = json_bool(value)
            }
            "Vehicle.Body.IsSideStandEngaged" => self.side_stand = json_bool(value),
            "Vehicle.Powertrain.Transmission.PerformanceMode" => {
                if let Some(s) = value.as_str() {
                    self.riding_mode = s.into();
                }
            }
            "Vehicle.FuelSystem.Level" => self.fuel_pct = json_f32(value),
            "Vehicle.OBD.CoolantTemperature" => self.coolant_c = json_i16(value),
            "Vehicle.OBD.OilTemperature" => self.oil_c = json_i16(value),
            "Vehicle.TraveledDistance" => self.odometer = json_f32(value),
            "Vehicle.TripMeter1" => self.trip1 = json_f32(value),
            "Vehicle.TripMeter2" => self.trip2 = json_f32(value),
            "Vehicle.Acceleration.Lateral" => self.lean_angle = json_f32(value),
            "Vehicle.Acceleration.Longitudinal" => self.gforce = json_f32(value),
            "Vehicle.ElectricalSystem.Battery.Voltage" => self.battery_v = json_f32(value),
            "Vehicle.Cabin.Infotainment.CanBusLoad" => self.can_load = json_u8(value),
            "Vehicle.OBD.DTCCount" => self.dtc = json_u8(value),
            "Vehicle.ADAS.ABS.IsActive" => self.abs_active = json_bool(value),
            "Vehicle.ADAS.TCS.IsActive" => self.tc_active = json_bool(value),
            _ => {}
        }
    }

    pub fn apply_vss_map(&mut self, data: &HashMap<String, Value>) {
        for (path, value) in data {
            self.apply_vss(path, value);
        }
        self.refresh_derived();
    }
}

fn json_f32(v: &Value) -> f32 {
    v.as_f64().unwrap_or(0.0) as f32
}

fn json_i8(v: &Value) -> i8 {
    v.as_i64().unwrap_or(0).clamp(-128, 127) as i8
}

fn json_i16(v: &Value) -> i16 {
    v.as_i64().unwrap_or(0).clamp(-32768, 32767) as i16
}

fn json_u8(v: &Value) -> u8 {
    v.as_u64().unwrap_or(0).min(255) as u8
}

fn json_bool(v: &Value) -> bool {
    v.as_bool().unwrap_or(false)
}
