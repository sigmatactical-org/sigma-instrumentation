//! Window index table — keep in sync with `road_dashboard.slint` and `app.slint`.
//!
//! Face buttons (see [`crate::buttons`]): Previous / Next / Back / Select.
//!
//! On Connectivity (index 4) and Updates (index 9), Previous/Next move
//! in-window focus; Select activates; Back leaves a list or returns home.
//! Edge Previous/Next leave the focused window.
//!
//! | Index | Window        | Tier        |
//! |-------|---------------|-------------|
//! | 0     | Systems       | left panel  |
//! | 1     | Navigation    | left panel  |
//! | 2     | Compass/GPS   | left panel  |
//! | 3     | Diagnostics   | left panel  |
//! | 4     | Connectivity  | full-screen |
//! | 5     | Camera        | full-screen |
//! | 6     | Maintenance   | full-screen |
//! | 7     | Fuel          | full-screen |
//! | 8     | Security      | full-screen |
//! | 9     | Updates       | full-screen |

/// Last index of glanceable left-panel windows (while moving).
pub const PANEL_MAX: i32 = 3;

/// Total window count.
pub const COUNT: i32 = 10;
