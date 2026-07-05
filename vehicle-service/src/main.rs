//! Co-Pilot vehicle.service — CAN → VSS → Unix socket telemetry.

mod broadcast;
mod can_log;
mod sim;
mod source;

use broadcast::Broadcaster;
use sigma_racer_wingman_telemetry::protocol::{diff_vss, Message, SOCKET_PATH};
use sigma_racer_wingman_telemetry::state::VehicleState;
use source::SignalSource;
use std::env;
use std::fs;
use std::os::unix::net::UnixListener;
use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};

fn main() {
    if let Err(err) = run() {
        eprintln!("sigma-racer-wingman-vehicle: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let socket_path = env::var("CO_PILOT_TELEMETRY_SOCKET")
        .unwrap_or_else(|_| SOCKET_PATH.into());
    prepare_socket(&socket_path)?;

    let demo = env_flag("CO_PILOT_VEHICLE_DEMO");
    let (mut source, mut can_logger) = SignalSource::open(demo)?;
    let mut state = VehicleState::idle();
    source.apply_to(&mut state, &mut can_logger);

    let listener = UnixListener::bind(&socket_path)
        .map_err(|err| format!("bind {socket_path}: {err}"))?;
    listener
        .set_nonblocking(true)
        .map_err(|err| format!("listener nonblocking: {err}"))?;

    let mut broadcaster = Broadcaster::new();
    let started = Instant::now();
    let mut seq: u64 = 0;
    let mut prev = state.clone();
    let mut sample_at = Instant::now();
    let mut heartbeat_at = Instant::now();

    eprintln!(
        "sigma-racer-wingman-vehicle: listening on {socket_path} (source={})",
        source.name()
    );

    loop {
        accept_clients(&listener, &mut broadcaster, &mut seq, &state);

        if sample_at.elapsed() >= Duration::from_millis(50) {
            source.step(Duration::from_millis(50));
            source.apply_to(&mut state, &mut can_logger);

            let patch = diff_vss(&prev, &state);
            if !patch.is_empty() {
                seq += 1;
                broadcaster.send(Message::signal_update(seq, patch).to_line());
                prev = state.clone();
            } else if sample_at.elapsed() >= Duration::from_millis(200) {
                seq += 1;
                broadcaster.send(Message::snapshot(seq, &state).to_line());
            }
            sample_at = Instant::now();
        }

        if heartbeat_at.elapsed() >= Duration::from_secs(1) {
            seq += 1;
            broadcaster.send(
                Message::heartbeat(seq, started.elapsed().as_millis() as u64).to_line(),
            );
            heartbeat_at = Instant::now();
        }

        thread::sleep(Duration::from_millis(5));
    }
}

fn accept_clients(
    listener: &UnixListener,
    broadcaster: &mut Broadcaster,
    seq: &mut u64,
    state: &VehicleState,
) {
    loop {
        match listener.accept() {
            Ok((stream, _)) => {
                *seq += 1;
                let snap = Message::snapshot(*seq, state);
                broadcaster.add(stream, snap.to_line());
            }
            Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => break,
            Err(err) => {
                eprintln!("sigma-racer-wingman-vehicle: accept: {err}");
                break;
            }
        }
    }
}

fn prepare_socket(path: &str) -> Result<(), String> {
    if let Some(dir) = Path::new(path).parent() {
        fs::create_dir_all(dir).map_err(|err| format!("mkdir {}: {err}", dir.display()))?;
    }
    let _ = fs::remove_file(path);
    Ok(())
}

fn env_flag(name: &str) -> bool {
    matches!(
        env::var(name).as_deref(),
        Ok("1") | Ok("true") | Ok("TRUE") | Ok("yes")
    )
}
