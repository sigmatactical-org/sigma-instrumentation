//! Co-Pilot vehicle telemetry — VSS state, CAN decode (M7 draft), JSON/NDJSON IPC.

pub mod can;
pub mod client;
pub mod m7_dbc;
pub mod protocol;
pub mod state;

pub use client::TelemetryClient;
pub use protocol::{Message, SOCKET_PATH};
pub use state::VehicleState;
