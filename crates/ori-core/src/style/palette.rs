use crate::{canvas::Color, style};

use super::{Style, Styles};

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

#[allow(missing_docs)]
impl Theme {
    pub const BACKGROUND: Style<Color> = Style::new("palette.background");
    pub const SURFACE_LOWER: Style<Color> = Style::new("palette.surface_lower");
    pub const SURFACE_LOW: Style<Color> = Style::new("palette.surface_low");
    pub const SURFACE: Style<Color> = Style::new("palette.surface");
    pub const SURFACE_HIGH: Style<Color> = Style::new("palette.surface_high");
    pub const SURFACE_HIGHER: Style<Color> = Style::new("palette.surface_higher");
    pub const SURFACE_HIGHEST: Style<Color> = Style::new("palette.surface_highest");
    pub const OUTLINE: Style<Color> = Style::new("palette.outline");
    pub const OUTLINE_LOW: Style<Color> = Style::new("palette.outline_low");
    pub const CONTRAST: Style<Color> = Style::new("palette.contrast");
    pub const CONTRAST_LOW: Style<Color> = Style::new("palette.contrast_low");
    pub const PRIMARY: Style<Color> = Style::new("palette.primary");
    pub const PRIMARY_LOW: Style<Color> = Style::new("palette.primary_low");
    pub const SECONDARY: Style<Color> = Style::new("palette.secondary");
    pub const SECONDARY_LOW: Style<Color> = Style::new("palette.secondary_low");
    pub const ACCENT: Style<Color> = Style::new("palette.accent");
    pub const ACCENT_LOW: Style<Color> = Style::new("palette.accent_low");
    pub const DANGER: Style<Color> = Style::new("palette.danger");
    pub const DANGER_LOW: Style<Color> = Style::new("palette.danger_low");
    pub const SUCCESS: Style<Color> = Style::new("palette.success");
    pub const SUCCESS_LOW: Style<Color> = Style::new("palette.success_low");
    pub const WARNING: Style<Color> = Style::new("palette.warning");
    pub const WARNING_LOW: Style<Color> = Style::new("palette.warning_low");
    pub const INFO: Style<Color> = Style::new("palette.info");
    pub const INFO_LOW: Style<Color> = Style::new("palette.info_low");
}
