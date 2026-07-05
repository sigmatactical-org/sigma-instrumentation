//! CAN or simulated signal input.

use crate::can_log::CanLogger;
use crate::sim::Simulator;
use sigma_racer_wingman_telemetry::can::encode_sim_frames;
use sigma_racer_wingman_telemetry::state::VehicleState;
use std::time::Duration;

pub enum SignalSource {
    Sim(Simulator),
}

impl SignalSource {
    pub fn open(demo: bool) -> Result<(Self, Option<CanLogger>), String> {
        let source = std::env::var("CO_PILOT_VEHICLE_SOURCE").unwrap_or_else(|_| "sim".into());
        let logger = CanLogger::open();
        match source.as_str() {
            "sim" => Ok((Self::Sim(Simulator::new(demo)), logger)),
            "can" | "socketcan" => Err(
                "CO_PILOT_VEHICLE_SOURCE=can requires a build with can-socket (not in virt image)"
                    .into(),
            ),
            other => Err(format!("unknown CO_PILOT_VEHICLE_SOURCE: {other}")),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Sim(_) => "sim",
        }
    }

    pub fn step(&mut self, dt: Duration) {
        if let Self::Sim(sim) = self {
            sim.step(dt);
        }
    }

    pub fn apply_to(&mut self, state: &mut VehicleState, logger: &mut Option<CanLogger>) {
        if let Self::Sim(sim) = self {
            sim.apply_to(state);
            if let Some(log) = logger {
                log.log_frames(&encode_sim_frames(state));
            }
        }
    }
}
