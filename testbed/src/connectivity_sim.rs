//! Desktop / testbed Connectivity simulator (no BlueZ / connman required).

use sigma_instrumentation::connectivity::{
    self, Action, BackResult, Controller, DeviceRow, NetworkRow, Snapshot, WINDOW as CONN_WINDOW,
};
use sigma_instrumentation::updates::{self as updates_nav, WINDOW as UPDATES_WINDOW};
use sigma_instrumentation::{camera, windows, SigmaDashboard};
use std::cell::{Cell, RefCell};

/// Shared nav + connectivity state for the testbed harness.
pub struct NavState {
    pub window: Cell<i32>,
    pub ctrl: RefCell<Controller>,
}

impl Default for NavState {
    fn default() -> Self {
        let mut ctrl = Controller::new();
        ctrl.snap = sim_snapshot();
        ctrl.snap.available = true;
        ctrl.snap.status = "Simulated radios (testbed)".into();
        Self {
            window: Cell::new(0),
            ctrl: RefCell::new(ctrl),
        }
    }
}

fn sim_snapshot() -> Snapshot {
    Snapshot {
        bt_powered: true,
        bt_connected: true,
        bt_device: "Cardo Packtalk".into(),
        bt_battery: 78,
        wifi_powered: false,
        wifi_connected: false,
        wifi_ssid: String::new(),
        devices: vec![
            DeviceRow {
                path: "sim/cardo".into(),
                title: "Cardo Packtalk".into(),
                detail: "AA:BB:CC:DD:EE:01".into(),
                badge: "CONNECTED".into(),
                connected: true,
                paired: true,
            },
            DeviceRow {
                path: "sim/sena".into(),
                title: "Sena 50S".into(),
                detail: "AA:BB:CC:DD:EE:02".into(),
                badge: "PAIRED".into(),
                connected: false,
                paired: true,
            },
        ],
        networks: vec![
            NetworkRow {
                path: "sim/home".into(),
                title: "Garage-AP".into(),
                detail: "****  idle".into(),
                badge: "SAVED".into(),
                connected: false,
                favorite: true,
            },
            NetworkRow {
                path: "sim/open".into(),
                title: "TrackDay-Guest".into(),
                detail: "***  idle".into(),
                badge: "OPEN".into(),
                connected: false,
                favorite: false,
            },
            NetworkRow {
                path: "sim/secure".into(),
                title: "CafeWifi".into(),
                detail: "**  idle".into(),
                badge: "SECURE".into(),
                connected: false,
                favorite: false,
            },
        ],
        status: "Simulated radios (testbed)".into(),
        busy: false,
        available: true,
    }
}

fn run_sim_action(action: Action, snap: &mut Snapshot) {
    match action {
        Action::ToggleBt => {
            snap.bt_powered = !snap.bt_powered;
            if !snap.bt_powered {
                snap.bt_connected = false;
            }
            snap.status = format!("Bluetooth {}", if snap.bt_powered { "on" } else { "off" });
        }
        Action::ToggleWifi => {
            snap.wifi_powered = !snap.wifi_powered;
            if !snap.wifi_powered {
                snap.wifi_connected = false;
                snap.wifi_ssid.clear();
            }
            snap.status = format!("Wi-Fi {}", if snap.wifi_powered { "on" } else { "off" });
        }
        Action::OpenBtList | Action::BtScan => {
            snap.status = "Scan complete (simulated)".into();
        }
        Action::OpenWifiList | Action::WifiScan => {
            snap.status = "Wi-Fi scan complete (simulated)".into();
        }
        Action::SelectDevice(i) => {
            let Some(was_connected) = snap.devices.get(i).map(|d| d.connected) else {
                return;
            };
            if was_connected {
                if let Some(dev) = snap.devices.get_mut(i) {
                    let title = dev.title.clone();
                    dev.connected = false;
                    dev.badge = if dev.paired {
                        "PAIRED".into()
                    } else {
                        "AVAIL".into()
                    };
                    snap.bt_connected = false;
                    snap.status = format!("Disconnected {title}");
                }
                return;
            }
            for d in &mut snap.devices {
                if d.connected {
                    d.connected = false;
                    d.badge = if d.paired {
                        "PAIRED".into()
                    } else {
                        "AVAIL".into()
                    };
                }
            }
            if let Some(dev) = snap.devices.get_mut(i) {
                let title = dev.title.clone();
                dev.connected = true;
                dev.paired = true;
                dev.badge = "CONNECTED".into();
                snap.bt_connected = true;
                snap.bt_powered = true;
                snap.bt_device = title.clone();
                snap.bt_battery = 78;
                snap.status = format!("Connected {title}");
            }
        }
        Action::SelectNetwork(i) => {
            let Some(net) = snap.networks.get(i) else {
                return;
            };
            if net.badge == "SECURE" && !net.favorite {
                snap.status = "Password required — provision with connmanctl first".into();
                return;
            }
            if net.connected {
                let title = net.title.clone();
                if let Some(net) = snap.networks.get_mut(i) {
                    net.connected = false;
                    net.badge = if net.favorite {
                        "SAVED".into()
                    } else {
                        "OPEN".into()
                    };
                }
                snap.wifi_connected = false;
                snap.wifi_ssid.clear();
                snap.status = format!("Disconnected {title}");
                return;
            }
            let title = net.title.clone();
            for n in &mut snap.networks {
                if n.connected {
                    n.connected = false;
                    n.badge = if n.favorite {
                        "SAVED".into()
                    } else {
                        n.badge.clone()
                    };
                }
            }
            if let Some(net) = snap.networks.get_mut(i) {
                net.connected = true;
                net.badge = "ONLINE".into();
            }
            snap.wifi_powered = true;
            snap.wifi_connected = true;
            snap.wifi_ssid = title.clone();
            snap.status = format!("Connected {title}");
        }
    }
}

impl NavState {
    pub fn paint(&self, ui: &SigmaDashboard) {
        self.ctrl.borrow().paint(ui);
        ui.set_current_window(self.window.get());
    }

    pub fn nav_next(&self, ui: &SigmaDashboard, stopped: bool) {
        self.step(ui, stopped, 1);
    }

    pub fn nav_prev(&self, ui: &SigmaDashboard, stopped: bool) {
        self.step(ui, stopped, -1);
    }

    pub fn nav_back(&self, ui: &SigmaDashboard) {
        let cur = self.window.get();
        if cur == CONN_WINDOW {
            let mut c = self.ctrl.borrow_mut();
            match c.menu.back() {
                BackResult::Stay => {
                    c.paint(ui);
                    return;
                }
                BackResult::LeaveWindow => c.menu.reset(),
            }
        }
        if cur == UPDATES_WINDOW {
            updates_nav::reset_focus(ui);
        }
        self.window.set(0);
        ui.set_current_window(0);
    }

    pub fn nav_select(&self, ui: &SigmaDashboard) {
        let cur = self.window.get();
        if cur == CONN_WINDOW {
            let mut c = self.ctrl.borrow_mut();
            let snap = c.snap.clone();
            if let Some(action) = c.menu.select(&snap) {
                run_sim_action(action, &mut c.snap);
            }
            c.paint(ui);
            return;
        }
        if cur == UPDATES_WINDOW {
            updates_nav::activate_focused(ui);
            return;
        }
        if cur == camera::WINDOW {
            camera::toggle_feed(ui);
        }
    }

    fn step(&self, ui: &SigmaDashboard, stopped: bool, delta: i32) {
        let max = if stopped {
            windows::COUNT - 1
        } else {
            windows::PANEL_MAX
        };
        let cur = self.window.get();
        if cur == CONN_WINDOW {
            let mut c = self.ctrl.borrow_mut();
            let snap = c.snap.clone();
            if let Some(leave) = c.menu.move_focus(&snap, delta) {
                let next = (CONN_WINDOW + leave).clamp(0, max);
                c.menu.reset();
                self.window.set(next);
                ui.set_current_window(next);
                if next == UPDATES_WINDOW {
                    updates_nav::reset_focus(ui);
                }
                return;
            }
            c.paint(ui);
            return;
        }
        if cur == UPDATES_WINDOW {
            if let Some(leave) = updates_nav::move_focus(ui, delta) {
                let next = (UPDATES_WINDOW + leave).clamp(0, max);
                updates_nav::reset_focus(ui);
                self.window.set(next);
                ui.set_current_window(next);
                if next == CONN_WINDOW {
                    self.ctrl.borrow_mut().menu.reset();
                    self.ctrl.borrow_mut().paint(ui);
                }
                return;
            }
            return;
        }
        let next = if stopped {
            (cur + delta).rem_euclid(windows::COUNT)
        } else {
            (cur.clamp(0, windows::PANEL_MAX) + delta).rem_euclid(windows::PANEL_MAX + 1)
        };
        if next == CONN_WINDOW {
            self.ctrl.borrow_mut().menu.reset();
        }
        if next == UPDATES_WINDOW {
            updates_nav::reset_focus(ui);
        }
        self.window.set(next);
        ui.set_current_window(next);
        if next == CONN_WINDOW {
            self.ctrl.borrow_mut().paint(ui);
        }
    }
}

/// Apply initial simulated connectivity props.
pub fn init(ui: &SigmaDashboard, nav: &NavState) {
    connectivity::apply_idle(ui);
    nav.ctrl.borrow_mut().paint(ui);
}
