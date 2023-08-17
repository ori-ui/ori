use crate::{
    style, Color, FontFamily, FontStretch, FontStyle, FontWeight, Palette, TextAlign, TextWrap,
    Theme, Transition,
};

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
        const BORDER_RADIUS: [f32; 4] = [8.0; 4];
        /// The border width.
        const BORDER_WIDTH: [f32; 4] = [0.0; 4];
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
        const BORDER_RADIUS: [f32; 4] = [0.0; 4];
        /// The border width.
        const BORDER_WIDTH: [f32; 4] = [0.0; 4];
        /// The border color.
        const BORDER_COLOR: Color = Color::TRANSPARENT;
    }
}

impl Theme {
    pub fn builtin() -> Self {
        let mut theme = Self::new();

        theme.extend(text::default_theme());
        theme.extend(text_input::default_theme());
        theme.extend(button::default_theme());
        theme.extend(container::default_theme());

        theme
    }
}
