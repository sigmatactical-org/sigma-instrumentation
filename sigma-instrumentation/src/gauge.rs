//! Gauge geometry for the RPM tach. Design box is 800x480. The dial sits on the
//! right; the windowing panel fills the left. The needle is a shouldered
//! spear-point with a diamond cross-section (bright spine + two bevels). The
//! swept sector (0 -> current) is shaded red and brightens at redline.
//!
//! Adapted from the Sigma cockpit reference geometry, retuned for the tuned
//! Yamaha XSR900 GP simulation (12 000 r/min sweep, 11 250 redline).
//!
//! **Geometry coupling:** CX/CY/R below must match the hard-coded dial face,
//! boss, and caption positions in `ui/widgets/rpm_dial.slint`. Change both
//! together or the bezel and computed paths will misalign.

use slint::SharedString;

const CX: f32 = 452.0;
const CY: f32 = 246.0;
const R: f32 = 198.0;
const S0_DEG: f32 = 135.0;
const SW_DEG: f32 = 270.0;

pub const MAX_RPM: f32 = 12_000.0;
pub const REDLINE: f32 = 11_250.0;
const REDLINE_ZONE: f32 = 11_000.0;

fn deg2rad(d: f32) -> f32 {
    d * std::f32::consts::PI / 180.0
}

fn angle_for(rpm: f32) -> f32 {
    let f = (rpm / MAX_RPM).clamp(0.0, 1.0);
    S0_DEG + f * SW_DEG
}

fn point(ang_deg: f32, radius: f32) -> (f32, f32) {
    let a = deg2rad(ang_deg);
    (CX + radius * a.cos(), CY + radius * a.sin())
}

fn arc(rpm_from: f32, rpm_to: f32, radius: f32) -> String {
    let a0 = angle_for(rpm_from);
    let a1 = angle_for(rpm_to);
    let (x0, y0) = point(a0, radius);
    let (x1, y1) = point(a1, radius);
    let large = if (a1 - a0).abs() > 180.0 { 1 } else { 0 };
    format!("M {x0:.2} {y0:.2} A {radius:.2} {radius:.2} 0 {large} 1 {x1:.2} {y1:.2}")
}

pub fn track_path() -> SharedString {
    arc(0.0, MAX_RPM, R * 0.97).into()
}

pub fn redline_path() -> SharedString {
    arc(REDLINE_ZONE, MAX_RPM, R * 0.90).into()
}

/// Red swept sector (annular) from 0 to the current rpm — the area the needle
/// has covered. Returns empty near idle so nothing renders at rest.
pub fn swept_path(rpm: f32) -> SharedString {
    let a0 = angle_for(0.0);
    let a1 = angle_for(rpm);
    if (a1 - a0) < 0.5 {
        return SharedString::from("");
    }
    let (rin, rout) = (R * 0.34, R * 0.90);
    let (osx, osy) = point(a0, rout);
    let (oex, oey) = point(a1, rout);
    let (iex, iey) = point(a1, rin);
    let (isx, isy) = point(a0, rin);
    let large = if (a1 - a0) > 180.0 { 1 } else { 0 };
    format!(
        "M {osx:.2} {osy:.2} A {rout:.2} {rout:.2} 0 {large} 1 {oex:.2} {oey:.2} \
         L {iex:.2} {iey:.2} A {rin:.2} {rin:.2} 0 {large} 0 {isx:.2} {isy:.2} Z"
    )
    .into()
}

fn ticks(major_wanted: bool, redline_wanted: bool) -> SharedString {
    let mut s = String::new();
    // 0..24 in 500-rpm steps across a 12 000 sweep
    for k in 0..=24 {
        let rpm = k as f32 * 500.0;
        let major = k % 2 == 0;
        let redline = rpm >= REDLINE_ZONE;
        if redline != redline_wanted {
            continue;
        }
        if !redline && major != major_wanted {
            continue;
        }
        let ang = angle_for(rpm);
        let inner = if major { R * 0.80 } else { R * 0.88 };
        let (xi, yi) = point(ang, inner);
        let (xo, yo) = point(ang, R * 0.97);
        s.push_str(&format!("M {xi:.2} {yi:.2} L {xo:.2} {yo:.2} "));
    }
    s.into()
}

pub fn ticks_major() -> SharedString {
    ticks(true, false)
}

pub fn ticks_minor() -> SharedString {
    ticks(false, false)
}

pub fn ticks_redline() -> SharedString {
    ticks(true, true)
}

/// Shouldered spear-point diamond needle → (left bevel, spine, right bevel, outline).
/// The tip reaches R*0.99 (a little further out than the reference) so it nearly
/// touches the tick ring.
pub fn needle_paths(rpm: f32) -> (SharedString, SharedString, SharedString, SharedString) {
    let a = deg2rad(angle_for(rpm));
    let (dx, dy) = (a.cos(), a.sin());
    let (px, py) = (-a.sin(), a.cos());
    let (rin, rsh, rtip, wb, ws) = (R * 0.30, R * 0.72, R * 0.99, 10.5, 3.5);
    let q = |rad: f32, off: f32| (CX + dx * rad + px * off, CY + dy * rad + py * off);
    let (bl, sl) = (q(rin, wb), q(rsh, wb)); // outer left: base + shoulder
    let (br, sr) = (q(rin, -wb), q(rsh, -wb)); // outer right
    let (cl, cr) = (q(rin, ws), q(rin, -ws)); // spine base
    let tp = q(rtip, 0.0);
    let p = |v: &[(f32, f32)]| {
        let mut s = format!("M {:.2} {:.2}", v[0].0, v[0].1);
        for pt in &v[1..] {
            s.push_str(&format!(" L {:.2} {:.2}", pt.0, pt.1));
        }
        s.push_str(" Z");
        s
    };
    (
        p(&[bl, sl, tp, cl]).into(),     // left bevel
        p(&[cl, tp, cr]).into(),         // spine (highlight)
        p(&[br, sr, tp, cr]).into(),     // right bevel (shadow)
        p(&[bl, sl, tp, sr, br]).into(), // silhouette outline
    )
}

pub struct Numeral {
    pub x: f32,
    pub y: f32,
    pub label: String,
    pub redline: bool,
}

pub fn numerals() -> Vec<Numeral> {
    (0..=12)
        .map(|k| {
            let rpm = k as f32 * 1000.0;
            let (x, y) = point(angle_for(rpm), R * 0.72);
            Numeral {
                x,
                y,
                label: format!("{k}"),
                redline: rpm >= REDLINE_ZONE,
            }
        })
        .collect()
}
