use std::{
    fmt::Display,
    ops::{Add, AddAssign, Mul},
};

/// Create a new color, with the given `red`, `green` and `blue` components.
pub fn rgb(r: f32, g: f32, b: f32) -> Color {
    Color::rgb(r, g, b)
}

/// Create a new color, with the given `red`, `green`, `blue` and alpha components.
pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
    Color::rgba(r, g, b, a)
}

/// Create a new color, with the given `hue`, `saturation` and `lightness` components.
pub fn hsl(h: f32, s: f32, l: f32) -> Color {
    Color::hsl(h, s, l)
}

/// Create a new color, with the given `hue`, `saturation`, `lightness` and alpha components.
pub fn hsla(h: f32, s: f32, l: f32, a: f32) -> Color {
    Color::hsla(h, s, l, a)
}

/// Create a new color, with the given `lightness`, `a` and `b` components.
pub fn oklab(l: f32, a: f32, b: f32) -> Color {
    Color::oklab(l, a, b)
}

/// Create a new color, with the given `lightness`, `a`, `b` and alpha components.
pub fn oklaba(l: f32, a: f32, b: f32, alpha: f32) -> Color {
    Color::oklaba(l, a, b, alpha)
}

/// Create a new color, with the given hex string.
pub fn hex(hex: &str) -> Color {
    Color::hex(hex)
}

/// A color with red, green, blue and alpha components.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Color {
    /// The red component of the color.
    pub r: f32,
    /// The green component of the color.
    pub g: f32,
    /// The blue component of the color.
    pub b: f32,
    /// The alpha component of the color.
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
    /// Create a new color with the given red, green, blue and alpha components.
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Create a new color with the given red, green and blue components.
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self::rgba(r, g, b, 1.0)
    }

    /// Create a new color with the given gray component.
    pub const fn grayscale(g: f32) -> Self {
        Self::rgb(g, g, g)
    }

    /// Get the red component as an 8 bit integer.
    pub fn r8(&self) -> u8 {
        (self.r * 255.0) as u8
    }

    /// Get the green component as an 8 bit integer.
    pub fn g8(&self) -> u8 {
        (self.g * 255.0) as u8
    }

    /// Get the blue component as an 8 bit integer.
    pub fn b8(&self) -> u8 {
        (self.b * 255.0) as u8
    }

    /// Get the alpha component as an 8 bit integer.
    pub fn a8(&self) -> u8 {
        (self.a * 255.0) as u8
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

    /// Returns a new color with the given hue, saturation, lightness and alpha components.
    ///
    /// See <https://en.wikipedia.org/wiki/HSL_and_HSV>.
    pub fn hsla(h: f32, s: f32, l: f32, a: f32) -> Self {
        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = l - c / 2.0;

        let (r, g, b) = match (h / 60.0) as u8 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };

        Self::rgba(r + m, g + m, b + m, a)
    }

    /// Returns a new color with the given hue, saturation, lightness and alpha components.
    ///
    /// See <https://en.wikipedia.org/wiki/HSL_and_HSV>.
    pub fn hsl(h: f32, s: f32, l: f32) -> Self {
        Self::hsla(h, s, l, 1.0)
    }

    /// Convert the color to a hue, saturation, lightness and alpha tuple.
    ///
    /// See <https://en.wikipedia.org/wiki/HSL_and_HSV>.
    pub fn to_hsla(self) -> (f32, f32, f32, f32) {
        let max = self.r.max(self.g).max(self.b);
        let min = self.r.min(self.g).min(self.b);
        let c = max - min;

        let h = if c == 0.0 {
            0.0
        } else if max == self.r {
            60.0 * ((self.g - self.b) / c).rem_euclid(6.0)
        } else if max == self.g {
            60.0 * ((self.b - self.r) / c + 2.0)
        } else {
            60.0 * ((self.r - self.g) / c + 4.0)
        };

        let l = (max + min) / 2.0;

        let s = if c == 0.0 {
            0.0
        } else {
            c / (1.0 - (2.0 * l - 1.0).abs())
        };

        (h, s, l, self.a)
    }

    /// Convert the color to a hue, saturation, lightness tuple.
    ///
    /// See <https://en.wikipedia.org/wiki/HSL_and_HSV>.
    pub fn to_hsl(self) -> (f32, f32, f32) {
        let (h, s, l, _) = self.to_hsla();
        (h, s, l)
    }

    /// Convert a color from oklab to linear sRGB.
    pub fn oklaba(l: f32, a: f32, b: f32, alpha: f32) -> Self {
        let l_ = l + 0.396_337_78 * a + 0.215_803_76 * b;
        let m_ = l - 0.105_561_346 * a - 0.063_854_17 * b;
        let s_ = l - 0.089_484_18 * a - 1.291_485_5 * b;

        let l = l_ * l_ * l_;
        let m = m_ * m_ * m_;
        let s = s_ * s_ * s_;

        Self {
            r: 4.076_741_7 * l - 3.307_711_6 * m + 0.230_969_94 * s,
            g: -1.268_438 * l + 2.609_757_4 * m - 0.341_319_38 * s,
            b: -0.004_196_086_3 * l - 0.703_418_6 * m + 1.707_614_7 * s,
            a: alpha,
        }
    }

    /// Convert a color from oklab to linear sRGB.
    pub fn oklab(l: f32, a: f32, b: f32) -> Self {
        Self::oklaba(l, a, b, 1.0)
    }

    /// Convert a color from linear sRGB to oklab.
    pub fn to_oklaba(self) -> (f32, f32, f32, f32) {
        let l = 0.412_165_1 * self.r + 0.536_275_2 * self.g + 0.051_457_5 * self.b;
        let m = 0.211_859_1 * self.r + 0.680_718_9 * self.g + 0.107_406_6 * self.b;
        let s = 0.088_309_3 * self.r + 0.281_847_4 * self.g + 0.630_261_7 * self.b;

        let l_ = l.cbrt();
        let m_ = m.cbrt();
        let s_ = s.cbrt();

        (
            0.210_454_26 * l_ + 0.793_617_8 * m_ - 0.004_072_047 * s_,
            1.977_998_5 * l_ - 2.428_592_ * m_ + 0.450_593_7 * s_,
            0.025_904_037 * l_ + 0.782_771_8 * m_ - 0.808_675_66 * s_,
            self.a,
        )
    }

    /// Convert a color from linear sRGB to oklab.
    pub fn to_oklab(self) -> (f32, f32, f32) {
        let (l, a, b, _) = self.to_oklaba();
        (l, a, b)
    }

    /// Linearly interpolate between two colors.
    ///
    /// This uses a fractor `t` between `0.0` and `1.0`.
    /// Where `0.0` is `self` and `1.0` is `other`.
    pub fn mix(self, other: Self, t: f32) -> Self {
        other * t + self * (1.0 - t)
    }

    /// Saturates the color by given `amount`.
    pub fn saturate(self, amount: f32) -> Self {
        let (h, s, l, a) = self.to_hsla();
        Self::hsla(h, s + amount, l, a)
    }

    /// Desaturates the color by given `amount`.
    pub fn desaturate(self, amount: f32) -> Self {
        let (h, s, l, a) = self.to_hsla();
        Self::hsla(h, s - amount, l, a)
    }

    /// Brighten the color by the given `amount`.
    pub fn brighten(self, amount: f32) -> Self {
        let (h, s, l, a) = self.to_hsla();
        Self::hsla(h, s, l + amount, a)
    }

    /// Darken the color by the given `amount`.
    pub fn darken(self, amount: f32) -> Self {
        let (h, s, l, a) = self.to_hsla();
        Self::hsla(h, s, l - amount, a)
    }

    /// Fade the color by the given `amount`.
    pub fn fade(self, amount: f32) -> Self {
        Self::rgba(self.r, self.g, self.b, self.a * amount)
    }

    /// Returns true if the color is translucent.
    pub fn is_translucent(self) -> bool {
        self.a < 1.0
    }

    /// Returns true if the color is transparent.
    pub fn is_transparent(self) -> bool {
        self.a == 0.0
    }

    /// Convert the color to sRGB.
    ///
    /// See <https://en.wikipedia.org/wiki/SRGB>.
    pub fn to_srgb(self) -> [f32; 4] {
        [self.r.powf(2.2), self.g.powf(2.2), self.b.powf(2.2), self.a]
    }

    /// Convert the color to linear sRGB.
    pub fn to_rgba8(self) -> [u8; 4] {
        [
            (self.r * 255.0) as u8,
            (self.g * 255.0) as u8,
            (self.b * 255.0) as u8,
            (self.a * 255.0) as u8,
        ]
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
