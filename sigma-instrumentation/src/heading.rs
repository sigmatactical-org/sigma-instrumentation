//! Compass heading helpers shared by cluster and testbed.

/// Eight-point compass label for a heading in degrees.
pub fn heading_label(deg: f32) -> &'static str {
    const DIRS: [&str; 8] = ["N", "NE", "E", "SE", "S", "SW", "W", "NW"];
    let idx = (((deg.rem_euclid(360.0)) / 45.0).round() as usize) % 8;
    DIRS[idx]
}
