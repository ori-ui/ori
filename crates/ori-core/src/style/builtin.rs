//! Builtin styles.

use crate::{
    BorderRadius, BorderWidth, Color, FontFamily, FontStretch, FontStyle, FontWeight, Palette,
    TextAlign, TextWrap, Theme, Transition,
};

macro_rules! style {
    (
        $(#[$module_attr:meta])*
        $module_vis:vis $module:ident {
            $(
                $(#[$attr:meta])*
                const $name:ident : $ty:ty = $expr:expr;
            )*
        }
    ) => {
        $(#[$module_attr])*
        $module_vis mod $module {
            use super::*;

            $(
                $(#[$attr])*
                pub const $name: $crate::Key<$ty> = $crate::Key::new(
                    ::std::concat!(::std::stringify!($module), ".", ::std::stringify!($name))
                );
            )*

            /// Get the default theme for this module.
            pub fn default_theme() -> $crate::Theme {
                $crate::Theme::new()
                    $(.with($name, $expr))*
            }
        }
    };
}

style! {
    /// Styles for [`Text`](crate::views::Text)s.
    pub text {
        /// The font size.
        const FONT_SIZE: f32 = 16.0;
        /// The font family.
        const FONT_FAMILY: FontFamily = FontFamily::SansSerif;
        /// The font weight.
        const FONT_WEIGHT: FontWeight = FontWeight::NORMAL;
        /// The font stretch.
        const FONT_STRETCH: FontStretch = FontStretch::Normal;
        /// The font style.
        const FONT_STYLE: FontStyle = FontStyle::Normal;
        /// The color.
        const COLOR: Color = Palette::TEXT;
        /// The vertical alignment.
        const V_ALIGN: TextAlign = TextAlign::Top;
        /// The horizontal alignment.
        const H_ALIGN: TextAlign = TextAlign::Left;
        /// The line height.
        const LINE_HEIGHT: f32 = 1.0;
        /// The text wrap.
        const WRAP: TextWrap = TextWrap::Word;
    }
}

style! {
    /// Styles for [`TextInput`](crate::views::Text)s.
    pub text_input {
        /// The font size.
        const FONT_SIZE: f32 = 16.0;
        /// The font family.
        const FONT_FAMILY: FontFamily = FontFamily::SansSerif;
        /// The font weight.
        const FONT_WEIGHT: FontWeight = FontWeight::NORMAL;
        /// The font stretch.
        const FONT_STRETCH: FontStretch = FontStretch::Normal;
        /// The font style.
        const FONT_STYLE: FontStyle = FontStyle::Normal;
        /// The color.
        const COLOR: Color = Palette::TEXT;
        /// The vertical alignment.
        const V_ALIGN: TextAlign = TextAlign::Top;
        /// The horizontal alignment.
        const H_ALIGN: TextAlign = TextAlign::Left;
        /// The line height.
        const LINE_HEIGHT: f32 = 1.0;
        /// The text wrap.
        const WRAP: TextWrap = TextWrap::Word;
    }
}

style! {
    /// Styles for [`Button`](crate::views::Button)s.
    pub button {
        /// The transition when the button is hovered.
        const TRANSITION: Transition = Transition::ease(0.1);
        /// The color.
        const COLOR: Color = Palette::PRIMARY;
        /// The border radius.
        const BORDER_RADIUS: BorderRadius = BorderRadius::all(8.0);
        /// The border width.
        const BORDER_WIDTH: BorderWidth = BorderWidth::all(0.0);
        /// The border color.
        const BORDER_COLOR: Color = Color::TRANSPARENT;
    }
}

style! {
    /// Styles for [`Container`](crate::views::Container)s.
    pub container {
        /// The background color.
        const BACKGROUND: Color = Color::TRANSPARENT;
        /// The border radius.
        const BORDER_RADIUS: BorderRadius = BorderRadius::all(0.0);
        /// The border width.
        const BORDER_WIDTH: BorderWidth = BorderWidth::all(0.0);
        /// The border color.
        const BORDER_COLOR: Color = Color::TRANSPARENT;
    }
}

style! {
    /// Styles for [`Checkbox`](crate::views::Checkbox)s.
    pub checkbox {
        /// The transition when the checkbox is hovered.
        const TRANSITION: Transition = Transition::ease(0.1);
        /// The size of the checkbox.
        const SIZE: f32 = 24.0;
        /// The color of the checkmark.
        const COLOR: Color = Palette::ACCENT;
        /// The stroke width of the checkmark.
        const STROKE: f32 = 1.0;
        /// The background color.
        const BACKGROUND: Color = Color::TRANSPARENT;
        /// The border radius.
        const BORDER_RADIUS: BorderRadius = BorderRadius::all(6.0);
        /// The border width.
        const BORDER_WIDTH: BorderWidth = BorderWidth::all(1.0);
        /// The border color.
        const BORDER_COLOR: Color = Palette::TEXT_BRIGHTER;
    }
}

impl Theme {
    /// Get the builtin theme.
    pub fn builtin() -> Self {
        let mut theme = Self::new();

        theme.extend(text::default_theme());
        theme.extend(text_input::default_theme());
        theme.extend(button::default_theme());
        theme.extend(container::default_theme());
        theme.extend(checkbox::default_theme());

        theme
    }
}
