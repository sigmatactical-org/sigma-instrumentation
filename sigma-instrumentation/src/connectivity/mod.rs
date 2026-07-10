//! Connectivity window focus model and Slint binding helpers.
//!
//! Face buttons on window 4 (Connectivity):
//! - Previous / Next move the focus highlight (edge Prev/Next leave the window)
//! - Select activates the focused row
//! - Back leaves a list to the main menu, or returns home from the main menu

use slint::{ModelRc, SharedString, VecModel};
use std::rc::Rc;

use crate::{ConnItem, SigmaDashboard};

/// Window index for Connectivity (keep in sync with [`crate::windows`]).
pub const WINDOW: i32 = 4;

/// Main menu / Bluetooth list / Wi-Fi list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Screen {
    #[default]
    Main = 0,
    BtList = 1,
    WifiList = 2,
}

impl Screen {
    pub fn as_i32(self) -> i32 {
        self as i32
    }
}

/// Fixed main-menu rows.
pub const MAIN_BT_POWER: usize = 0;
pub const MAIN_BT_DEVICES: usize = 1;
pub const MAIN_WIFI_POWER: usize = 2;
pub const MAIN_WIFI_NETWORKS: usize = 3;
pub const MAIN_COUNT: usize = 4;

#[derive(Debug, Clone, Default)]
pub struct DeviceRow {
    pub path: String,
    pub title: String,
    pub detail: String,
    pub badge: String,
    pub connected: bool,
    pub paired: bool,
}

#[derive(Debug, Clone, Default)]
pub struct NetworkRow {
    pub path: String,
    pub title: String,
    pub detail: String,
    pub badge: String,
    pub connected: bool,
    pub favorite: bool,
}

#[derive(Debug, Clone, Default)]
pub struct Snapshot {
    pub bt_powered: bool,
    pub bt_connected: bool,
    pub bt_device: String,
    pub bt_battery: i32,
    pub wifi_powered: bool,
    pub wifi_connected: bool,
    pub wifi_ssid: String,
    pub devices: Vec<DeviceRow>,
    pub networks: Vec<NetworkRow>,
    pub status: String,
    pub busy: bool,
    pub available: bool,
}

#[derive(Debug, Clone, Default)]
pub struct Menu {
    pub screen: Screen,
    pub focus: usize,
}

impl Menu {
    pub fn reset(&mut self) {
        self.screen = Screen::Main;
        self.focus = 0;
    }

    pub fn item_count(&self, snap: &Snapshot) -> usize {
        match self.screen {
            Screen::Main => MAIN_COUNT,
            Screen::BtList => snap.devices.len().saturating_add(1), // + Scan
            Screen::WifiList => snap.networks.len().saturating_add(1), // + Rescan
        }
    }

    /// Move focus by `delta`. Returns `Some(window_delta)` when leaving the window
    /// at the first/last edge on the main menu.
    pub fn move_focus(&mut self, snap: &Snapshot, delta: i32) -> Option<i32> {
        let count = self.item_count(snap).max(1);
        if self.screen == Screen::Main {
            if delta < 0 && self.focus == 0 {
                return Some(-1);
            }
            if delta > 0 && self.focus + 1 >= count {
                return Some(1);
            }
        }
        let next = self.focus as i32 + delta;
        self.focus = next.clamp(0, count as i32 - 1) as usize;
        None
    }

    pub fn back(&mut self) -> BackResult {
        match self.screen {
            Screen::Main => BackResult::LeaveWindow,
            Screen::BtList | Screen::WifiList => {
                self.screen = Screen::Main;
                self.focus = 0;
                BackResult::Stay
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackResult {
    Stay,
    LeaveWindow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    ToggleBt,
    OpenBtList,
    ToggleWifi,
    OpenWifiList,
    BtScan,
    WifiScan,
    /// Index into [`Snapshot::devices`].
    SelectDevice(usize),
    /// Index into [`Snapshot::networks`].
    SelectNetwork(usize),
}

impl Menu {
    pub fn select(&mut self, snap: &Snapshot) -> Option<Action> {
        match self.screen {
            Screen::Main => match self.focus {
                MAIN_BT_POWER => Some(Action::ToggleBt),
                MAIN_BT_DEVICES => {
                    self.screen = Screen::BtList;
                    self.focus = 0;
                    Some(Action::OpenBtList)
                }
                MAIN_WIFI_POWER => Some(Action::ToggleWifi),
                MAIN_WIFI_NETWORKS => {
                    self.screen = Screen::WifiList;
                    self.focus = 0;
                    Some(Action::OpenWifiList)
                }
                _ => None,
            },
            Screen::BtList => {
                let n = snap.devices.len();
                if self.focus < n {
                    Some(Action::SelectDevice(self.focus))
                } else {
                    Some(Action::BtScan)
                }
            }
            Screen::WifiList => {
                let n = snap.networks.len();
                if self.focus < n {
                    Some(Action::SelectNetwork(self.focus))
                } else {
                    Some(Action::WifiScan)
                }
            }
        }
    }
}

fn item(title: &str, detail: &str, badge: &str, focused: bool) -> ConnItem {
    ConnItem {
        title: SharedString::from(title),
        detail: SharedString::from(detail),
        badge: SharedString::from(badge),
        focused,
    }
}

pub fn build_items(menu: &Menu, snap: &Snapshot) -> Vec<ConnItem> {
    match menu.screen {
        Screen::Main => {
            let bt_badge = if snap.bt_powered { "ON" } else { "OFF" };
            let wifi_badge = if snap.wifi_powered { "ON" } else { "OFF" };
            let bt_detail = if snap.bt_connected {
                snap.bt_device.as_str()
            } else if snap.bt_powered {
                "No headset connected"
            } else {
                "Radio off"
            };
            let wifi_detail = if snap.wifi_connected {
                snap.wifi_ssid.as_str()
            } else if snap.wifi_powered {
                "Not associated"
            } else {
                "Radio off"
            };
            vec![
                item(
                    "Bluetooth",
                    bt_detail,
                    bt_badge,
                    menu.focus == MAIN_BT_POWER,
                ),
                item(
                    "Devices",
                    "Pair or connect headset",
                    "",
                    menu.focus == MAIN_BT_DEVICES,
                ),
                item(
                    "Wi-Fi",
                    wifi_detail,
                    wifi_badge,
                    menu.focus == MAIN_WIFI_POWER,
                ),
                item(
                    "Networks",
                    "Join a saved or open network",
                    "",
                    menu.focus == MAIN_WIFI_NETWORKS,
                ),
            ]
        }
        Screen::BtList => {
            let mut rows: Vec<ConnItem> = snap
                .devices
                .iter()
                .enumerate()
                .map(|(i, d)| {
                    item(
                        &d.title,
                        &d.detail,
                        &d.badge,
                        menu.focus == i,
                    )
                })
                .collect();
            let scan_i = rows.len();
            rows.push(item(
                "Scan for headsets",
                "Discover nearby Bluetooth devices",
                "",
                menu.focus == scan_i,
            ));
            rows
        }
        Screen::WifiList => {
            let mut rows: Vec<ConnItem> = snap
                .networks
                .iter()
                .enumerate()
                .map(|(i, n)| {
                    item(
                        &n.title,
                        &n.detail,
                        &n.badge,
                        menu.focus == i,
                    )
                })
                .collect();
            let scan_i = rows.len();
            rows.push(item(
                "Rescan networks",
                "Refresh Wi-Fi scan results",
                "",
                menu.focus == scan_i,
            ));
            rows
        }
    }
}

/// Push snapshot + menu state onto the dashboard.
pub fn apply(ui: &SigmaDashboard, menu: &Menu, snap: &Snapshot) {
    ui.set_conn_screen(menu.screen.as_i32());
    ui.set_conn_busy(snap.busy);
    ui.set_bt_powered(snap.bt_powered);
    ui.set_bt_connected(snap.bt_connected);
    ui.set_bt_device(SharedString::from(if snap.bt_device.is_empty() {
        "—"
    } else {
        snap.bt_device.as_str()
    }));
    ui.set_bt_battery(snap.bt_battery);
    ui.set_wifi_powered(snap.wifi_powered);
    ui.set_wifi_connected(snap.wifi_connected);
    ui.set_wifi_ssid(SharedString::from(if snap.wifi_ssid.is_empty() {
        "—"
    } else {
        snap.wifi_ssid.as_str()
    }));

    let status = if !snap.available && snap.status.is_empty() {
        "BlueZ / connman unavailable — connect radios on the device image."
    } else {
        snap.status.as_str()
    };
    ui.set_conn_status(SharedString::from(status));

    let items = build_items(menu, snap);
    ui.set_conn_items(ModelRc::new(VecModel::from(items)));
}

/// Empty idle state for startup before the first poll.
pub fn apply_idle(ui: &SigmaDashboard) {
    let menu = Menu::default();
    let snap = Snapshot {
        status: "Starting connectivity…".into(),
        bt_battery: -1,
        ..Snapshot::default()
    };
    apply(ui, &menu, &snap);
}

/// Shared handle used by nav + poll timers.
pub struct Controller {
    pub menu: Menu,
    pub snap: Snapshot,
}

impl Controller {
    pub fn new() -> Self {
        Self {
            menu: Menu::default(),
            snap: Snapshot {
                bt_battery: -1,
                status: String::new(),
                ..Snapshot::default()
            },
        }
    }

    pub fn paint(&self, ui: &SigmaDashboard) {
        apply(ui, &self.menu, &self.snap);
    }
}

impl Default for Controller {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience for tests / callers that want an Rc.
pub type SharedController = Rc<std::cell::RefCell<Controller>>;
