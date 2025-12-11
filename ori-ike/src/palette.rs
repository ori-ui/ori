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
        Self::dark()
    }
}

impl Palette {
    pub const fn dark() -> Self {
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

    pub const fn paper() -> Self {
        Self {
            background: Color::hex("#fdf6e3"),
            surface:    Color::hex("#fdf6e3"),
            outline:    Color::hex("#323d43"),
            contrast:   Color::hex("#323d43"),
            primary:    Color::hex("#e68183"),
            secondary:  Color::hex("#7fbbb3"),
            accent:     Color::hex("#d699b6"),
            danger:     Color::hex("#e68183"),
            success:    Color::hex("#a7c080"),
            warning:    Color::hex("#dbbc7f"),
            info:       Color::hex("#3a94c5"),
        }
    }

    fn level(color: Color, is_light: bool, level: i8) -> Color {
        if level == 0 {
            return color;
        }

        let level = level as f32;
        let (h, s, l, a) = color.to_okhsla();

        if is_light {
            Color::okhsla(
                h,
                s - level * 0.025,
                l + level * 0.015,
                a,
            )
        } else {
            Color::okhsla(h, s - level * 0.04, l + level * 0.02, a)
        }
    }

    fn level_low(color: Color, is_light: bool, level: i8) -> Color {
        let level = level as f32;
        let (h, s, l, a) = color.to_okhsla();

        if is_light {
            Color::okhsla(
                h,
                s - level * 0.025 - 0.1,
                l + level * 0.015 + 0.2,
                a,
            )
        } else {
            Color::okhsla(
                h,
                s - level * 0.04 - 0.1,
                l + level * 0.02 - 0.2,
                a,
            )
        }
    }

    fn is_light(&self) -> bool {
        self.background.luminocity() > 0.5
    }
}

macro_rules! palette_levels {
    ($($field:ident $(/ $field_low:ident)?),* $(,)?) => {
        impl Palette {$(
            /// Get the `
            #[doc = stringify!($field)]
            /// ` color at a specific `level`.
            pub fn $field(&self, level: i8) -> Color {
                Self::level(self.$field, self.is_light(), level)
            }

            $(
                /// Get the low contrast `
                #[doc = stringify!($field)]
                /// ` color at a specific `level`.
                pub fn $field_low(&self, level: i8) -> Color {
                    Self::level_low(self.$field, self.is_light(), level)
                }
            )?
        )*}
    };
}

palette_levels! {
    background,
    surface,
    outline/outline_low,
    contrast/contrast_low,
    primary/primary_low,
    secondary/secondary_low,
    accent/accent_low,
    danger/danger_low,
    success/success_low,
    warning/warning_low,
    info/info_low,
}
