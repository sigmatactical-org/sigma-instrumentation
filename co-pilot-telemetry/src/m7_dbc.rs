//! M7 draft DBC (schemas/can/m7-draft.dbc).

use dbc_rs::Dbc;
use std::sync::OnceLock;

const M7_DBC: &str = include_str!("../dbc/m7-draft.dbc");

static PARSED: OnceLock<Dbc> = OnceLock::new();

/// Parsed M7 draft database (lazy, thread-safe).
pub fn m7_dbc() -> &'static Dbc {
    PARSED.get_or_init(|| {
        Dbc::parse(M7_DBC).expect("m7-draft.dbc must parse")
    })
}
