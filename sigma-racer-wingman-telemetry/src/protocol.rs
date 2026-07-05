//! NDJSON telemetry envelopes (schemas/telemetry/vehicle-messages.yaml v0.1).

use crate::state::VehicleState;
use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

pub const PROTOCOL_VERSION: &str = "0.1";
pub const SOCKET_PATH: &str = "/run/sigma-racer-wingman/vehicle.sock";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub version: String,
    pub msg: String,
    pub ts: String,
    pub seq: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<HashMap<String, Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vss: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uptime_ms: Option<u64>,
}

impl Message {
    pub fn snapshot(seq: u64, state: &VehicleState) -> Self {
        Self {
            version: PROTOCOL_VERSION.into(),
            msg: "Snapshot".into(),
            ts: now_iso(),
            seq,
            data: Some(state.to_vss_map()),
            event: None,
            vss: None,
            value: None,
            uptime_ms: None,
        }
    }

    pub fn signal_update(seq: u64, data: HashMap<String, Value>) -> Self {
        Self {
            version: PROTOCOL_VERSION.into(),
            msg: "SignalUpdate".into(),
            ts: now_iso(),
            seq,
            data: Some(data),
            event: None,
            vss: None,
            value: None,
            uptime_ms: None,
        }
    }

    pub fn heartbeat(seq: u64, uptime_ms: u64) -> Self {
        Self {
            version: PROTOCOL_VERSION.into(),
            msg: "Heartbeat".into(),
            ts: now_iso(),
            seq,
            data: None,
            event: None,
            vss: None,
            value: None,
            uptime_ms: Some(uptime_ms),
        }
    }

    pub fn to_line(&self) -> String {
        serde_json::to_string(self).expect("telemetry message serializes")
    }

    pub fn parse_line(line: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(line.trim())
    }

    pub fn vss_data(&self) -> Option<&HashMap<String, Value>> {
        self.data.as_ref()
    }
}

pub fn now_iso() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

pub fn diff_vss(prev: &VehicleState, next: &VehicleState) -> HashMap<String, Value> {
    let a = prev.to_vss_map();
    let b = next.to_vss_map();
    b.into_iter()
        .filter(|(k, v)| a.get(k) != Some(v))
        .collect()
}
