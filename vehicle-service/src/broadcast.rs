//! Connected telemetry clients.

use std::io::{BufWriter, Write};
use std::os::unix::net::UnixStream;

pub struct Broadcaster {
    clients: Vec<BufWriter<UnixStream>>,
}

impl Broadcaster {
    pub fn new() -> Self {
        Self {
            clients: Vec::new(),
        }
    }

    pub fn add(&mut self, stream: UnixStream, initial: String) {
        let mut writer = BufWriter::new(stream);
        let _ = writeln!(writer, "{initial}");
        let _ = writer.flush();
        self.clients.push(writer);
    }

    pub fn send(&mut self, line: String) {
        self.clients.retain_mut(|client| {
            writeln!(client, "{line}").is_ok() && client.flush().is_ok()
        });
    }
}
