//! Color representation.

use std::ops::{Deref, DerefMut};

/// Helper macro for creating colors.
#[macro_export]
macro_rules! color {
    // Creates a new color from r, g, b values.
    ($r:expr, $g:expr, $b:expr) => {
        Color::new($r, $g, $b, 1.0)
    };
    // Creates a new color from r, g, b, a values.
    ($r:expr, $g:expr, $b:expr, $a:expr) => {
        Color::new($r, $g, $b, $a)
    };
    // Creates a new color from a hex value.
    ($hex:expr) => {
        Color::from_hex($hex)
    };
    // Creates a new color from a hex value with a custom alpha.
    ($hex:expr, $a:expr) => {
        Color::from_hex($hex).with_alpha($a)
    };
}

/// Linear color representation.
#[pyo3::pyclass]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color(wgpu::Color);

/// A set of predefined colors.
pub const COLORS: [Color; 16] = [
    color!(0.02122, 0.02122, 0.02732), // Dark grey
    color!(0.14413, 0.09306, 0.16203), // Purplish grey
    color!(0.67954, 0.58408, 0.52100), // Very light pink
    color!(0.86316, 0.25415, 0.23455), // Peachy pink
    color!(0.43415, 0.39157, 0.76052), // Light periwinkle
    color!(0.43415, 0.59062, 0.76815), // Cloudy blue
    color!(0.48515, 0.75294, 0.70110), // Ice blue
    color!(0.92158, 0.41789, 0.76815), // Light lavender
    color!(0.25818, 0.38643, 0.25415), // Greenish grey
    color!(0.47932, 0.82279, 0.30947), // Washed out green
    color!(0.82279, 0.92158, 0.35640), // Light khaki
    color!(0.97345, 0.80695, 0.57758), // Pale
    color!(0.68669, 0.38643, 0.26636), // Pinkish tan
    color!(0.94731, 0.57112, 0.24620), // Very light brown
    color!(0.99110, 0.94731, 0.37124), // Buff
    color!(0.99110, 0.93869, 0.78354), // Off white
];

impl Color {
    pub const DARK_GREY: Self = COLORS[0];
    pub const PURPLISH_GREY: Self = COLORS[1];
    pub const VERY_LIGHT_PINK: Self = COLORS[2];
    pub const PEACHY_PINK: Self = COLORS[3];
    pub const LIGHT_PERIWINKLE: Self = COLORS[4];
    pub const CLOUDY_BLUE: Self = COLORS[5];
    pub const ICE_BLUE: Self = COLORS[6];
    pub const LIGHT_LAVENDER: Self = COLORS[7];
    pub const GREENISH_GREY: Self = COLORS[8];
    pub const WASHED_OUT_GREEN: Self = COLORS[9];
    pub const LIGHT_KHAKI: Self = COLORS[10];
    pub const PALE: Self = COLORS[11];
    pub const PINKISH_TAN: Self = COLORS[12];
    pub const VERY_LIGHT_BROWN: Self = COLORS[13];
    pub const BUFF: Self = COLORS[14];
    pub const OFF_WHITE: Self = COLORS[15];
    pub const WHITE: Self = color!(1.0, 1.0, 1.0);
    pub const BLACK: Self = color!(0.0, 0.0, 0.0);

    /// Creates a new color.
    #[inline]
    pub const fn new(r: f64, g: f64, b: f64, a: f64) -> Self {
        Self(wgpu::Color { r, g, b, a })
    }

    /// Creates a new color from a hex value.
    ///
    /// The hex value should be in the format `0xRRGGBBAA`
    #[inline]
    pub fn from_hex(hex: u32) -> Self {
        Self::new(
            ((hex >> 24) & 0xFF) as f64 / 255.0,
            ((hex >> 16) & 0xFF) as f64 / 255.0,
            ((hex >> 8) & 0xFF) as f64 / 255.0,
            (hex & 0xFF) as f64 / 255.0,
        )
    }

    /// Creates a new color from the current color with a new alpha value.
    #[inline]
    pub const fn with_alpha(&self, alpha: f64) -> Self {
        Self::new(self.0.r, self.0.g, self.0.b, alpha)
    }

    /// Creates a new color from a hex string.
    ///
    /// The hex string should be in the format `RRGGBBAA`. It is case
    /// insensitive and can optionally start with `0x` or `#`.
    ///
    /// If the string is invalid, the color will be black.
    #[inline]
    pub fn from_hex_str(hex: &str) -> Self {
        let hex = hex.trim_start_matches(|c| c == '#' || c == '0' || c == 'x' || c == 'X');
        let hex = u32::from_str_radix(hex, 16).unwrap_or(0);
        Self::from_hex(hex)
    }
}

impl Deref for Color {
    type Target = wgpu::Color;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Color {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Color> for wgpu::Color {
    fn from(c: Color) -> Self {
        c.0
    }
}

impl From<Color> for [f64; 4] {
    fn from(c: Color) -> Self {
        [c.0.r, c.0.g, c.0.b, c.0.a]
    }
}

impl From<Color> for [f32; 4] {
    fn from(c: Color) -> Self {
        [c.0.r as f32, c.0.g as f32, c.0.b as f32, c.0.a as f32]
    }
}

impl From<Color> for [u8; 4] {
    fn from(c: Color) -> Self {
        [
            (c.0.r * 255.0) as u8,
            (c.0.g * 255.0) as u8,
            (c.0.b * 255.0) as u8,
            (c.0.a * 255.0) as u8,
        ]
    }
}

// Python interface
#[cfg(feature = "python")]
#[pyo3::pymethods]
impl Color {
    #[new]
    pub fn new_py(r: f64, g: f64, b: f64, a: f64) -> Self {
        Self::new(r, g, b, a)
    }
}
