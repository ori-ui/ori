use crate::{canvas::Color, style};

use super::Styles;

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
}

impl From<Theme> for Styles {
    fn from(theme: Theme) -> Self {
        fn emphasize(color: Color, is_light: bool, level: i32) -> Color {
            let level = level as f32;

            if is_light {
                color.darken(level * 0.025).saturate(level * 0.015)
            } else {
                color.lighten(level * 0.04).saturate(level * 0.02)
            }
        }

        fn low(color: Color, is_light: bool) -> Color {
            if is_light {
                color.lighten(0.2).desaturate(0.1)
            } else {
                color.darken(0.2).desaturate(0.1)
            }
        }

        fn contrast_low(color: Color, is_light: bool) -> Color {
            if is_light {
                color.lighten(0.2).desaturate(0.1)
            } else {
                color.darken(0.2).desaturate(0.1)
            }
        }

        let is_light = theme.background.luminocity() > 0.5;

        style! {
            "palette" {
                "background": theme.background,
                "surface_lower": emphasize(theme.surface, is_light, -2),
                "surface_low": emphasize(theme.surface, is_light, -1),
                "surface": theme.surface,
                "surface_high": emphasize(theme.surface, is_light, 1),
                "surface_higher": emphasize(theme.surface, is_light, 2),
                "surface_highest": emphasize(theme.surface, is_light, 3),
                "outline": theme.outline,
                "outline_low": low(theme.outline, is_light),
                "contrast": theme.contrast,
                "contrast_low": contrast_low(theme.contrast, is_light),
                "primary": theme.primary,
                "primary_low": low(theme.primary, is_light),
                "secondary": theme.secondary,
                "secondary_low": low(theme.secondary, is_light),
                "accent": theme.accent,
                "accent_low": low(theme.accent, is_light),
                "danger": theme.danger,
                "danger_low": low(theme.danger, is_light),
                "success": theme.success,
                "success_low": low(theme.success, is_light),
                "warning": theme.warning,
                "warning_low": low(theme.warning, is_light),
                "info": theme.info,
                "info_low": low(theme.info, is_light),
            },
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Theme::dark()
    }
}
