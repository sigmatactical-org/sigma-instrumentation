//! Filesystem IPC with the on-device OTA install daemon (`ota-service`).
//!
//! The daemon (root) watches `/run/sigma-ota/request` for a bundle URL and
//! reports progress through `/run/sigma-ota/status`:
//! `idle | downloading | installing | success:<version> | error:<message>`.
//! Both paths are overridable for bench runs via `SIGMA_OTA_REQUEST` /
//! `SIGMA_OTA_STATUS`.

use std::io::Write;
use std::path::PathBuf;

/// Where install requests are written for the daemon.
fn request_path() -> PathBuf {
    std::env::var("SIGMA_OTA_REQUEST")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/run/sigma-ota/request"))
}

/// Where the daemon reports install progress.
fn status_path() -> PathBuf {
    std::env::var("SIGMA_OTA_STATUS")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/run/sigma-ota/status"))
}

/// Hand a bundle URL to the install daemon.
pub fn request_install(bundle_url: &str) -> Result<(), String> {
    let path = request_path();
    let write = || -> std::io::Result<()> {
        let mut f = std::fs::File::create(&path)?;
        writeln!(f, "{bundle_url}")?;
        Ok(())
    };
    write().map_err(|e| format!("OTA daemon unreachable ({}: {e})", path.display()))
}

/// The daemon's current status line, if it is running.
pub fn read_status() -> Option<String> {
    std::fs::read_to_string(status_path())
        .ok()
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
}

/// Human wording for a raw daemon status line; `None` for `idle` (nothing to
/// show) or an unknown format.
pub fn describe(status: &str) -> Option<String> {
    if status == "downloading" {
        Some("Downloading update…".into())
    } else if status == "installing" {
        Some("Installing update — do not power off.".into())
    } else if let Some(rest) = status.strip_prefix("success:") {
        Some(format!("Installed {rest}."))
    } else {
        status
            .strip_prefix("error:")
            .map(|rest| format!("Update failed: {rest}"))
    }
}

/// Whether this status line means an install is still in flight.
pub fn in_flight(status: &str) -> bool {
    status == "downloading" || status == "installing"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn describes_statuses() {
        assert!(describe("idle").is_none());
        assert_eq!(describe("downloading").unwrap(), "Downloading update…");
        assert!(
            describe("success:1.1.0 — reboot to apply")
                .unwrap()
                .contains("1.1.0")
        );
        assert!(
            describe("error:download failed")
                .unwrap()
                .contains("failed")
        );
        assert!(in_flight("installing"));
        assert!(!in_flight("success:1.1.0"));
    }
}
