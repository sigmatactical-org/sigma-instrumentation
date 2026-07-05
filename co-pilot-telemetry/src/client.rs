//! Unix socket telemetry subscriber (sigma-dash).

use crate::protocol::{Message, SOCKET_PATH};
use std::io::{BufRead, BufReader, Error, ErrorKind};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread::{self, JoinHandle};
use std::time::Duration;

pub struct TelemetryClient {
    rx: Receiver<Message>,
    _thread: JoinHandle<()>,
}

impl TelemetryClient {
    /// Connect to vehicle.service; spawns a reader thread. Returns `None` if unavailable.
    pub fn connect() -> Option<Self> {
        Self::connect_path(default_socket())
    }

    pub fn connect_path(path: impl AsRef<Path>) -> Option<Self> {
        let stream = match UnixStream::connect(path.as_ref()) {
            Ok(s) => s,
            Err(_) => return None,
        };
        let _ = stream.set_read_timeout(Some(Duration::from_millis(500)));
        let (tx, rx) = mpsc::channel();
        let thread = thread::spawn(move || read_loop(stream, tx));
        Some(Self {
            rx,
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

fn read_loop(stream: UnixStream, tx: mpsc::Sender<Message>) {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => match Message::parse_line(&line) {
                Ok(msg) => {
                    if tx.send(msg).is_err() {
                        break;
                    }
                }
                Err(err) => {
                    eprintln!("telemetry: ignore malformed frame: {err}");
                }
            },
            Err(err) if err.kind() == ErrorKind::WouldBlock || err.kind() == ErrorKind::TimedOut => {
                continue;
            }
            Err(_) => break,
        }
    }
}

pub fn default_socket() -> String {
    std::env::var("CO_PILOT_TELEMETRY_SOCKET").unwrap_or_else(|_| SOCKET_PATH.into())
}

pub fn connect_error(path: &Path, err: &Error) -> String {
    format!(
        "telemetry: could not connect to {}: {err}",
        path.display()
    )
}
