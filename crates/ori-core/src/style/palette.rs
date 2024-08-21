use crate::canvas::Color;

use super::style;

/// Get the palette of the style.
#[track_caller]
pub fn palette() -> Palette {
    style()
}

/// A color palette.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Palette {
    /// The background color.
    pub background: Color,

    /// The lower emphasis surface color.
    pub surface_lower: Color,

    /// The low emphasis surface color.
    pub surface_low: Color,

    /// The surface color.
    pub surface: Color,

    /// The high emphasis surface color.
    pub surface_high: Color,

    /// The higher emphasis surface color.
    pub surface_higher: Color,

    /// The highest emphasis surface color.
    pub surface_highest: Color,

    /// The contrast color.
    ///
    /// Used for text and icons.
    pub contrast: Color,

    /// The primary color.
    pub primary: Color,

    /// The low emphasis primary color.
    pub primary_low: Color,

    /// The secondary color.
    pub secondary: Color,

    /// The low emphasis secondary color.
    pub secondary_low: Color,

    /// The accent color.
    pub accent: Color,

    /// The low emphasis accent color.
    pub accent_low: Color,

    /// The danger color.
    ///
    /// Used for errors and destructive actions.
    pub danger: Color,

    /// The low emphasis danger color.
    pub danger_low: Color,

    /// The success color.
    pub success: Color,

    /// The low emphasis success color.
    pub success_low: Color,

    /// The warning color.
    pub warning: Color,

    /// The low emphasis warning color.
    pub warning_low: Color,

    /// The info color.
    pub info: Color,

    /// The low emphasis info color.
    pub info_low: Color,
}

impl Default for Palette {
    fn default() -> Self {
        Palette::dark()
    }
}

impl Palette {
    /// Create a new palette, derived from the given colors.
    #[allow(clippy::too_many_arguments)]
    pub fn derived(
        background: Color,
        surface: Color,
        contrast: Color,
        primary: Color,
        secondary: Color,
        accent: Color,
        danger: Color,
        success: Color,
        warning: Color,
        info: Color,
    ) -> Self {
        fn emphasize(color: Color, is_light: bool, amount: f32) -> Color {
            if is_light {
                color.darken(amount).saturate(amount * 0.3)
            } else {
                color.lighten(amount).saturate(amount * 0.3)
            }
        }

        fn low(color: Color, is_light: bool) -> Color {
            if is_light {
                color.lighten(0.3).desaturate(0.2)
            } else {
                color.darken(0.3).desaturate(0.2)
            }
        }

        let is_light = background.luminocity() > 0.5;

        Self {
            background,
            surface_lower: emphasize(surface, is_light, -0.1),
            surface_low: emphasize(surface, is_light, -0.05),
            surface,
            surface_high: emphasize(surface, is_light, 0.05),
            surface_higher: emphasize(surface, is_light, 0.1),
            surface_highest: emphasize(surface, is_light, 0.15),
            contrast,
            primary,
            primary_low: low(primary, is_light),
            secondary,
            secondary_low: low(secondary, is_light),
            accent,
            accent_low: low(accent, is_light),
            danger,
            danger_low: low(danger, is_light),
            success,
            success_low: low(success, is_light),
            warning,
            warning_low: low(warning, is_light),
            info,
            info_low: low(info, is_light),
        }
    }

    /// The default light palette.
    pub fn light() -> Self {
        Self::derived(
            Color::hex("#f5f5f5"),
            Color::hex("#e4e4e4"),
            Color::hex("#212121"),
            Color::hex("#1c71d8"),
            Color::hex("#f6d32d"),
            Color::hex("#0077c2"),
            Color::hex("#e01b24"),
            Color::hex("#33d17a"),
            Color::hex("#f6d32d"),
            Color::hex("#0077c2"),
        )
    }

    /// The default dark palette.
    pub fn dark() -> Self {
        Self::derived(
            Color::hex("#1e1e1eff"),
            Color::hex("#242424ff"),
            Color::hex("#dedddaff"),
            Color::hex("#55b1f0ff"),
            Color::hex("#d3a4f9ff"),
            Color::hex("#f9afadff"),
            Color::hex("#f05d51ff"),
            Color::hex("#9af079ff"),
            Color::hex("#f9e35fff"),
            Color::hex("#639ff7ff"),
        )
    }
}
