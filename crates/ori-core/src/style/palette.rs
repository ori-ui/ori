use std::sync::Arc;

use crate::canvas::Color;

use super::style;

/// Get the palette of the style.
pub fn palette() -> Arc<Palette> {
    style()
}

/// A color palette.
#[derive(Clone, Copy, Debug)]
pub struct Palette {
    /// The text color.
    pub text: Color,
    /// The background color.
    pub background: Color,
    /// The primary color.
    pub primary: Color,
    /// The secondary color.
    pub secondary: Color,
    /// The accent color.
    pub accent: Color,
}

impl Default for Palette {
    fn default() -> Self {
        Self::light()
    }
}

impl Palette {
    const MODIFIER: f32 = 0.05;

    /// The default light theme.
    pub fn light() -> Self {
        Self {
            text: Color::hsl(0.0, 0.0, 0.2),
            background: Color::hsl(0.0, 0.0, 0.9),
            primary: Color::hsl(221.0, 1.0, 0.78),
            secondary: Color::hsl(100.0, 0.03, 0.85),
            accent: Color::hsl(150.0, 0.82, 0.47),
        }
    }

    /// The default dark theme.
    pub fn dark() -> Self {
        Self {
            text: Color::hsl(0.0, 0.0, 0.8),
            background: Color::hsl(0.0, 0.0, 0.1),
            primary: Color::hsl(221.0, 0.7, 0.52),
            secondary: Color::hsl(237.0, 0.05, 0.17),
            accent: Color::hsl(334.0, 0.76, 0.47),
        }
    }

    /// Get the text color.
    pub fn text(&self) -> Color {
        self.text
    }

    /// Get the light text color.
    pub fn text_light(&self) -> Color {
        self.text.lighten(Self::MODIFIER)
    }

    /// Get the lighter text color.
    pub fn text_lighter(&self) -> Color {
        self.text.lighten(Self::MODIFIER * 2.0)
    }

    /// Get the dark text color.
    pub fn text_dark(&self) -> Color {
        self.text.darken(Self::MODIFIER)
    }

    /// Get the darker text color.
    pub fn text_darker(&self) -> Color {
        self.text.darken(Self::MODIFIER * 2.0)
    }

    /// Get the background color.
    pub fn background(&self) -> Color {
        self.background
    }

    /// Get the light background color.
    pub fn background_light(&self) -> Color {
        self.background.lighten(Self::MODIFIER)
    }

    /// Get the lighter background color.
    pub fn background_lighter(&self) -> Color {
        self.background.lighten(Self::MODIFIER * 2.0)
    }

    /// Get the dark background color.
    pub fn background_dark(&self) -> Color {
        self.background.darken(Self::MODIFIER)
    }

    /// Get the darker background color.
    pub fn background_darker(&self) -> Color {
        self.background.darken(Self::MODIFIER * 2.0)
    }

    /// Get the primary color.
    pub fn primary(&self) -> Color {
        self.primary
    }

    /// Get the light primary color.
    pub fn primary_light(&self) -> Color {
        self.primary.lighten(Self::MODIFIER)
    }

    /// Get the lighter primary color.
    pub fn primary_lighter(&self) -> Color {
        self.primary.lighten(Self::MODIFIER * 2.0)
    }

    /// Get the dark primary color.
    pub fn primary_dark(&self) -> Color {
        self.primary.darken(Self::MODIFIER)
    }

    /// Get the darker primary color.
    pub fn primary_darker(&self) -> Color {
        self.primary.darken(Self::MODIFIER * 2.0)
    }

    /// Get the secondary color.
    pub fn secondary(&self) -> Color {
        self.secondary
    }

    /// Get the light secondary color.
    pub fn secondary_light(&self) -> Color {
        self.secondary.lighten(Self::MODIFIER)
    }

    /// Get the lighter secondary color.
    pub fn secondary_lighter(&self) -> Color {
        self.secondary.lighten(Self::MODIFIER * 2.0)
    }

    /// Get the dark secondary color.
    pub fn secondary_dark(&self) -> Color {
        self.secondary.darken(Self::MODIFIER)
    }

    /// Get the darker secondary color.
    pub fn secondary_darker(&self) -> Color {
        self.secondary.darken(Self::MODIFIER * 2.0)
    }

    /// Get the accent color.
    pub fn accent(&self) -> Color {
        self.accent
    }

    /// Get the light accent color.
    pub fn accent_light(&self) -> Color {
        self.accent.lighten(Self::MODIFIER)
    }

    /// Get the lighter accent color.
    pub fn accent_lighter(&self) -> Color {
        self.accent.lighten(Self::MODIFIER * 2.0)
    }

    /// Get the dark accent color.
    pub fn accent_dark(&self) -> Color {
        self.accent.darken(Self::MODIFIER)
    }

    /// Get the darker accent color.
    pub fn accent_darker(&self) -> Color {
        self.accent.darken(Self::MODIFIER * 2.0)
    }
}
