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
#[cfg_attr(feature = "python", pyo3::pyclass)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color(wgpu::Color);

impl Color {
    /// Creates a new color.
    #[inline]
    pub const fn new(r: f64, g: f64, b: f64, a: f64) -> Self {
        Self(wgpu::Color { r, g, b, a })
    }

    /// Creates a new color from a hex value.
    ///
    /// The hex value should be in the format `0xRRGGBBAA`
    #[inline]
    pub const fn from_hex(hex: u32) -> Self {
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
