//! Poll the sigma-updates catalog and bind results onto the dashboard.
//!
//! Face buttons on the Updates window (index 9): Previous/Next move focus between
//! Check and Install; Select runs the focused action; Back returns home.

use serde::Deserialize;
use slint::{ComponentHandle, SharedString};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use crate::SigmaDashboard;

/// Window index for Updates (keep in sync with [`crate::windows`]).
pub const WINDOW: i32 = 9;

pub const FOCUS_CHECK: i32 = 0;
pub const FOCUS_INSTALL: i32 = 1;
pub const FOCUS_COUNT: i32 = 2;

#[derive(Debug, Clone, Deserialize)]
pub struct ChannelRelease {
    pub channel: String,
    pub version: String,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub install: String,
    #[serde(default)]
    pub bundle_url: String,
}

#[derive(Debug, Clone)]
pub struct UpdatesConfig {
    pub base_url: String,
    pub channel: String,
    pub current_version: String,
}

impl UpdatesConfig {
    pub fn from_env() -> Self {
        Self {
            base_url: std::env::var("SIGMA_UPDATES_URL")
                .unwrap_or_else(|_| "http://updates.sigma.localtest.me:30080".into())
                .trim_end_matches('/')
                .to_owned(),
            channel: std::env::var("SIGMA_UPDATES_CHANNEL")
                .unwrap_or_else(|_| "dev".into()),
            current_version: std::env::var("SIGMA_IMAGE_VERSION")
                .unwrap_or_else(|_| "0.0.0".into()),
        }
    }

    pub fn latest_url(&self) -> String {
        format!("{}/v1/channel/{}/latest", self.base_url, self.channel)
    }
}

pub fn apply_idle(ui: &SigmaDashboard, cfg: &UpdatesConfig) {
    ui.set_update_channel(SharedString::from(cfg.channel.as_str()));
    ui.set_update_current_version(SharedString::from(cfg.current_version.as_str()));
    ui.set_update_available(false);
    ui.set_update_available_version(SharedString::from(""));
    ui.set_update_notes(SharedString::from(""));
    ui.set_update_busy(false);
    ui.set_update_status(SharedString::from(""));
    ui.set_update_focus(FOCUS_CHECK);
}

pub fn apply_release(ui: &SigmaDashboard, cfg: &UpdatesConfig, rel: &ChannelRelease) {
    let newer = rel.version != cfg.current_version;
    ui.set_update_available(newer);
    ui.set_update_available_version(SharedString::from(rel.version.as_str()));
    ui.set_update_notes(SharedString::from(rel.notes.as_str()));
    ui.set_update_status(SharedString::from(if newer {
        "Update found — focus Install and press ↓ to apply."
    } else {
        "Already on the latest catalog version."
    }));
}

pub fn fetch_latest(cfg: &UpdatesConfig) -> Result<ChannelRelease, String> {
    let url = cfg.latest_url();
    let body = ureq::get(&url)
        .timeout(Duration::from_secs(5))
        .call()
        .map_err(|e| format!("updates fetch: {e}"))?
        .into_string()
        .map_err(|e| format!("updates body: {e}"))?;
    serde_json::from_str(&body).map_err(|e| format!("updates json: {e}"))
}

/// Move Updates focus; `Some(window_delta)` when leaving at an edge.
pub fn move_focus(ui: &SigmaDashboard, delta: i32) -> Option<i32> {
    let cur = ui.get_update_focus();
    if delta < 0 && cur <= FOCUS_CHECK {
        return Some(-1);
    }
    if delta > 0 && cur >= FOCUS_INSTALL {
        return Some(1);
    }
    ui.set_update_focus((cur + delta).clamp(FOCUS_CHECK, FOCUS_INSTALL));
    None
}

pub fn reset_focus(ui: &SigmaDashboard) {
    ui.set_update_focus(FOCUS_CHECK);
}

/// Activate the focused Updates row (Check or Install).
pub fn activate_focused(ui: &SigmaDashboard) {
    match ui.get_update_focus() {
        FOCUS_INSTALL => ui.invoke_install_updates(),
        _ => ui.invoke_check_updates(),
    }
}

/// Wire Check / Install callbacks and a slow background poll.
pub fn start(ui: &SigmaDashboard, cfg: UpdatesConfig) {
    apply_idle(ui, &cfg);
    let last = Rc::new(RefCell::new(None::<ChannelRelease>));

    {
        let cfg = cfg.clone();
        let last = last.clone();
        let ui_weak = ui.as_weak();
        ui.on_check_updates(move || {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            if ui.get_update_busy() {
                return;
            }
            ui.set_update_busy(true);
            ui.set_update_status(SharedString::from("Searching catalog…"));
            match fetch_latest(&cfg) {
                Ok(rel) => {
                    *last.borrow_mut() = Some(rel.clone());
                    apply_release(&ui, &cfg, &rel);
                    if rel.version != cfg.current_version {
                        ui.set_update_focus(FOCUS_INSTALL);
                    }
                }
                Err(err) => {
                    ui.set_update_status(SharedString::from(err));
                    ui.set_update_available(false);
                }
            }
            ui.set_update_busy(false);
        });
    }

    {
        let cfg = cfg.clone();
        let last = last.clone();
        let ui_weak = ui.as_weak();
        ui.on_install_updates(move || {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };
            if !ui.get_update_available() || ui.get_update_busy() {
                ui.set_update_status(SharedString::from(
                    "No update to install — run Check for Updates first.",
                ));
                return;
            }
            ui.set_update_busy(true);
            let rel = last.borrow().clone();
            let bundle = rel
                .as_ref()
                .map(|r| r.bundle_url.as_str())
                .filter(|u| !u.is_empty())
                .map(str::to_owned)
                .unwrap_or_else(|| {
                    format!(
                        "{}/v1/channel/{}/bundle/{}.raucb",
                        cfg.base_url,
                        cfg.channel,
                        ui.get_update_available_version()
                    )
                });
            let version = ui.get_update_available_version();
            // RAUC client integration lands with the A/B updater service.
            // Until then, surface a clear apply path so HMI + catalog are testable.
            ui.set_update_status(SharedString::from(format!(
                "Applying {version} from {bundle} — RAUC install hooks next; reboot after apply."
            )));
            ui.set_update_busy(false);
        });
    }

    // Initial + periodic check (catalog may be offline in the lab).
    let weak = ui.as_weak();
    let cfg_tick = cfg.clone();
    let last_tick = last.clone();
    let timer = slint::Timer::default();
    timer.start(
        slint::TimerMode::Repeated,
        Duration::from_secs(60),
        move || {
            let Some(ui) = weak.upgrade() else {
                return;
            };
            if let Ok(rel) = fetch_latest(&cfg_tick) {
                *last_tick.borrow_mut() = Some(rel.clone());
                apply_release(&ui, &cfg_tick, &rel);
            }
        },
    );
    std::mem::forget(timer);

    // One immediate check shortly after start.
    let weak = ui.as_weak();
    let cfg_once = cfg;
    let last_once = last;
    slint::Timer::single_shot(Duration::from_secs(2), move || {
        if let Some(ui) = weak.upgrade()
            && let Ok(rel) = fetch_latest(&cfg_once)
        {
            *last_once.borrow_mut() = Some(rel.clone());
            apply_release(&ui, &cfg_once, &rel);
        }
    });
}
