use crate::canvas::Color;

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
        fn surf(color: Color, is_light: bool, level: i32) -> Color {
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

        Styles::new()
            .with(Theme::BACKGROUND, theme.background)
            .with(Theme::SURFACE_LOWER, surf(theme.surface, is_light, -2))
            .with(Theme::SURFACE_LOW, surf(theme.surface, is_light, -1))
            .with(Theme::SURFACE, theme.surface)
            .with(Theme::SURFACE_HIGH, surf(theme.surface, is_light, 1))
            .with(Theme::SURFACE_HIGHER, surf(theme.surface, is_light, 2))
            .with(Theme::SURFACE_HIGHEST, surf(theme.surface, is_light, 3))
            .with(Theme::OUTLINE, theme.outline)
            .with(Theme::OUTLINE_LOW, low(theme.outline, is_light))
            .with(Theme::CONTRAST, theme.contrast)
            .with(Theme::CONTRAST_LOW, contrast_low(theme.contrast, is_light))
            .with(Theme::PRIMARY, theme.primary)
            .with(Theme::PRIMARY_LOW, low(theme.primary, is_light))
            .with(Theme::SECONDARY, theme.secondary)
            .with(Theme::SECONDARY_LOW, low(theme.secondary, is_light))
            .with(Theme::ACCENT, theme.accent)
            .with(Theme::ACCENT_LOW, low(theme.accent, is_light))
            .with(Theme::DANGER, theme.danger)
            .with(Theme::DANGER_LOW, low(theme.danger, is_light))
            .with(Theme::SUCCESS, theme.success)
            .with(Theme::SUCCESS_LOW, low(theme.success, is_light))
            .with(Theme::WARNING, theme.warning)
            .with(Theme::WARNING_LOW, low(theme.warning, is_light))
            .with(Theme::INFO, theme.info)
            .with(Theme::INFO_LOW, low(theme.info, is_light))
    }
}

impl Default for Theme {
    fn default() -> Self {
        Theme::dark()
    }
}

#[allow(missing_docs)]
impl Theme {
    pub const BACKGROUND: Style<Color> = Style::new("theme.background");
    pub const SURFACE_LOWER: Style<Color> = Style::new("theme.surface-lower");
    pub const SURFACE_LOW: Style<Color> = Style::new("theme.surface-low");
    pub const SURFACE: Style<Color> = Style::new("theme.surface");
    pub const SURFACE_HIGH: Style<Color> = Style::new("theme.surface-high");
    pub const SURFACE_HIGHER: Style<Color> = Style::new("theme.surface-higher");
    pub const SURFACE_HIGHEST: Style<Color> = Style::new("theme.surface-highest");
    pub const OUTLINE: Style<Color> = Style::new("theme.outline");
    pub const OUTLINE_LOW: Style<Color> = Style::new("theme.outline-low");
    pub const CONTRAST: Style<Color> = Style::new("theme.contrast");
    pub const CONTRAST_LOW: Style<Color> = Style::new("theme.contrast-low");
    pub const PRIMARY: Style<Color> = Style::new("theme.primary");
    pub const PRIMARY_LOW: Style<Color> = Style::new("theme.primary-low");
    pub const SECONDARY: Style<Color> = Style::new("theme.secondary");
    pub const SECONDARY_LOW: Style<Color> = Style::new("theme.secondary-low");
    pub const ACCENT: Style<Color> = Style::new("theme.accent");
    pub const ACCENT_LOW: Style<Color> = Style::new("theme.accent-low");
    pub const DANGER: Style<Color> = Style::new("theme.danger");
    pub const DANGER_LOW: Style<Color> = Style::new("theme.danger-low");
    pub const SUCCESS: Style<Color> = Style::new("theme.success");
    pub const SUCCESS_LOW: Style<Color> = Style::new("theme.success-low");
    pub const WARNING: Style<Color> = Style::new("theme.warning");
    pub const WARNING_LOW: Style<Color> = Style::new("theme.warning-low");
    pub const INFO: Style<Color> = Style::new("theme.info");
    pub const INFO_LOW: Style<Color> = Style::new("theme.info-low");
}
