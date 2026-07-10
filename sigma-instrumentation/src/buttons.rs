//! Physical HMI buttons on the Wingman cluster face.
//!
//! ```text
//!   LEFT                    RIGHT
//!   ┌─────────┐             ┌─────────┐
//!   │ Previous│             │  Back   │
//!   ├─────────┤             ├─────────┤
//!   │  Next   │             │ Select  │
//!   └─────────┘             └─────────┘
//! ```
//!
//! Desktop / testbed keyboard map (same semantics):
//! - `←` Previous · `→` Next · `↑` Back · `↓` Select
//!
//! On Connectivity / Updates, Previous/Next move the focus highlight (leaving
//! at the first/last row continues to the adjacent window). Select activates;
//! Back leaves a list or returns to Systems.

/// Left column, upper button — previous window / item.
pub const PREV: &str = "previous";
/// Left column, lower button — next window / item.
pub const NEXT: &str = "next";
/// Right column, upper button — back / exit to ride (Systems).
pub const BACK: &str = "back";
/// Right column, lower button — select / confirm.
pub const SELECT: &str = "select";
