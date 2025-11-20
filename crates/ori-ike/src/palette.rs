use ike::Color;

#[derive(Clone, Debug, PartialEq)]
pub struct Palette {
    pub background: Color,
    pub surface:    Color,
    pub outline:    Color,
    pub contrast:   Color,
    pub primary:    Color,
    pub secondary:  Color,
    pub accent:     Color,
    pub danger:     Color,
    pub success:    Color,
    pub warning:    Color,
    pub info:       Color,
}

impl Default for Palette {
    fn default() -> Self {
        Palette {
            background: Color::hex("#1e1e1e"),
            surface:    Color::hex("#242424"),
            outline:    Color::hex("#4d4d4d"),
            contrast:   Color::hex("#f9f9f8"),
            primary:    Color::hex("#55b1f0"),
            secondary:  Color::hex("#8c8bed"),
            accent:     Color::hex("#f4a151"),
            danger:     Color::hex("#f05d51"),
            success:    Color::hex("#9af079"),
            warning:    Color::hex("#f9e35f"),
            info:       Color::hex("#639ff7"),
        }
    }
}

impl Palette {
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
