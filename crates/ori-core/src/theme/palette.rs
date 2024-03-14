use crate::canvas::Color;

use super::{Key, Theme};

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

#[allow(missing_docs)]
impl Palette {
    pub const TEXT: Key<Color> = Key::new("palette.text");
    pub const BACKGROUND: Key<Color> = Key::new("palette.background");
    pub const PRIMARY: Key<Color> = Key::new("palette.primary");
    pub const SECONDARY: Key<Color> = Key::new("palette.secondary");
    pub const ACCENT: Key<Color> = Key::new("palette.accent");

    pub const TEXT_DARK: Key<Color> = Key::new("palette.text-dark");
    pub const TEXT_DARKER: Key<Color> = Key::new("palette.text-darker");
    pub const TEXT_LIGHT: Key<Color> = Key::new("palette.text-light");
    pub const TEXT_LIGHTER: Key<Color> = Key::new("palette.text-lighter");

    pub const BACKGROUND_DARK: Key<Color> = Key::new("palette.background-dark");
    pub const BACKGROUND_DARKER: Key<Color> = Key::new("palette.background-darker");
    pub const BACKGROUND_LIGHT: Key<Color> = Key::new("palette.background-light");
    pub const BACKGROUND_LIGHTER: Key<Color> = Key::new("palette.background-lighter");

    pub const PRIMARY_DARK: Key<Color> = Key::new("palette.primary-dark");
    pub const PRIMARY_DARKER: Key<Color> = Key::new("palette.primary-darker");
    pub const PRIMARY_LIGHT: Key<Color> = Key::new("palette.primary-light");
    pub const PRIMARY_LIGHTER: Key<Color> = Key::new("palette.primary-lighter");

    pub const SECONDARY_DARK: Key<Color> = Key::new("palette.secondary-dark");
    pub const SECONDARY_DARKER: Key<Color> = Key::new("palette.secondary-darker");
    pub const SECONDARY_LIGHT: Key<Color> = Key::new("palette.secondary-light");
    pub const SECONDARY_LIGHTER: Key<Color> = Key::new("palette.secondary-lighter");

    pub const ACCENT_DARK: Key<Color> = Key::new("palette.accent-dark");
    pub const ACCENT_DARKER: Key<Color> = Key::new("palette.accent-darker");
    pub const ACCENT_LIGHT: Key<Color> = Key::new("palette.accent-light");
    pub const ACCENT_LIGHTER: Key<Color> = Key::new("palette.accent-lighter");
}

impl Palette {
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

    /// Convert this palette to a theme.
    pub fn to_theme(&self) -> Theme {
        let mut theme = Theme::new();

        theme.set(Self::TEXT, self.text);
        theme.set(Self::BACKGROUND, self.background);
        theme.set(Self::PRIMARY, self.primary);
        theme.set(Self::SECONDARY, self.secondary);
        theme.set(Self::ACCENT, self.accent);

        self.derived_theme(&mut theme, 0.05);

        theme
    }

    fn derived_theme(&self, theme: &mut Theme, f: f32) {
        theme.set(Self::TEXT_DARK, self.text.darken(f));
        theme.set(Self::TEXT_DARKER, self.text.darken(f * 2.0));
        theme.set(Self::TEXT_LIGHT, self.text.lighten(f));
        theme.set(Self::TEXT_LIGHTER, self.text.lighten(f * 2.0));

        theme.set(Self::BACKGROUND_DARK, self.background.darken(f));
        theme.set(Self::BACKGROUND_DARKER, self.background.darken(f * 2.0));
        theme.set(Self::BACKGROUND_LIGHT, self.background.lighten(f));
        theme.set(Self::BACKGROUND_LIGHTER, self.background.lighten(f * 2.0));

        theme.set(Self::PRIMARY_DARK, self.primary.darken(f));
        theme.set(Self::PRIMARY_DARKER, self.primary.darken(f * 2.0));
        theme.set(Self::PRIMARY_LIGHT, self.primary.lighten(f));
        theme.set(Self::PRIMARY_LIGHTER, self.primary.lighten(f * 2.0));

        theme.set(Self::SECONDARY_DARK, self.secondary.darken(f));
        theme.set(Self::SECONDARY_DARKER, self.secondary.darken(f * 2.0));
        theme.set(Self::SECONDARY_LIGHT, self.secondary.lighten(f));
        theme.set(Self::SECONDARY_LIGHTER, self.secondary.lighten(f * 2.0));

        theme.set(Self::ACCENT_DARK, self.accent.darken(f));
        theme.set(Self::ACCENT_DARKER, self.accent.darken(f * 2.0));
        theme.set(Self::ACCENT_LIGHT, self.accent.lighten(f));
        theme.set(Self::ACCENT_LIGHTER, self.accent.lighten(f * 2.0));
    }
}

impl From<Palette> for Theme {
    fn from(palette: Palette) -> Self {
        palette.to_theme()
    }
}
