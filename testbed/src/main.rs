//! Interactive testbed — ride simulation, window navigation, and panel testing.

mod sim;

use sigma_instrumentation::{
    configure_window, init_gauge_art, theme, DisplayConfig, SigmaDashboard,
};
use slint::ComponentHandle;
use sim::RideSimulation;
use std::rc::Rc;
use std::time::Duration;

fn main() -> Result<(), slint::PlatformError> {
    let ui = SigmaDashboard::new()?;

    theme::init_from_env(&ui);
    configure_window(
        &ui,
        if cfg!(feature = "virt-panel") {
            DisplayConfig::virt_panel()
        } else {
            DisplayConfig::desktop()
        },
    );
    init_gauge_art(&ui);

    let state = Rc::new(RideSimulation::default());

    {
        let state = state.clone();
        ui.on_rpm_up(move || state.restart_run());
    }
    {
        let state = state.clone();
        ui.on_rpm_down(move || state.force_decel());
    }
    {
        let state = state.clone();
        ui.on_nav_next(move || state.nav_next());
    }
    {
        let state = state.clone();
        ui.on_nav_prev(move || state.nav_prev());
    }
    {
        let state = state.clone();
        ui.on_nav_home(move || state.nav_home());
    }
    {
        let state = state.clone();
        ui.on_nav_select(move |idx| state.nav_select(idx));
    }

    let timer = slint::Timer::default();
    let tick_state = state.clone();
    let tick_ui = ui.as_weak();
    timer.start(
        slint::TimerMode::Repeated,
        Duration::from_millis(16),
        move || {
            if let Some(ui) = tick_ui.upgrade() {
                tick_state.step(&ui);
            }
        },
    );

    ui.run()
}
