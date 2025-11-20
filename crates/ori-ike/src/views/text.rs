use std::borrow::Cow;

use ike::{Color, FontStretch, FontStyle, FontWeight, TextAlign, TextWrap};

#[derive(Clone, Debug, PartialEq)]
pub struct TextTheme {
    pub font_size:    f32,
    pub font_family:  Cow<'static, str>,
    pub font_weight:  FontWeight,
    pub font_stretch: FontStretch,
    pub font_style:   FontStyle,
    pub line_height:  f32,
    pub align:        TextAlign,
    pub wrap:         TextWrap,
    pub color:        Option<Color>,
}

impl Default for TextTheme {
    fn default() -> Self {
        Self {
            font_size:    16.0,
            font_family:  Cow::Borrowed("Ubuntu Light"),
            font_weight:  FontWeight::NORMAL,
            font_stretch: FontStretch::Normal,
            font_style:   FontStyle::Normal,
            line_height:  1.0,
            align:        TextAlign::Start,
            wrap:         TextWrap::Word,
            color:        None,
        }
    }
}
