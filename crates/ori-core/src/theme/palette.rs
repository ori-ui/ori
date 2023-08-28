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
    pub const TEXT: Key<Color> = Key::new("--text");
    pub const BACKGROUND: Key<Color> = Key::new("--background");
    pub const PRIMARY: Key<Color> = Key::new("--primary");
    pub const SECONDARY: Key<Color> = Key::new("--secondary");
    pub const ACCENT: Key<Color> = Key::new("--accent");

    pub const TEXT_DARK: Key<Color> = Key::new("--text-dark");
    pub const TEXT_DARKER: Key<Color> = Key::new("--text-darker");
    pub const TEXT_BRIGHT: Key<Color> = Key::new("--text-bright");
    pub const TEXT_BRIGHTER: Key<Color> = Key::new("--text-brighter");

    pub const BACKGROUND_DARK: Key<Color> = Key::new("--background-dark");
    pub const BACKGROUND_DARKER: Key<Color> = Key::new("--background-darker");
    pub const BACKGROUND_BRIGHT: Key<Color> = Key::new("--background-bright");
    pub const BACKGROUND_BRIGHTER: Key<Color> = Key::new("--background-brighter");

    pub const PRIMARY_DARK: Key<Color> = Key::new("--primary-dark");
    pub const PRIMARY_DARKER: Key<Color> = Key::new("--primary-darker");
    pub const PRIMARY_BRIGHT: Key<Color> = Key::new("--primary-bright");
    pub const PRIMARY_BRIGHTER: Key<Color> = Key::new("--primary-brighter");

    pub const SECONDARY_DARK: Key<Color> = Key::new("--secondary-dark");
    pub const SECONDARY_DARKER: Key<Color> = Key::new("--secondary-darker");
    pub const SECONDARY_BRIGHT: Key<Color> = Key::new("--secondary-bright");
    pub const SECONDARY_BRIGHTER: Key<Color> = Key::new("--secondary-brighter");

    pub const ACCENT_DARK: Key<Color> = Key::new("--accent-dark");
    pub const ACCENT_DARKER: Key<Color> = Key::new("--accent-darker");
    pub const ACCENT_BRIGHT: Key<Color> = Key::new("--accent-bright");
    pub const ACCENT_BRIGHTER: Key<Color> = Key::new("--accent-brighter");
}

impl Palette {
    /// The default light theme.
    pub fn light() -> Self {
        Self {
            text: Color::hsl(0.0, 0.0, 0.2),
            background: Color::hsl(0.0, 0.0, 1.0),
            primary: Color::hsl(221.0, 1.0, 0.78),
            secondary: Color::hsl(100.0, 0.03, 0.88),
            accent: Color::hsl(150.0, 0.82, 0.47),
        }
    }

    /// The default dark theme.
    pub fn dark() -> Self {
        Self {
            text: Color::hsl(0.0, 0.0, 0.8),
            background: Color::hsl(0.0, 0.0, 0.2),
            primary: Color::hsl(221.0, 0.7, 0.62),
            secondary: Color::hsl(0.0, 0.0, 0.27),
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

        self.derived_theme(&mut theme, 0.075);

        theme
    }

    fn derived_theme(&self, theme: &mut Theme, f: f32) {
        theme.set(Self::TEXT_DARK, self.text.darken(f));
        theme.set(Self::TEXT_DARKER, self.text.darken(f * 2.0));
        theme.set(Self::TEXT_BRIGHT, self.text.brighten(f));
        theme.set(Self::TEXT_BRIGHTER, self.text.brighten(f * 2.0));

        theme.set(Self::BACKGROUND_DARK, self.background.darken(f));
        theme.set(Self::BACKGROUND_DARKER, self.background.darken(f * 2.0));
        theme.set(Self::BACKGROUND_BRIGHT, self.background.brighten(f));
        theme.set(Self::BACKGROUND_BRIGHTER, self.background.brighten(f * 2.0));

        theme.set(Self::PRIMARY_DARK, self.primary.darken(f));
        theme.set(Self::PRIMARY_DARKER, self.primary.darken(f * 2.0));
        theme.set(Self::PRIMARY_BRIGHT, self.primary.brighten(f));
        theme.set(Self::PRIMARY_BRIGHTER, self.primary.brighten(f * 2.0));

        theme.set(Self::SECONDARY_DARK, self.secondary.darken(f));
        theme.set(Self::SECONDARY_DARKER, self.secondary.darken(f * 2.0));
        theme.set(Self::SECONDARY_BRIGHT, self.secondary.brighten(f));
        theme.set(Self::SECONDARY_BRIGHTER, self.secondary.brighten(f * 2.0));

        theme.set(Self::ACCENT_DARK, self.accent.darken(f));
        theme.set(Self::ACCENT_DARKER, self.accent.darken(f * 2.0));
        theme.set(Self::ACCENT_BRIGHT, self.accent.brighten(f));
        theme.set(Self::ACCENT_BRIGHTER, self.accent.brighten(f * 2.0));
    }
}

impl From<Palette> for Theme {
    fn from(palette: Palette) -> Self {
        palette.to_theme()
    }
}
