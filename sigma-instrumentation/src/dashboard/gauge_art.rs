//! Install computed RPM dial artwork on a dashboard instance.

use slint::{ModelRc, SharedString, VecModel};
use std::rc::Rc;

use crate::gauge::{self, GaugeScale};
use crate::{SigmaDashboard, Tick};

/// Install static dial artwork for `scale` (track, ticks, numerals, caption).
pub fn init_gauge_art(ui: &SigmaDashboard, scale: &GaugeScale) {
    ui.set_track_path(gauge::track_path(scale));
    ui.set_redline_path(gauge::redline_path(scale));
    ui.set_ticks_major(gauge::ticks_major(scale));
    ui.set_ticks_minor(gauge::ticks_minor(scale));
    ui.set_ticks_redline(gauge::ticks_redline(scale));
    ui.set_labels(build_numerals(scale));
    ui.set_rpm_scale_caption(SharedString::from(scale.caption()));
    ui.set_signal_left_path(gauge::turn_signal_left_path());
    ui.set_signal_right_path(gauge::turn_signal_right_path());
}

fn build_numerals(scale: &GaugeScale) -> ModelRc<Tick> {
    let rows: Vec<Tick> = gauge::numerals(scale)
        .into_iter()
        .map(|n| Tick {
            x: n.x,
            y: n.y,
            label: SharedString::from(n.label),
            redline: n.redline,
        })
        .collect();
    ModelRc::new(Rc::new(VecModel::from(rows)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use slint::Model;

    #[test]
    fn installs_thirteen_positioned_numerals_for_default_scale() {
        let labels = build_numerals(&GaugeScale::DEFAULT);
        assert_eq!(labels.row_count(), 13);

        let zero = labels.row_data(0).expect("0k label");
        assert_eq!(zero.label.as_str(), "0");
        assert!(zero.x > 300.0, "x={}", zero.x);
        assert!(zero.y > 300.0, "y={}", zero.y);

        let twelve = labels.row_data(12).expect("12k label");
        assert_eq!(twelve.label.as_str(), "12");
        assert!(twelve.x > 500.0, "x={}", twelve.x);
    }

    #[test]
    fn autoscales_numeral_count_to_max() {
        let scale = GaugeScale::new(8_000.0, 7_500.0);
        let numerals = gauge::numerals(&scale);
        assert_eq!(numerals.len(), 9);
        assert_eq!(numerals.last().expect("top").label, "8");
    }
}
