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

    /// The lowest emphasis surface color.
    pub surface_lowest: Color,

    /// The low emphasis surface color.
    pub surface_low: Color,

    /// The surface color.
    pub surface: Color,

    /// The high emphasis surface color.
    pub surface_high: Color,

    /// The highest emphasis surface color.
    pub surface_highest: Color,

    /// The outline color.
    pub outline: Color,

    /// The outline variant color.
    pub outline_variant: Color,

    /// The text color.
    pub text: Color,

    /// The subtle text color.
    pub subtext: Color,

    /// The text contrast color.
    pub text_contrast: Color,

    /// The subtle text contrast color.
    pub subtext_contrast: Color,

    /// The primary color.
    pub primary: Color,

    /// The primary variant color.
    pub primary_variant: Color,

    /// The secondary color.
    pub secondary: Color,

    /// The secondary variant color.
    pub secondary_variant: Color,

    /// The accent color.
    pub accent: Color,

    /// The accent variant color.
    pub accent_variant: Color,

    /// The error color.
    pub error: Color,

    /// The error variant color.
    pub error_variant: Color,
}

impl Default for Palette {
    fn default() -> Self {
        Palette::light()
    }
}

impl Palette {
    /// The default light palette.
    pub fn light() -> Self {
        Self {
            background: Color::hex("#eff1f5"),
            surface_lowest: Color::hex("#dce0e8"),
            surface_low: Color::hex("#e6e9ef"),
            surface: Color::hex("#ccd0da"),
            surface_high: Color::hex("#bcc0cc"),
            surface_highest: Color::hex("#acb0be"),
            outline: Color::hex("#6c6f85"),
            outline_variant: Color::hex("#8c8fa1"),
            text: Color::hex("#4c4f69"),
            subtext: Color::hex("#5c5f77"),
            text_contrast: Color::hex("#dce0e8"),
            subtext_contrast: Color::hex("#cfd3db"),
            primary: Color::hex("#04a5e5"),
            primary_variant: Color::hex("#2196f3"),
            secondary: Color::hex("#8839ef"),
            secondary_variant: Color::hex("#9c27b0"),
            accent: Color::hex("#40a02b"),
            accent_variant: Color::hex("#4caf50"),
            error: Color::hex("#d20f39"),
            error_variant: Color::hex("#f44336"),
        }
    }

    /// The default dark palette.
    pub fn dark() -> Self {
        Self {
            background: Color::hex("#1e1e2e"),
            surface_lowest: Color::hex("#11111b"),
            surface_low: Color::hex("#181825"),
            surface: Color::hex("#313244"),
            surface_high: Color::hex("#45475a"),
            surface_highest: Color::hex("#585b70"),
            outline: Color::hex("#6c6f85"),
            outline_variant: Color::hex("#484c63"),
            text: Color::hex("#cdd6f4"),
            subtext: Color::hex("#bac2de"),
            text_contrast: Color::hex("#1e1e2e"),
            subtext_contrast: Color::hex("#2e2e3e"),
            primary: Color::hex("#74c7ec"),
            primary_variant: Color::hex("#2196f3"),
            secondary: Color::hex("#f38ba8"),
            secondary_variant: Color::hex("#e91e63"),
            accent: Color::hex("#99d196"),
            accent_variant: Color::hex("#4caf50"),
            error: Color::hex("#e64553"),
            error_variant: Color::hex("#f44336"),
        }
    }
}
