//! Unix socket telemetry subscriber (sigma-dash).
//!
//! The reader runs on its own thread and **auto-reconnects**: if vehicle.service
//! restarts (or the socket drops), the thread keeps retrying until the client is
//! dropped. Messages are delivered over a bounded channel so a slow UI cannot
//! cause unbounded memory growth — excess frames are dropped (the protocol sends
//! periodic full snapshots, so the UI re-syncs on the next one).

use crate::protocol::{Message, SOCKET_PATH};
use std::io::{BufRead, BufReader, Error, ErrorKind};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender, TryRecvError, TrySendError};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// Bound on queued messages before the reader starts dropping frames.
const CHANNEL_CAPACITY: usize = 512;
/// Backoff between reconnect attempts when the service is down.
const RECONNECT_DELAY: Duration = Duration::from_millis(500);
/// Read timeout so the reader thread periodically observes shutdown requests.
const READ_TIMEOUT: Duration = Duration::from_millis(500);
/// Upper bound on a single NDJSON frame. Anything longer is treated as corrupt,
/// dropped, and the stream is resynced at the next newline.
const MAX_LINE_BYTES: usize = 64 * 1024;

pub struct TelemetryClient {
    rx: Receiver<Message>,
    alive: Arc<AtomicBool>,
    _thread: JoinHandle<()>,
}

impl TelemetryClient {
    /// Connect to vehicle.service; spawns a reader thread. Returns `None` if the
    /// service is not currently reachable (callers may retry later).
    pub fn connect() -> Option<Self> {
        Self::connect_path(default_socket())
    }

    pub fn connect_path(path: impl AsRef<Path>) -> Option<Self> {
        let path = path.as_ref().to_path_buf();
        // Require one successful connection up front so the caller can tell
        // "service present" from "service absent"; later drops auto-reconnect.
        let stream = UnixStream::connect(&path).ok()?;
        configure(&stream);

        let (tx, rx) = mpsc::sync_channel(CHANNEL_CAPACITY);
        let alive = Arc::new(AtomicBool::new(true));
        let alive_thread = Arc::clone(&alive);
        let thread = thread::spawn(move || run(path, Some(stream), tx, alive_thread));
        Some(Self {
            rx,
            alive,
            _thread: thread,
        })
    }

    pub fn try_recv(&self) -> Option<Message> {
        match self.rx.try_recv() {
            Ok(msg) => Some(msg),
            Err(TryRecvError::Empty | TryRecvError::Disconnected) => None,
        }
    }

    pub fn drain(&self) -> impl Iterator<Item = Message> + '_ {
        std::iter::from_fn(|| self.try_recv())
    }
}

impl Drop for TelemetryClient {
    fn drop(&mut self) {
        // Signal the reader thread to stop reconnecting/looping.
        self.alive.store(false, Ordering::Relaxed);
    }
}

fn configure(stream: &UnixStream) {
    let _ = stream.set_read_timeout(Some(READ_TIMEOUT));
}

/// Outcome of reading a single connection.
enum Outcome {
    /// The client was dropped; stop entirely.
    Stop,
    /// The connection ended; reconnect after a delay.
    Reconnect,
}

fn run(path: PathBuf, initial: Option<UnixStream>, tx: SyncSender<Message>, alive: Arc<AtomicBool>) {
    let mut stream = initial;
    while alive.load(Ordering::Relaxed) {
        let connection = match stream.take() {
            Some(s) => s,
            None => match UnixStream::connect(&path) {
                Ok(s) => {
                    configure(&s);
                    s
                }
                Err(_) => {
                    thread::sleep(RECONNECT_DELAY);
                    continue;
                }
            },
        };

        match read_stream(connection, &tx, &alive) {
            Outcome::Stop => return,
            Outcome::Reconnect => {
                if alive.load(Ordering::Relaxed) {
                    thread::sleep(RECONNECT_DELAY);
                }
            }
        }
    }
}

fn read_stream(stream: UnixStream, tx: &SyncSender<Message>, alive: &Arc<AtomicBool>) -> Outcome {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    loop {
        if !alive.load(Ordering::Relaxed) {
            return Outcome::Stop;
        }
        match reader.read_line(&mut line) {
            Ok(0) => return Outcome::Reconnect, // EOF: server closed the socket
            Ok(_) => {
                if !line.ends_with('\n') {
                    // Partial frame (EOF mid-line, or an over-long corrupt frame).
                    if line.len() > MAX_LINE_BYTES {
                        line.clear();
                    }
                    continue;
                }
                match Message::parse_line(&line) {
                    Ok(msg) => match tx.try_send(msg) {
                        Ok(()) => {}
                        // UI is behind: drop this frame, it will catch up on the
                        // next snapshot rather than back-pressure the socket.
                        Err(TrySendError::Full(_)) => {}
                        Err(TrySendError::Disconnected(_)) => return Outcome::Stop,
                    },
                    Err(err) => eprintln!("telemetry: ignore malformed frame: {err}"),
                }
                line.clear();
            }
            Err(err)
                if err.kind() == ErrorKind::WouldBlock || err.kind() == ErrorKind::TimedOut =>
            {
                continue;
            }
            Err(_) => return Outcome::Reconnect,
        }
    }
}

pub fn default_socket() -> String {
    for name in [
        "SIGMA_RACER_WINGMAN_TELEMETRY_SOCKET",
        "CO_PILOT_TELEMETRY_SOCKET",
    ] {
        if let Ok(value) = std::env::var(name) {
            if !value.is_empty() {
                return value;
            }
        }
    }
    SOCKET_PATH.into()
}

pub fn connect_error(path: &Path, err: &Error) -> String {
    format!("telemetry: could not connect to {}: {err}", path.display())
}
