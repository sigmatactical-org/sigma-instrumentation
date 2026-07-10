//! Riding-mode camera panel — front/rear feed selection.

use crate::SigmaDashboard;

/// Window index for the left-panel camera view (keep in sync with [`crate::windows`]).
pub const WINDOW: i32 = 4;

/// Toggle the camera feed between front and rear.
pub fn toggle_feed(ui: &SigmaDashboard) {
    ui.set_camera_front(!ui.get_camera_front());
}
