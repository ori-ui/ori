use crate::canvas::Color;

use super::{Style, StyleBuilder};

/// A theme.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Theme {
    /// The background color.
    pub background: Color,

    /// The surface color.
    pub surface: Color,

    /// The outline color.
    pub outline: Color,

    /// The contrast color.
    pub contrast: Color,

    /// The primary color.
    pub primary: Color,

    /// The secondary color.
    pub secondary: Color,

    /// The accent color.
    pub accent: Color,

    /// The danger color.
    pub danger: Color,

    /// The success color.
    pub success: Color,

    /// The warning color.
    pub warning: Color,

    /// The info color.
    pub info: Color,
}

impl Theme {
    /// Create the default light theme.
    pub fn light() -> Self {
        Self {
            background: Color::hex("#ffffff"),
            surface: Color::hex("#eeeff0"),
            outline: Color::hex("#d7d8dd"),
            contrast: Color::hex("#060607"),
            primary: Color::hex("#1c71d8"),
            secondary: Color::hex("#f6d32d"),
            accent: Color::hex("#0077c2"),
            danger: Color::hex("#e01b24"),
            success: Color::hex("#33d17a"),
            warning: Color::hex("#f6d32d"),
            info: Color::hex("#0077c2"),
        }
    }

    /// Create the default dark theme.
    pub fn dark() -> Self {
        Self {
            background: Color::hex("#1e1e1e"),
            surface: Color::hex("#242424"),
            outline: Color::hex("#4d4d4d"),
            contrast: Color::hex("#f9f9f8"),
            primary: Color::hex("#55b1f0"),
            secondary: Color::hex("#8c8bed"),
            accent: Color::hex("#f4a151"),
            danger: Color::hex("#f05d51"),
            success: Color::hex("#9af079"),
            warning: Color::hex("#f9e35f"),
            info: Color::hex("#639ff7"),
        }
    }

    /// Get the surface color with a specific level.
    ///
    /// Common levels are:
    /// - `-2`: very low
    /// - `-1`: low
    /// - `0`: normal
    /// - `1`: high
    /// - `2`: very high
    pub fn surface(&self, level: i8) -> Color {
        let level = level as f32;

        if self.is_light() {
            self.surface.darken(level * 0.025).saturate(level * 0.015)
        } else {
            self.surface.lighten(level * 0.04).saturate(level * 0.02)
        }
    }

    /// Get the low contrast outline color.
    pub fn outline_low(&self) -> Color {
        Self::low(self.outline, self.is_light())
    }

    /// Get the low contrast contrast color.
    pub fn contrast_low(&self) -> Color {
        Self::low(self.contrast, self.is_light())
    }

    /// Get the low contrast primary color.
    pub fn primary_low(&self) -> Color {
        Self::low(self.primary, self.is_light())
    }

    /// Get the low contrast secondary color.
    pub fn secondary_low(&self) -> Color {
        Self::low(self.secondary, self.is_light())
    }

    /// Get the low contrast accent color.
    pub fn accent_low(&self) -> Color {
        Self::low(self.accent, self.is_light())
    }

    /// Get the low contrast danger color.
    pub fn danger_low(&self) -> Color {
        Self::low(self.danger, self.is_light())
    }

    /// Get the low contrast success color.
    pub fn success_low(&self) -> Color {
        Self::low(self.success, self.is_light())
    }

    /// Get the low contrast warning color.
    pub fn warning_low(&self) -> Color {
        Self::low(self.warning, self.is_light())
    }

    /// Get the low contrast info color.
    pub fn info_low(&self) -> Color {
        Self::low(self.info, self.is_light())
    }

    fn low(color: Color, is_light: bool) -> Color {
        if is_light {
            color.lighten(0.2).desaturate(0.1)
        } else {
            color.darken(0.2).desaturate(0.1)
        }
    }

    fn is_light(&self) -> bool {
        self.background.luminocity() > 0.5
    }
}

impl Style for Theme {
    fn builder() -> StyleBuilder<Self> {
        StyleBuilder::new(Theme::dark)
    }
}
