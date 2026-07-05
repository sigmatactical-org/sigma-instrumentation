//! Display modes — day / dusk / night, each with its own palette and opacity tier.
//!
//! Select via `SIGMA_DISPLAY_MODE` (preferred) or `SIGMA_UI_TONE` (alias):
//!   night (default) · dusk · day

use slint::{Color, Global};

use crate::{SigmaDashboard, SigmaTheme, SigmaTone};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisplayMode {
    Day,
    Dusk,
    Night,
}

impl DisplayMode {
    pub fn parse(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "day" | "bright" | "daylight" => Self::Day,
            "dusk" | "twilight" | "normal" | "default" | "std" => Self::Dusk,
            _ => Self::Night, // night · stealth · dark · unknown
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Day => "day",
            Self::Dusk => "dusk",
            Self::Night => "night",
        }
    }
}

/// Full palette + opacity tier for one display mode.
#[derive(Clone, Copy, Debug)]
pub struct ThemePreset {
    pub mode: DisplayMode,
    // panel readouts
    pub dial_ink: Color,
    pub dial_value: Color,
    pub dial_label: Color,
    pub dial_whisper: Color,
    pub dial_caption: Color,
    pub dial_rule: Color,
    // dial face
    pub dial_face: Color,
    pub dial_gear: Color,
    // dial chrome
    pub dial_bezel: Color,
    pub dial_bezel_border: Color,
    pub dial_face_bg: Color,
    pub dial_gear_ring: Color,
    pub dial_track: Color,
    pub dial_tick_minor: Color,
    pub dial_tick_major: Color,
    pub dial_swept: Color,
    pub needle_left: Color,
    pub needle_right: Color,
    pub needle_spine: Color,
    pub needle_outline: Color,
    // accents
    pub accent_red: Color,
    pub accent_red_dim: Color,
    pub redline_bright: Color,
    pub redline_bright_dim: Color,
    pub redline_needle_tip: Color,
    pub accent_green: Color,
    pub accent_amber: Color,
    pub accent_blue: Color,
    // opacity tier
    pub title_opacity: f32,
    pub divider_opacity: f32,
    pub stat_opacity: f32,
    pub content_opacity: f32,
    pub bar_opacity: f32,
    pub bar_fill_opacity: f32,
    pub chip_off_opacity: f32,
    pub chip_on_opacity: f32,
    pub speed_opacity: f32,
    pub hint_opacity: f32,
    pub compass_opacity: f32,
    pub side_stand_opacity: f32,
    pub toast_opacity: f32,
}

impl ThemePreset {
    pub fn display_mode(self) -> DisplayMode {
        self.mode
    }

    pub fn by_mode(mode: DisplayMode) -> Self {
        match mode {
            DisplayMode::Day => Self::day(),
            DisplayMode::Dusk => Self::dusk(),
            DisplayMode::Night => Self::night(),
        }
    }

    /// Sunlight — high contrast, full panel readability.
    pub fn day() -> Self {
        Self {
            mode: DisplayMode::Day,
            dial_face: rgb(0x9a, 0x9a, 0xa4),
            dial_gear: rgb(0xd0, 0xd0, 0xd8),
            dial_ink: rgb(0x9a, 0x9a, 0xa4),
            dial_value: rgb(0xb4, 0xb4, 0xbc),
            dial_label: rgb(0x70, 0x70, 0x7a),
            dial_whisper: rgb(0x50, 0x50, 0x58),
            dial_caption: rgb(0x68, 0x68, 0x72),
            dial_rule: rgb(0x38, 0x38, 0x40),
            dial_bezel: rgb(0x18, 0x18, 0x1c),
            dial_bezel_border: rgb(0x2a, 0x2a, 0x30),
            dial_face_bg: rgb(0x0c, 0x0c, 0x0e),
            dial_gear_ring: rgb(0x50, 0x50, 0x58),
            dial_track: rgb(0x32, 0x32, 0x38),
            dial_tick_minor: rgb(0x40, 0x40, 0x48),
            dial_tick_major: rgb(0x6a, 0x6a, 0x72),
            dial_swept: rgb(0x8a, 0x28, 0x20),
            needle_left: rgb(0x7a, 0x58, 0x54),
            needle_right: rgb(0x4a, 0x30, 0x2c),
            needle_spine: rgb(0xc8, 0x68, 0x68),
            needle_outline: rgb(0x08, 0x08, 0x0a),
            accent_red: rgb(0xc8, 0x68, 0x68),
            accent_red_dim: rgb(0x9a, 0x48, 0x48),
            redline_bright: rgb(0xff, 0x3a, 0x2e),
            redline_bright_dim: rgb(0xc0, 0x28, 0x18),
            redline_needle_tip: rgb(0xff, 0x80, 0x72),
            accent_green: rgb(0x5a, 0x7a, 0x64),
            accent_amber: rgb(0x7a, 0x6a, 0x50),
            accent_blue: rgb(0x5a, 0x6a, 0x7a),
            title_opacity: 0.90,
            divider_opacity: 0.55,
            stat_opacity: 1.0,
            content_opacity: 1.0,
            bar_opacity: 0.92,
            bar_fill_opacity: 0.78,
            chip_off_opacity: 0.72,
            chip_on_opacity: 1.0,
            speed_opacity: 1.0,
            hint_opacity: 0.88,
            compass_opacity: 0.90,
            side_stand_opacity: 0.95,
            toast_opacity: 0.82,
        }
    }

    /// Twilight — balanced ghost panel, dial still primary.
    pub fn dusk() -> Self {
        Self {
            mode: DisplayMode::Dusk,
            dial_face: rgb(0x8a, 0x8a, 0x92),
            dial_gear: rgb(0xb6, 0xb6, 0xbe),
            dial_ink: rgb(0x6e, 0x6e, 0x76),
            dial_value: rgb(0x86, 0x86, 0x90),
            dial_label: rgb(0x4c, 0x4c, 0x55),
            dial_whisper: rgb(0x31, 0x31, 0x38),
            dial_caption: rgb(0x44, 0x44, 0x4c),
            dial_rule: rgb(0x20, 0x20, 0x24),
            dial_bezel: rgb(0x10, 0x10, 0x14),
            dial_bezel_border: rgb(0x1e, 0x1e, 0x21),
            dial_face_bg: rgb(0x0a, 0x0a, 0x0c),
            dial_gear_ring: rgb(0x3e, 0x3e, 0x46),
            dial_track: rgb(0x26, 0x26, 0x2b),
            dial_tick_minor: rgb(0x33, 0x33, 0x3a),
            dial_tick_major: rgb(0x5a, 0x5a, 0x62),
            dial_swept: rgb(0x7a, 0x1a, 0x14),
            needle_left: rgb(0x6a, 0x4a, 0x48),
            needle_right: rgb(0x3a, 0x24, 0x22),
            needle_spine: rgb(0xb8, 0x58, 0x58),
            needle_outline: rgb(0x00, 0x00, 0x00),
            accent_red: rgb(0xb8, 0x58, 0x58),
            accent_red_dim: rgb(0x68, 0x38, 0x38),
            redline_bright: rgb(0xff, 0x2a, 0x1e),
            redline_bright_dim: rgb(0xb3, 0x18, 0x10),
            redline_needle_tip: rgb(0xff, 0x6b, 0x60),
            accent_green: rgb(0x4a, 0x62, 0x54),
            accent_amber: rgb(0x62, 0x5a, 0x48),
            accent_blue: rgb(0x4a, 0x55, 0x62),
            title_opacity: 0.62,
            divider_opacity: 0.28,
            stat_opacity: 0.91,
            content_opacity: 0.87,
            bar_opacity: 0.72,
            bar_fill_opacity: 0.54,
            chip_off_opacity: 0.52,
            chip_on_opacity: 0.80,
            speed_opacity: 0.87,
            hint_opacity: 0.72,
            compass_opacity: 0.62,
            side_stand_opacity: 0.76,
            toast_opacity: 0.58,
        }
    }

    /// Dark — maximum stealth; panel recedes, dial stays legible.
    pub fn night() -> Self {
        Self {
            mode: DisplayMode::Night,
            dial_face: rgb(0x8a, 0x8a, 0x92),
            dial_gear: rgb(0xb6, 0xb6, 0xbe),
            dial_ink: rgb(0x58, 0x58, 0x60),
            dial_value: rgb(0x6a, 0x6a, 0x72),
            dial_label: rgb(0x38, 0x38, 0x40),
            dial_whisper: rgb(0x24, 0x24, 0x28),
            dial_caption: rgb(0x34, 0x34, 0x3a),
            dial_rule: rgb(0x18, 0x18, 0x1c),
            dial_bezel: rgb(0x0c, 0x0c, 0x0e),
            dial_bezel_border: rgb(0x18, 0x18, 0x1a),
            dial_face_bg: rgb(0x08, 0x08, 0x0a),
            dial_gear_ring: rgb(0x34, 0x34, 0x3a),
            dial_track: rgb(0x20, 0x20, 0x24),
            dial_tick_minor: rgb(0x2a, 0x2a, 0x30),
            dial_tick_major: rgb(0x48, 0x48, 0x50),
            dial_swept: rgb(0x6a, 0x16, 0x12),
            needle_left: rgb(0x5a, 0x3e, 0x3c),
            needle_right: rgb(0x30, 0x1e, 0x1c),
            needle_spine: rgb(0xa8, 0x50, 0x50),
            needle_outline: rgb(0x00, 0x00, 0x00),
            accent_red: rgb(0xb8, 0x58, 0x58),
            accent_red_dim: rgb(0x52, 0x28, 0x28),
            redline_bright: rgb(0xff, 0x2a, 0x1e),
            redline_bright_dim: rgb(0xb3, 0x18, 0x10),
            redline_needle_tip: rgb(0xff, 0x6b, 0x60),
            accent_green: rgb(0x34, 0x44, 0x3c),
            accent_amber: rgb(0x44, 0x3e, 0x34),
            accent_blue: rgb(0x34, 0x3c, 0x44),
            title_opacity: 0.48,
            divider_opacity: 0.18,
            stat_opacity: 0.82,
            content_opacity: 0.76,
            bar_opacity: 0.58,
            bar_fill_opacity: 0.40,
            chip_off_opacity: 0.38,
            chip_on_opacity: 0.68,
            speed_opacity: 0.76,
            hint_opacity: 0.58,
            compass_opacity: 0.48,
            side_stand_opacity: 0.62,
            toast_opacity: 0.44,
        }
    }

    pub fn apply(self, ui: &SigmaDashboard) {
        let t = SigmaTheme::get(ui);

        t.set_dial_face(self.dial_face);
        t.set_dial_gear(self.dial_gear);
        t.set_dial_ink(self.dial_ink);
        t.set_dial_value(self.dial_value);
        t.set_dial_label(self.dial_label);
        t.set_dial_whisper(self.dial_whisper);
        t.set_dial_caption(self.dial_caption);
        t.set_dial_rule(self.dial_rule);

        t.set_dial_bezel(self.dial_bezel);
        t.set_dial_bezel_border(self.dial_bezel_border);
        t.set_dial_face_bg(self.dial_face_bg);
        t.set_dial_gear_ring(self.dial_gear_ring);
        t.set_dial_track(self.dial_track);
        t.set_dial_tick_minor(self.dial_tick_minor);
        t.set_dial_tick_major(self.dial_tick_major);
        t.set_dial_swept(self.dial_swept);
        t.set_needle_left(self.needle_left);
        t.set_needle_right(self.needle_right);
        t.set_needle_spine(self.needle_spine);
        t.set_needle_outline(self.needle_outline);

        t.set_side_stand_fg(self.dial_face);
        t.set_accent_red(self.accent_red);
        t.set_accent_red_dim(self.accent_red_dim);
        t.set_redline_bright(self.redline_bright);
        t.set_redline_bright_dim(self.redline_bright_dim);
        t.set_redline_needle_tip(self.redline_needle_tip);
        t.set_accent_green(self.accent_green);
        t.set_accent_amber(self.accent_amber);
        t.set_accent_blue(self.accent_blue);

        let tone = SigmaTone::get(ui);
        tone.set_title_opacity(self.title_opacity);
        tone.set_divider_opacity(self.divider_opacity);
        tone.set_stat_opacity(self.stat_opacity);
        tone.set_content_opacity(self.content_opacity);
        tone.set_bar_opacity(self.bar_opacity);
        tone.set_bar_fill_opacity(self.bar_fill_opacity);
        tone.set_chip_off_opacity(self.chip_off_opacity);
        tone.set_chip_on_opacity(self.chip_on_opacity);
        tone.set_speed_opacity(self.speed_opacity);
        tone.set_hint_opacity(self.hint_opacity);
        tone.set_compass_opacity(self.compass_opacity);
        tone.set_side_stand_opacity(self.side_stand_opacity);
        tone.set_toast_opacity(self.toast_opacity);
    }
}

fn parse_mode_from_env() -> DisplayMode {
    if let Ok(v) = std::env::var("SIGMA_DISPLAY_MODE") {
        return DisplayMode::parse(&v);
    }
    if let Ok(v) = std::env::var("SIGMA_UI_TONE") {
        return DisplayMode::parse(&v);
    }
    DisplayMode::Night
}

/// Load display mode from the environment and apply. Returns the preset used.
pub fn init_from_env(ui: &SigmaDashboard) -> ThemePreset {
    let preset = ThemePreset::by_mode(parse_mode_from_env());
    preset.apply(ui);
    preset
}

/// Apply a specific display mode (e.g. from a light sensor or user setting).
pub fn apply_mode(ui: &SigmaDashboard, mode: DisplayMode) -> ThemePreset {
    let preset = ThemePreset::by_mode(mode);
    preset.apply(ui);
    preset
}

const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgb_u8(r, g, b)
}
