use crate::canvas::Color;

use super::style;

/// Get the palette of the style.
pub fn palette() -> Palette {
    style()
}

/// A color palette, based on catppuccin [https://catppuccin.com/palette].
#[derive(Clone, Copy, Debug)]
pub struct Palette {
    /// The crust color.
    pub crust: Color,

    /// The mantle color.
    pub mantle: Color,

    /// The base color.
    pub base: Color,

    /// The surface color.
    pub surface: Color,

    /// The secondary surface color.
    pub surface_secondary: Color,

    /// The tertiary surface color.
    pub surface_tertiary: Color,

    /// The overlay color.
    pub overlay: Color,

    /// The secondary overlay color.
    pub overlay_secondary: Color,

    /// The tertiary overlay color.
    pub overlay_tertiary: Color,

    /// The subtext color.
    pub subtext: Color,

    /// The secondary subtext color.
    pub subtext_secondary: Color,

    /// The text color.
    pub text: Color,

    /// The primary color.
    pub primary: Color,

    /// The accent color.
    pub accent: Color,

    /// The lavender color.
    pub lavender: Color,

    /// The blue color.
    pub blue: Color,

    /// The saphire color.
    pub sapphire: Color,

    /// The sky color.
    pub sky: Color,

    /// The teal color.
    pub teal: Color,

    /// The green color.
    pub green: Color,

    /// The yellow color.
    pub yellow: Color,

    /// The peach color.
    pub peach: Color,

    /// The maroon color.
    pub maroon: Color,

    /// The red color.
    pub red: Color,

    /// The mauve color.
    pub mauve: Color,

    /// The pink color.
    pub pink: Color,

    /// The flamingo color.
    pub flamingo: Color,

    /// The rosewater color.
    pub rosewater: Color,
}

impl Default for Palette {
    fn default() -> Self {
        Self::catppuccin_latte()
    }
}

impl Palette {
    /// Get the light palette.
    pub fn catppuccin_latte() -> Self {
        Self {
            crust: Color::hex("#dce0e8"),
            mantle: Color::hex("#e6e9ef"),
            base: Color::hex("#eff1f5"),
            surface: Color::hex("#ccd0da"),
            surface_secondary: Color::hex("#bcc0cc"),
            surface_tertiary: Color::hex("#acb0be"),
            overlay: Color::hex("#9ca0b0"),
            overlay_secondary: Color::hex("#8c8fa1"),
            overlay_tertiary: Color::hex("#7c7f93"),
            subtext: Color::hex("#6c6f85"),
            subtext_secondary: Color::hex("#5c5f77"),
            text: Color::hex("#4c4f69"),
            primary: Color::hex("#1e66f5"),
            accent: Color::hex("#df8e1d"),
            lavender: Color::hex("#7287fd"),
            blue: Color::hex("#1e66f5"),
            sapphire: Color::hex("#209fb5"),
            sky: Color::hex("#04a5e5"),
            teal: Color::hex("#179299"),
            green: Color::hex("#40a02b"),
            yellow: Color::hex("#df8e1d"),
            peach: Color::hex("#fe640b"),
            maroon: Color::hex("#e64553"),
            red: Color::hex("#d20f39"),
            mauve: Color::hex("#8839ef"),
            pink: Color::hex("#ea76cb"),
            flamingo: Color::hex("#dd7878"),
            rosewater: Color::hex("#dc8a78"),
        }
    }

    /// Get the medium palette.
    pub fn catppuccin_frappe() -> Self {
        Self {
            crust: Color::hex("#232634"),
            mantle: Color::hex("#292c3c"),
            base: Color::hex("#303446"),
            surface: Color::hex("#414559"),
            surface_secondary: Color::hex("#51576d"),
            surface_tertiary: Color::hex("#626880"),
            overlay: Color::hex("#737994"),
            overlay_secondary: Color::hex("#838ba7"),
            overlay_tertiary: Color::hex("#949cbb"),
            subtext: Color::hex("#a5adce"),
            subtext_secondary: Color::hex("#b5bfe2"),
            text: Color::hex("#c6d0f5"),
            primary: Color::hex("#8caaee"),
            accent: Color::hex("#e5c890"),
            lavender: Color::hex("#babbf1"),
            blue: Color::hex("#8caaee"),
            sapphire: Color::hex("#85c1dc"),
            sky: Color::hex("#99d1db"),
            teal: Color::hex("#81c8be"),
            green: Color::hex("#a6d189"),
            yellow: Color::hex("#e5c890"),
            peach: Color::hex("#ef9f76"),
            maroon: Color::hex("#ea999c"),
            red: Color::hex("#e78284"),
            mauve: Color::hex("#ca9ee6"),
            pink: Color::hex("#f4b8e4"),
            flamingo: Color::hex("#eebebe"),
            rosewater: Color::hex("#f2d5cf"),
        }
    }

    /// Get the dark palette.
    pub fn catppuccin_macchiato() -> Self {
        Self {
            crust: Color::hex("#181926"),
            mantle: Color::hex("#1e2030"),
            base: Color::hex("#24273a"),
            surface: Color::hex("#363a4f"),
            surface_secondary: Color::hex("#494d64"),
            surface_tertiary: Color::hex("#5b6078"),
            overlay: Color::hex("#6e738d"),
            overlay_secondary: Color::hex("#8087a2"),
            overlay_tertiary: Color::hex("#939ab7"),
            subtext: Color::hex("#a5adcb"),
            subtext_secondary: Color::hex("#b8c0e0"),
            text: Color::hex("#cad3f5"),
            primary: Color::hex("#8aadf4"),
            accent: Color::hex("#eed49f"),
            lavender: Color::hex("#b7bdf8"),
            blue: Color::hex("#8aadf4"),
            sapphire: Color::hex("#7dc4e4"),
            sky: Color::hex("#91d7e3"),
            teal: Color::hex("#8bd5ca"),
            green: Color::hex("#a6da95"),
            yellow: Color::hex("#eed49f"),
            peach: Color::hex("#f5a97f"),
            maroon: Color::hex("#ee99a0"),
            red: Color::hex("#ed8796"),
            mauve: Color::hex("#c6a0f6"),
            pink: Color::hex("#f5bde6"),
            flamingo: Color::hex("#f0c6c6"),
            rosewater: Color::hex("#f4dbd6"),
        }
    }

    /// Get the darkest palette.
    pub fn catppuccin_mocha() -> Self {
        Self {
            crust: Color::hex("#11111b"),
            mantle: Color::hex("#181825"),
            base: Color::hex("#1e1e2e"),
            surface: Color::hex("#313244"),
            surface_secondary: Color::hex("#45475a"),
            surface_tertiary: Color::hex("#585b70"),
            overlay: Color::hex("#6c7086"),
            overlay_secondary: Color::hex("#7f849c"),
            overlay_tertiary: Color::hex("#9399b2"),
            subtext: Color::hex("#a6adc8"),
            subtext_secondary: Color::hex("#bac2de"),
            text: Color::hex("#cdd6f4"),
            primary: Color::hex("#89b4fa"),
            accent: Color::hex("#f9e2af"),
            lavender: Color::hex("#b4befe"),
            blue: Color::hex("#89b4fa"),
            sapphire: Color::hex("#74c7ec"),
            sky: Color::hex("#89dceb"),
            teal: Color::hex("#94e2d5"),
            green: Color::hex("#a6e3a1"),
            yellow: Color::hex("#f9e2af"),
            peach: Color::hex("#fab387"),
            maroon: Color::hex("#eba0ac"),
            red: Color::hex("#f38ba8"),
            mauve: Color::hex("#cba6f7"),
            pink: Color::hex("#f5c2e7"),
            flamingo: Color::hex("#f2cdcd"),
            rosewater: Color::hex("#f5e0dc"),
        }
    }
}
