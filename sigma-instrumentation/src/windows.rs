//! Window index table — keep in sync with `road_dashboard.slint` and `app.slint` (keys 1–9).
//!
//! | Index | Window        | Tier        | Key |
//! |-------|---------------|-------------|-----|
//! | 0     | Systems       | left panel  | 1   |
//! | 1     | Navigation    | left panel  | 2   |
//! | 2     | Compass/GPS   | left panel  | 3   |
//! | 3     | Diagnostics   | left panel  | 4   |
//! | 4     | Connectivity  | full-screen | 5   |
//! | 5     | Camera        | full-screen | 6   |
//! | 6     | Maintenance   | full-screen | 7   |
//! | 7     | Fuel          | full-screen | 8   |
//! | 8     | Security      | full-screen | 9   |

/// Last index of glanceable left-panel windows (while moving).
pub const PANEL_MAX: i32 = 3;

/// Total window count.
pub const COUNT: i32 = 9;
