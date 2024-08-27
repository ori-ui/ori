use std::{
    fmt::{Debug, Display},
    hash::{Hash, Hasher},
    ops::{Add, AddAssign, Deref, Mul},
};

/// Create a new color, with the given `red`, `green` and `blue` components.
pub fn rgb(r: f32, g: f32, b: f32) -> Color {
    Color::rgb(r, g, b)
}

/// Create a new color, with the given `red`, `green`, `blue` and `alpha` components.
pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
    Color::rgba(r, g, b, a)
}

/// Create a new color, with the given `hue`, `saturation` and `lightness` components.
pub fn hsl(h: f32, s: f32, l: f32) -> Color {
    Color::hsl(h, s, l)
}

/// Create a new color, with the given `hue`, `saturation`, `lightness` and `alpha` components.
pub fn hsla(h: f32, s: f32, l: f32, a: f32) -> Color {
    Color::hsla(h, s, l, a)
}

/// Create a new color, with the given `hue`, `saturation` and `value` components.
pub fn hsv(h: f32, s: f32, v: f32) -> Color {
    Color::hsv(h, s, v)
}

/// Create a new color, with the given `hue`, `saturation`, `value` and `alpha` components.
pub fn hsva(h: f32, s: f32, v: f32, a: f32) -> Color {
    Color::hsva(h, s, v, a)
}

/// Create a new color, with the given `lightness`, `a` and `b` components.
pub fn oklab(l: f32, a: f32, b: f32) -> Color {
    Color::oklab(l, a, b)
}

/// Create a new color, with the given `lightness`, `a`, `b` and `alpha` components.
pub fn oklaba(l: f32, a: f32, b: f32, alpha: f32) -> Color {
    Color::oklaba(l, a, b, alpha)
}

/// Create a new color, with the given `lightness`, `chroma` and `hue` components.
pub fn oklch(l: f32, c: f32, h: f32) -> Color {
    Color::oklch(l, c, h)
}

/// Create a new color, with the given `lightness`, `chroma`, `hue` and `alpha` components.
pub fn oklcha(l: f32, c: f32, h: f32, alpha: f32) -> Color {
    Color::oklcha(l, c, h, alpha)
}

/// Create a new color, with the given `hue`, `saturation` and `lightness` components.
pub fn okhsl(h: f32, s: f32, l: f32) -> Color {
    Color::okhsl(h, s, l)
}

/// Create a new color, with the given `hue`, `saturation`, `lightness` and `alpha` components.
pub fn okhsla(h: f32, s: f32, l: f32, a: f32) -> Color {
    Color::okhsla(h, s, l, a)
}

/// Create a new color, with the given `hue`, `saturation` and `value` components.
pub fn okhsv(h: f32, s: f32, v: f32) -> Color {
    Color::okhsv(h, s, v)
}

/// Create a new color, with the given `hue`, `saturation`, `value` and `alpha` components.
pub fn okhsva(h: f32, s: f32, v: f32, a: f32) -> Color {
    Color::okhsva(h, s, v, a)
}

/// Create a new color, with the given hex string.
pub fn hex(hex: &str) -> Color {
    Color::hex(hex)
}

/// A color with red, green, blue and alpha components.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

    /// Create a new color with the given red, green, blue and alpha components.
    pub fn rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self::rgba(
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a as f32 / 255.0,
        )
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

    /// Convert the color to hex.
    pub fn to_hex(self) -> DisplayHex {
        DisplayHex::new(self.r8(), self.g8(), self.b8(), self.a8())
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

    /// Returns a new color with the given hue, saturation, value and alpha components.
    ///
    /// See <https://en.wikipedia.org/wiki/HSL_and_HSV>.
    pub fn hsva(h: f32, s: f32, v: f32, a: f32) -> Self {
        let l = v * (1.0 - s / 2.0);
        let s = if l == 0.0 || l == 1.0 {
            0.0
        } else {
            (v - l) / (l.min(1.0 - l))
        };

        Self::hsla(h, s, l, a)
    }

    /// Returns a new color with the given hue, saturation and value.
    ///
    /// See <https://en.wikipedia.org/wiki/HSL_and_HSV>.
    pub fn hsv(h: f32, s: f32, v: f32) -> Self {
        Self::hsva(h, s, v, 1.0)
    }

    /// Convert the color to a hue, saturation, value and alpha tuple.
    ///
    /// See <https://en.wikipedia.org/wiki/HSL_and_HSV>.
    pub fn to_hsva(self) -> (f32, f32, f32, f32) {
        let (h, s, l, a) = self.to_hsla();

        let v = l + s * l.min(1.0 - l);
        let s = if v == 0.0 { 0.0 } else { 2.0 * (1.0 - l / v) };

        (h, s, v, a)
    }

    /// Convert the color to a hue, saturation, value tuple.
    ///
    /// See <https://en.wikipedia.org/wiki/HSL_and_HSV>.
    pub fn to_hsv(self) -> (f32, f32, f32) {
        let (h, s, v, _) = self.to_hsva();
        (h, s, v)
    }

    fn linear_srgb_to_oklab(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
        let l = 0.412_221_46 * r + 0.536_332_55 * g + 0.051_445_995 * b;
        let m = 0.211_903_5 * r + 0.680_699_5 * g + 0.107_396_96 * b;
        let s = 0.088_302_46 * r + 0.281_718_85 * g + 0.629_978_7 * b;

        let l = l.cbrt();
        let m = m.cbrt();
        let s = s.cbrt();

        (
            0.210_454_26 * l + 0.793_617_8 * m - 0.004_072_047 * s,
            1.977_998_5 * l - 2.428_592_2 * m + 0.450_593_7 * s,
            0.025_904_037 * l + 0.782_771_77 * m - 0.808_675_77 * s,
        )
    }

    fn oklab_to_linear_srgb(l: f32, a: f32, b: f32) -> (f32, f32, f32) {
        let s = l - 0.089_484_18 * a - 1.291_485_5 * b;
        let m = l - 0.105_561_346 * a - 0.063_854_17 * b;
        let l = l + 0.396_337_78 * a + 0.215_803_76 * b;

        let l = l * l * l;
        let m = m * m * m;
        let s = s * s * s;

        (
            4.076_741_7 * l - 3.307_711_6 * m + 0.230_969_94 * s,
            -1.268_438 * l + 2.609_757_4 * m - 0.341_319_38 * s,
            -0.0041960863 * l - 0.703_418_6 * m + 1.707_614_7 * s,
        )
    }

    fn to_linear(x: f32) -> f32 {
        if x < 0.04045 {
            x * 12.92
        } else {
            f32::powf((x + 0.055) / 1.055, 2.4)
        }
    }

    fn from_linear(x: f32) -> f32 {
        if x <= 0.0031308 {
            x / 12.92
        } else {
            1.055 * f32::powf(x, 0.416_666_66) - 0.055
        }
    }

    /// Convert a color from oklab to sRGB.
    ///
    /// See <https://bottosson.github.io/posts/oklab/>.
    pub fn oklaba(l: f32, a: f32, b: f32, alpha: f32) -> Self {
        let (r, g, b) = Self::oklab_to_linear_srgb(l, a, b);

        Self::rgba(
            Self::from_linear(r),
            Self::from_linear(g),
            Self::from_linear(b),
            alpha,
        )
    }

    /// Convert a color from oklab to sRGB.
    ///
    /// See <https://bottosson.github.io/posts/oklab/>.
    pub fn oklab(l: f32, a: f32, b: f32) -> Self {
        Self::oklaba(l, a, b, 1.0)
    }

    /// Convert a color from sRGB to oklab.
    ///
    /// See <https://bottosson.github.io/posts/oklab/>.
    pub fn to_oklaba(self) -> (f32, f32, f32, f32) {
        let (l, a, b) = Self::linear_srgb_to_oklab(
            Self::to_linear(self.r),
            Self::to_linear(self.g),
            Self::to_linear(self.b),
        );

        (l, a, b, self.a)
    }

    /// Convert a color from sRGB to oklab.
    ///
    /// See <https://bottosson.github.io/posts/oklab/>.
    pub fn to_oklab(self) -> (f32, f32, f32) {
        let (l, a, b, _) = self.to_oklaba();
        (l, a, b)
    }

    /// Convert a color from oklch to sRGB.
    ///
    /// See <https://bottosson.github.io/posts/oklab/>.
    pub fn oklcha(l: f32, c: f32, h: f32, alpha: f32) -> Self {
        let (b, a) = h.to_radians().sin_cos();

        Self::oklaba(l, a * c, b * c, alpha)
    }

    /// Convert a color from oklch to sRGB.
    ///
    /// See <https://bottosson.github.io/posts/oklab/>.
    pub fn oklch(l: f32, c: f32, h: f32) -> Self {
        Self::oklcha(l, c, h, 1.0)
    }

    /// Convert a color from sRGB to oklch.
    ///
    /// See <https://bottosson.github.io/posts/oklab/>.
    pub fn to_oklcha(self) -> (f32, f32, f32, f32) {
        let (l, a, b, alpha) = self.to_oklaba();
        let c = (a * a + b * b).sqrt();
        let h = (b.atan2(a).to_degrees() + 360.0).rem_euclid(360.0);

        (l, c, h, alpha)
    }

    /// Convert a color from sRGB to oklch.
    ///
    /// See <https://bottosson.github.io/posts/oklab/>.
    pub fn to_oklch(self) -> (f32, f32, f32) {
        let (l, c, h, _) = self.to_oklcha();
        (l, c, h)
    }

    fn toe(x: f32) -> f32 {
        let k1 = 0.206;
        let k2 = 0.03;
        let k3 = (1.0 + k1) / (1.0 + k2);

        0.5 * (k3 * x - k1 + f32::sqrt((k3 * x - k1) * (k3 * x - k1) + 4.0 * k2 * k3 * x))
    }

    fn toe_inv(x: f32) -> f32 {
        let k1 = 0.206;
        let k2 = 0.03;
        let k3 = (1.0 + k1) / (1.0 + k2);

        (x * x + k1 * x) / (k3 * (x + k2))
    }

    fn compute_max_saturation(a: f32, b: f32) -> f32 {
        let (k0, k1, k2, k3, k4, wl, wm, ws) = match () {
            _ if -1.881_703_3 * a - 0.809_364_9 * b > 1.0 => (
                1.190_862_8,
                1.765_767_3,
                0.596_626_4,
                0.755_152,
                0.567_712_4,
                4.076_741_7,
                -3.307_711_6,
                0.230_969_94,
            ),
            _ if 1.814_441_1 * a - 1.194_452_8 * b > 1.0 => (
                0.739_565_2,
                -0.459_544,
                0.082_854_27,
                0.125_410_7,
                0.145_032_2,
                -1.268_438,
                2.609_757_4,
                -0.341_319_38,
            ),
            _ => (
                1.357_336_5,
                -0.00915799,
                -1.151_302_1,
                -0.50559606,
                0.00692167,
                -0.0041960863,
                -0.703_418_6,
                1.707_614_7,
            ),
        };

        let s_prime = k0 + k1 * a + k2 * b + k3 * a * a + k4 * a * b;

        let kl = 0.396_337_78 * a + 0.215_803_76 * b;
        let km = -0.105_561_346 * a - 0.063_854_17 * b;
        let ks = -0.089_484_18 * a - 1.291_485_5 * b;

        let l_ = 1.0 + s_prime * kl;
        let m_ = 1.0 + s_prime * km;
        let s_ = 1.0 + s_prime * ks;

        let l = l_ * l_ * l_;
        let m = m_ * m_ * m_;
        let s = s_ * s_ * s_;

        let lds = 3.0 * kl * l_ * l_;
        let mds = 3.0 * km * m_ * m_;
        let sds = 3.0 * ks * s_ * s_;

        let lds2 = 6.0 * kl * kl * l_;
        let mds2 = 6.0 * km * km * m_;
        let sds2 = 6.0 * ks * ks * s_;

        let f = wl * l + wm * m + ws * s;
        let f1 = wl * lds + wm * mds + ws * sds;
        let f2 = wl * lds2 + wm * mds2 + ws * sds2;

        s_prime - f * f1 / (f1 * f1 - 0.5 * f * f2)
    }

    fn find_cusp(a: f32, b: f32) -> (f32, f32) {
        let s_cusp = Self::compute_max_saturation(a, b);

        let (r_max, g_max, b_max) = Self::oklab_to_linear_srgb(1.0, s_cusp * a, s_cusp * b);
        let max = f32::max(f32::max(r_max, g_max), b_max);
        let l_cusp = f32::cbrt(1.0 / max);
        let c_cusp = s_cusp * l_cusp;

        (l_cusp, c_cusp)
    }

    fn cusp_to_st(l_cusp: f32, c_cusp: f32) -> (f32, f32) {
        (c_cusp / l_cusp, c_cusp / (1.0 - l_cusp))
    }

    fn find_gamut_intersection(
        a: f32,
        b: f32,
        l1: f32,
        c1: f32,
        l0: f32,
        l_cusp: f32,
        c_cusp: f32,
    ) -> f32 {
        if ((l1 - l0) * c_cusp - (l_cusp - l0) * c1) < 0.0 {
            c_cusp * l0 / (c1 * l_cusp + c_cusp * (l0 - l1))
        } else {
            let t = c_cusp * (l0 - 1.0) / (c1 * (l_cusp - 1.0) + c_cusp * (l0 - l1));

            let dl = l1 - l0;
            let dc = c1;

            let kl = 0.396_337_78 * a + 0.215_803_76 * b;
            let km = -0.105_561_346 * a - 0.063_854_17 * b;
            let ks = -0.089_484_18 * a - 1.291_485_5 * b;

            let l_dt = dl + dc * kl;
            let m_dt = dl + dc * km;
            let s_dt = dl + dc * ks;

            let l = l0 * (1.0 - t) + t * l1;
            let c = t * c1;

            let l_ = l + c * kl;
            let m_ = l + c * km;
            let s_ = l + c * ks;

            let l = l_ * l_ * l_;
            let m = m_ * m_ * m_;
            let s = s_ * s_ * s_;

            let ldt = 3.0 * l_dt * l_ * l_;
            let mdt = 3.0 * m_dt * m_ * m_;
            let sdt = 3.0 * s_dt * s_ * s_;

            let ldt2 = 6.0 * l_dt * l_dt * l_;
            let mdt2 = 6.0 * m_dt * m_dt * m_;
            let sdt2 = 6.0 * s_dt * s_dt * s_;

            let r = 4.076_741_7 * l - 3.307_711_6 * m + 0.230_969_94 * s - 1.0;
            let r1 = 4.076_741_7 * ldt - 3.307_711_6 * mdt + 0.230_969_94 * sdt;
            let r2 = 4.076_741_7 * ldt2 - 3.307_711_6 * mdt2 + 0.230_969_94 * sdt2;

            let u_r = r1 / (r1 * r1 - 0.5 * r * r2);
            let t_r = -r * u_r;

            let g = -1.268_438 * l + 2.609_757_4 * m - 0.341_319_38 * s - 1.0;
            let g1 = -1.268_438 * ldt + 2.609_757_4 * mdt - 0.341_319_38 * sdt;
            let g2 = -1.268_438 * ldt2 + 2.609_757_4 * mdt2 - 0.341_319_38 * sdt2;

            let u_g = g1 / (g1 * g1 - 0.5 * g * g2);
            let t_g = -g * u_g;

            let b = -0.0041960863 * l - 0.703_418_6 * m + 1.707_614_7 * s - 1.0;
            let b1 = -0.0041960863 * ldt - 0.703_418_6 * mdt + 1.707_614_7 * sdt;
            let b2 = -0.0041960863 * ldt2 - 0.703_418_6 * mdt2 + 1.707_614_7 * sdt2;

            let u_b = b1 / (b1 * b1 - 0.5 * b * b2);
            let t_b = -b * u_b;

            let t_r = if u_r >= 0.0 { t_r } else { f32::MAX };
            let t_g = if u_g >= 0.0 { t_g } else { f32::MAX };
            let t_b = if u_b >= 0.0 { t_b } else { f32::MAX };

            t + f32::min(f32::min(t_r, t_g), t_b)
        }
    }

    fn get_st_mid(a: f32, b: f32) -> (f32, f32) {
        // formatting !?

        let s = 0.11516993
            + 1.0
                / (7.447_789_7
                    + 4.159_012_3 * b
                    + a * (-2.195_573_6
                        + 1.751_984 * b
                        + a * (-2.137_049_4 - 10.023_01 * b
                            + a * (-4.248_945_7 + 5.387_708 * b + 4.698_91 * a))));

        let t = 0.11239642
            + 1.0
                / (1.613_203_2 - 0.681_243_8 * b
                    + a * (0.40370612
                        + 0.901_481_2 * b
                        + a * (-0.27087943
                            + 0.612_239_9 * b
                            + a * (0.00299215 - 0.45399568 * b - 0.14661872 * a))));

        (s, t)
    }

    fn get_cs(l: f32, a: f32, b: f32) -> (f32, f32, f32) {
        let (l_cusp, c_cusp) = Self::find_cusp(a, b);
        let (s_max, t_max) = Self::cusp_to_st(l_cusp, c_cusp);
        let c_max = Self::find_gamut_intersection(a, b, l, 1.0, l, l_cusp, c_cusp);

        let k = c_max / f32::min(l * s_max, (1.0 - l) * t_max);

        let (s_mid, t_mid) = Self::get_st_mid(a, b);

        let c_a = l * s_mid;
        let c_b = (1.0 - l) * t_mid;

        let c_a4 = 1.0 / (c_a * c_a * c_a * c_a);
        let c_b4 = 1.0 / (c_b * c_b * c_b * c_b);
        let c_mid = 0.9 * k * f32::sqrt(f32::sqrt(1.0 / (c_a4 + c_b4)));

        let c_a = l * 0.4;
        let c_b = (1.0 - l) * 0.8;

        let c_0 = f32::sqrt(1.0 / (1.0 / (c_a * c_a) + 1.0 / (c_b * c_b)));

        (c_0, c_mid, c_max)
    }

    /// Convert a color from okhsl to sRGB.
    pub fn okhsla(h: f32, s: f32, l: f32, alpha: f32) -> Self {
        if l == 1.0 {
            return Self::WHITE;
        };

        if l == 0.0 {
            return Self::BLACK;
        };

        let (b, a) = h.to_radians().sin_cos();
        let l = Self::toe_inv(l);

        let (c_0, c_mid, c_max) = Self::get_cs(l, a, b);

        let mid = 0.8;
        let mid_inv = 1.25;

        let c = if s < mid {
            let t = mid_inv * s;

            let k_1 = mid * c_0;
            let k_2 = 1.0 - k_1 / c_mid;

            t * k_1 / (1.0 - k_2 * t)
        } else {
            let t = (s - mid) / (1.0 - mid);

            let k_1 = (1.0 - mid) * c_mid * c_mid * mid_inv * mid_inv / c_0;
            let k_2 = 1.0 - k_1 / (c_max - c_mid);

            c_mid + t * k_1 / (1.0 - k_2 * t)
        };

        Self::oklaba(l, c * a, c * b, alpha)
    }

    /// Convert a color from okhsl to sRGB.
    pub fn okhsl(h: f32, s: f32, l: f32) -> Self {
        Self::okhsla(h, s, l, 1.0)
    }

    /// Convert a color from sRGB to okhsl.
    pub fn to_okhsla(self) -> (f32, f32, f32, f32) {
        let (l, a, b) = self.to_oklab();

        if l == 1.0 {
            return (0.0, 0.0, 1.0, self.a);
        };

        if l == 0.0 {
            return (0.0, 0.0, 0.0, self.a);
        };

        if a == 0.0 && b == 0.0 {
            return (0.0, 0.0, l, self.a);
        };

        let c = f32::sqrt(a * a + b * b);
        let a = a / c;
        let b = b / c;

        let h = 180.0 + f32::atan2(-b, -a).to_degrees();

        let (c_0, c_mid, c_max) = Self::get_cs(l, a, b);

        let mid = 0.8;
        let mid_inv = 1.25;

        let s = if c < c_mid {
            let k_1 = mid * c_0;
            let k_2 = 1.0 - k_1 / c_mid;

            c / (k_1 + k_2 * c) * mid
        } else {
            let k_1 = (1.0 - mid) * c_mid * c_mid * mid_inv * mid_inv / c_0;
            let k_2 = 1.0 - k_1 / (c_max - c_mid);

            let t = (c - c_mid) / (k_1 + k_2 * (c - c_mid));
            mid + (1.0 - mid) * t
        };

        let l = Self::toe(l);
        (h, s, l, self.a)
    }

    /// Convert a color from sRGB to okhsl.
    pub fn to_okhsl(self) -> (f32, f32, f32) {
        let (h, s, l, _) = self.to_okhsla();
        (h, s, l)
    }

    /// Convert a color from okhsv to sRGB.
    pub fn okhsva(h: f32, s: f32, v: f32, alpha: f32) -> Self {
        if v == 1.0 {
            return Self::rgba(1.0, 1.0, 1.0, alpha);
        };

        if v == 0.0 {
            return Self::rgba(0.0, 0.0, 0.0, alpha);
        };

        let (b, a) = h.to_radians().sin_cos();

        let (l_cusp, c_cusp) = Self::find_cusp(a, b);
        let (s_max, t_max) = Self::cusp_to_st(l_cusp, c_cusp);
        let s0 = 0.5;
        let k = 1.0 - s0 / s_max;

        let lv = 1.0 - s * s0 / (s0 + t_max - t_max * k * s);
        let cv = s * t_max * s0 / (s0 + t_max - t_max * k * s);

        let l = v * lv;
        let c = v * cv;

        let lvt = Self::toe_inv(lv);
        let cvt = cv * lvt / lv;

        let l_new = Self::toe_inv(l);
        let c = c * l_new / l;
        let l = l_new;

        let (r_max, g_max, b_max) = Self::oklab_to_linear_srgb(lvt, a * cvt, b * cvt);
        let max = f32::max(f32::max(r_max, g_max), f32::max(b_max, 0.0));
        let scale_l = f32::cbrt(1.0 / max);

        let l = l * scale_l;
        let c = c * scale_l;

        Self::oklaba(l, c * a, c * b, alpha)
    }

    /// Convert a color from okhsv to sRGB.
    pub fn okhsv(h: f32, s: f32, v: f32) -> Self {
        Self::okhsva(h, s, v, 1.0)
    }

    /// Convert a color from sRGB to okhsv.
    pub fn to_okhsva(self) -> (f32, f32, f32, f32) {
        let (l, a, b) = self.to_oklab();

        if l == 1.0 {
            return (0.0, 0.0, 1.0, self.a);
        };

        if l == 0.0 {
            return (0.0, 0.0, 0.0, self.a);
        };

        let c = f32::sqrt(a * a + b * b);
        let a = a / c;
        let b = b / c;

        let h = 180.0 + f32::atan2(-b, -a).to_degrees();

        let (l_cusp, c_cusp) = Self::find_cusp(a, b);
        let (s_max, t_max) = Self::cusp_to_st(l_cusp, c_cusp);
        let s0 = 0.5;
        let k = 1.0 - s0 / s_max;

        let t = t_max / (c + l * t_max);
        let lv = t * l;
        let cv = t * c;

        let lvt = Self::toe_inv(lv);
        let cvt = cv * lvt / lv;

        let (r, g, b) = Self::oklab_to_linear_srgb(lvt, a * cvt, b * cvt);
        let max = f32::max(f32::max(r, g), f32::max(b, 0.0));
        let scale_l = f32::cbrt(1.0 / max);

        let v = Self::toe(l / scale_l) / lv;
        let s = (s0 + t_max) * cv / ((t_max * s0) + t_max * k * cv);

        (h, s, v, self.a)
    }

    /// Convert a color from sRGB to okhsv.
    pub fn to_okhsv(self) -> (f32, f32, f32) {
        let (h, s, v, _) = self.to_okhsva();
        (h, s, v)
    }

    /// Get the luminocity.
    pub fn luminocity(self) -> f32 {
        let (_, _, l) = self.to_okhsl();
        l
    }

    /// Linearly interpolate between two colors.
    ///
    /// This uses a fractor `t` between `0.0` and `1.0`.
    /// Where `0.0` is `self` and `1.0` is `other`.
    ///
    /// Note that this is a linear interpolation in the oklab color space.
    /// If rgb interpolation is required use `mix_rgb`.
    pub fn mix(self, other: Self, t: f32) -> Self {
        let (al, aa, ab, aalpha) = self.to_oklaba();
        let (bl, ba, bb, balpha) = other.to_oklaba();

        let l = al * (1.0 - t) + bl * t;
        let a = aa * (1.0 - t) + ba * t;
        let b = ab * (1.0 - t) + bb * t;
        let alpha = aalpha * (1.0 - t) + balpha * t;

        Self::oklaba(l, a, b, alpha)
    }

    /// Linearly interpolate between two colors.
    ///
    /// This uses a fractor `t` between `0.0` and `1.0`.
    /// Where `0.0` is `self` and `1.0` is `other`.
    ///
    /// Note that this is a linear interpolation in the sRGB color space.
    /// If this isn't necessary use `mix`, as it uses the oklab color space,
    /// which is more perceptually uniform.
    pub fn mix_rgb(self, other: Self, t: f32) -> Self {
        self * (1.0 - t) + other * t
    }

    /// Saturates the color by given `amount`.
    pub fn saturate(self, amount: f32) -> Self {
        let (h, s, l, alpha) = self.to_okhsla();
        Self::okhsla(h, s + amount, l, alpha)
    }

    /// Desaturates the color by given `amount`.
    pub fn desaturate(self, amount: f32) -> Self {
        let (h, s, l, alpha) = self.to_okhsla();
        Self::okhsla(h, s - amount, l, alpha)
    }

    /// Brighten the color by the given `amount`.
    pub fn lighten(self, amount: f32) -> Self {
        let (h, s, l, alpha) = self.to_okhsla();
        Self::okhsla(h, s, l + amount, alpha)
    }

    /// Darken the color by the given `amount`.
    pub fn darken(self, amount: f32) -> Self {
        let (h, s, l, alpha) = self.to_okhsla();
        Self::okhsla(h, s, l - amount, alpha)
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

    /// Convert the color to linear sRGB.
    ///
    /// See <https://en.wikipedia.org/wiki/SRGB>.
    pub fn to_srgb(self) -> [f32; 4] {
        [
            Self::to_linear(self.r),
            Self::to_linear(self.g),
            Self::to_linear(self.b),
            self.a,
        ]
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

impl Hash for Color {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.r.to_bits().hash(state);
        self.g.to_bits().hash(state);
        self.b.to_bits().hash(state);
        self.a.to_bits().hash(state);
    }
}

/// A type for displaying a color as a hex string.
///
/// This notably does not allocate.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct DisplayHex {
    bytes: [u8; 9],
}

impl DisplayHex {
    /// Create a new hex color display with the given `red`, `green`, `blue` and `alpha` components.
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        fn to_hex(n: u8) -> [u8; 2] {
            const HEX: &[u8] = b"0123456789abcdef";

            let first = n / 16;
            let second = n % 16;

            [HEX[first as usize], HEX[second as usize]]
        }

        let [r1, r2] = to_hex(r);
        let [g1, g2] = to_hex(g);
        let [b1, b2] = to_hex(b);
        let [a1, a2] = to_hex(a);

        Self {
            bytes: [b'#', r1, r2, g1, g2, b1, b2, a1, a2],
        }
    }

    /// Convert the hex color display to a `&str`.
    pub fn as_str_with_alpha(&self) -> &str {
        std::str::from_utf8(&self.bytes).unwrap()
    }

    /// Convert the hex color display to a `&str`.
    ///
    /// If the alpha component is `ff` it is omitted.
    pub fn as_str(&self) -> &str {
        if &self.bytes[7..] == b"ff" {
            &self.as_str_with_alpha()[..7]
        } else {
            self.as_str_with_alpha()
        }
    }
}

impl Debug for DisplayHex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str_with_alpha())
    }
}

impl Display for DisplayHex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<DisplayHex> for String {
    fn from(value: DisplayHex) -> Self {
        String::from(value.as_str())
    }
}

impl From<&DisplayHex> for String {
    fn from(value: &DisplayHex) -> Self {
        String::from(value.as_str())
    }
}

impl AsRef<str> for DisplayHex {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Deref for DisplayHex {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex() {
        let color = Color::hex("#ff00ff");
        assert_eq!(color, Color::MAGENTA);
    }

    #[test]
    fn test_hex_display() {
        let display = DisplayHex::new(0xa0, 0xb2, 0xcb, 0xd6);
        assert_eq!(display.as_ref(), "#a0b2cbd6");
    }

    #[test]
    fn okhsl_inverse() {
        let color = Color::rgb(0.05, 0.15, 0.20);
        let (h, s, l) = color.to_okhsl();
        let color2 = Color::okhsl(h, s, l);
        assert!(f32::abs(color.r - color2.r) < 0.0001);
        assert!(f32::abs(color.g - color2.g) < 0.0001);
        assert!(f32::abs(color.b - color2.b) < 0.0001);
    }

    #[test]
    fn okhsv_inverse() {
        let color = Color::rgb(0.05, 0.15, 0.20);
        let (h, s, v) = color.to_okhsv();
        let color2 = Color::okhsv(h, s, v);
        assert!(f32::abs(color.r - color2.r) < 0.0001);
        assert!(f32::abs(color.g - color2.g) < 0.0001);
        assert!(f32::abs(color.b - color2.b) < 0.0001);
    }
}
