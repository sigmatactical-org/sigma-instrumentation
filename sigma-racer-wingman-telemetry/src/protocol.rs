//! NDJSON telemetry envelopes (schemas/telemetry/vehicle-messages.yaml v0.1).

use crate::state::VehicleState;
use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

pub const PROTOCOL_VERSION: &str = "0.1";
pub const SOCKET_PATH: &str = "/run/sigma-racer-wingman/vehicle.sock";

/// Full snapshot rate when nothing changed (10 Hz per schema).
pub const SNAPSHOT_INTERVAL_MS: u64 = 100;

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

#[derive(Debug)]
pub enum ParseError {
    Json(serde_json::Error),
    UnsupportedVersion(String),
    UnknownKind(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json(err) => write!(f, "{err}"),
            Self::UnsupportedVersion(v) => write!(f, "unsupported protocol version {v}"),
            Self::UnknownKind(kind) => write!(f, "unknown message kind {kind}"),
        }
    }
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

    pub fn parse_validated(line: &str) -> Result<Self, ParseError> {
        let msg = Self::parse_line(line).map_err(ParseError::Json)?;
        if msg.version != PROTOCOL_VERSION {
            return Err(ParseError::UnsupportedVersion(msg.version));
        }
        match msg.msg.as_str() {
            "Snapshot" | "SignalUpdate" | "Heartbeat" => Ok(msg),
            other => Err(ParseError::UnknownKind(other.into())),
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_wrong_protocol_version() {
        let line = r#"{"version":"9.9","msg":"Snapshot","ts":"t","seq":1,"data":{}}"#;
        match Message::parse_validated(line) {
            Err(ParseError::UnsupportedVersion(v)) => assert_eq!(v, "9.9"),
            other => panic!("expected version error, got {other:?}"),
        }
    }

    #[test]
    fn accepts_snapshot() {
        let line = Message::snapshot(1, &VehicleState::idle()).to_line();
        assert!(Message::parse_validated(&line).is_ok());
    }
}
