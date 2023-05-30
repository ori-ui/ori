use std::{
    fmt::Display,
    ops::{Add, AddAssign, Mul},
};

use bytemuck::{Pod, Zeroable};
use glam::Vec4;

/// A color with red, green, blue and alpha components in linear space.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

#[allow(missing_docs)]
impl Color {
    pub const TRANSPARENT: Self = Self::rgba(0.0, 0.0, 0.0, 0.0);
    pub const BLACK: Self = Self::rgb(0.0, 0.0, 0.0);
    pub const WHITE: Self = Self::rgb(1.0, 1.0, 1.0);

    pub const RED: Self = Self::rgb(1.0, 0.0, 0.0);
    pub const GREEN: Self = Self::rgb(0.0, 1.0, 0.0);
    pub const BLUE: Self = Self::rgb(0.0, 0.0, 1.0);

    pub const YELLOW: Self = Self::rgb(1.0, 1.0, 0.0);
    pub const CYAN: Self = Self::rgb(0.0, 1.0, 1.0);
    pub const MAGENTA: Self = Self::rgb(1.0, 0.0, 1.0);
}

impl Color {
    /// Returns a new color with the given red, green, blue and alpha components.
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Returns a new color with the given red, green and blue components.
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    /// Returns a new color with the given red, green, blue and alpha components.
    pub fn rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self::rgba(
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a as f32 / 255.0,
        )
    }

    /// Returns a new color with the given red, green and blue components.
    pub fn rgb8(r: u8, g: u8, b: u8) -> Self {
        Self::rgba8(r, g, b, 255)
    }

    /// Try to parse a color from a hex string.
    pub fn try_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');

        let mut color = Self::BLACK;

        match hex.len() {
            2 => {
                color.r = u8::from_str_radix(hex, 16).ok()? as f32 / 255.0;
                color.g = color.r;
                color.b = color.r;
            }
            3 => {
                color.r = u8::from_str_radix(&hex[0..1], 16).ok()? as f32 / 15.0;
                color.g = u8::from_str_radix(&hex[1..2], 16).ok()? as f32 / 15.0;
                color.b = u8::from_str_radix(&hex[2..3], 16).ok()? as f32 / 15.0;
            }
            4 => {
                color.r = u8::from_str_radix(&hex[0..1], 16).ok()? as f32 / 15.0;
                color.g = u8::from_str_radix(&hex[1..2], 16).ok()? as f32 / 15.0;
                color.b = u8::from_str_radix(&hex[2..3], 16).ok()? as f32 / 15.0;
                color.a = u8::from_str_radix(&hex[3..4], 16).ok()? as f32 / 15.0;
            }
            6 => {
                color.r = u8::from_str_radix(&hex[0..2], 16).ok()? as f32 / 255.0;
                color.g = u8::from_str_radix(&hex[2..4], 16).ok()? as f32 / 255.0;
                color.b = u8::from_str_radix(&hex[4..6], 16).ok()? as f32 / 255.0;
            }
            8 => {
                color.r = u8::from_str_radix(&hex[0..2], 16).ok()? as f32 / 255.0;
                color.g = u8::from_str_radix(&hex[2..4], 16).ok()? as f32 / 255.0;
                color.b = u8::from_str_radix(&hex[4..6], 16).ok()? as f32 / 255.0;
                color.a = u8::from_str_radix(&hex[6..8], 16).ok()? as f32 / 255.0;
            }
            _ => return None,
        }

        Some(color)
    }

    /// Parse a color from a hex string.
    ///
    /// # Panics
    /// - If the string is not a valid hex color.
    pub fn hex(hex: &str) -> Self {
        Self::try_hex(hex).expect("Invalid hex color")
    }

    /// Convert the color to a hex string.
    pub fn to_hex(self) -> String {
        format!(
            "#{:02x}{:02x}{:02x}",
            (self.r * 255.0) as u8,
            (self.g * 255.0) as u8,
            (self.b * 255.0) as u8,
        )
    }

    /// Returns true if the color is translucent.
    pub fn is_translucent(self) -> bool {
        self.a < 1.0
    }

    /// Convert the color to sRGB.
    ///
    /// See <https://en.wikipedia.org/wiki/SRGB>.
    pub fn to_srgb(self) -> [f32; 4] {
        [self.r.powf(2.2), self.g.powf(2.2), self.b.powf(2.2), self.a]
    }
}

impl From<Color> for [f32; 4] {
    fn from(val: Color) -> Self {
        [val.r, val.g, val.b, val.a]
    }
}

impl From<[f32; 4]> for Color {
    fn from([r, g, b, a]: [f32; 4]) -> Self {
        Self { r, g, b, a }
    }
}

impl From<Color> for (f32, f32, f32, f32) {
    fn from(val: Color) -> Self {
        (val.r, val.g, val.b, val.a)
    }
}

impl From<(f32, f32, f32, f32)> for Color {
    fn from((r, g, b, a): (f32, f32, f32, f32)) -> Self {
        Self { r, g, b, a }
    }
}

impl From<Color> for Vec4 {
    fn from(val: Color) -> Self {
        Vec4::new(val.r, val.g, val.b, val.a)
    }
}

impl From<Vec4> for Color {
    fn from(vec: Vec4) -> Self {
        Self::rgba(vec.x, vec.y, vec.z, vec.w)
    }
}

impl Mul<f32> for Color {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            r: self.r * rhs,
            g: self.g * rhs,
            b: self.b * rhs,
            a: self.a * rhs,
        }
    }
}

impl Mul for Color {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            r: self.r * rhs.r,
            g: self.g * rhs.g,
            b: self.b * rhs.b,
            a: self.a * rhs.a,
        }
    }
}

impl Add for Color {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            r: self.r + rhs.r,
            g: self.g + rhs.g,
            b: self.b + rhs.b,
            a: self.a + rhs.a,
        }
    }
}

impl AddAssign for Color {
    fn add_assign(&mut self, rhs: Self) {
        self.r += rhs.r;
        self.g += rhs.g;
        self.b += rhs.b;
        self.a += rhs.a;
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "rgba({}, {}, {}, {})", self.r, self.g, self.b, self.a)
    }
}
