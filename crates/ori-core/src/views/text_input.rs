use ori_macro::{example, Build, Styled};

use crate::{
    canvas::Color,
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::{Capitalize, Event},
    layout::{Size, Space},
    style::{Styled, Theme},
    text::{FontFamily, FontStretch, FontStyle, FontWeight, TextAlign, TextWrap},
    view::View,
};

/// Create a new [`TextInput`].
pub fn text_input<T>() -> TextInput<T> {
    TextInput::new()
}

/// A text input.
///
/// Can be styled using the [`TextInputStyle`].
#[example(name = "text_input", width = 400, height = 300)]
#[derive(Styled, Build)]
pub struct TextInput<T> {
    /// The text.
    #[build(ignore)]
    pub text: Option<String>,

    /// A callback that is called when an input is received.
    #[build(ignore)]
    #[allow(clippy::type_complexity)]
    pub on_input: Option<Box<dyn FnMut(&mut EventCx, &mut T, String)>>,

    /// A callback that is called when the input is submitted.
    #[build(ignore)]
    #[allow(clippy::type_complexity)]
    pub on_submit: Option<Box<dyn FnMut(&mut EventCx, &mut T, String)>>,

    /// Placeholder text to display when the input is empty.
    pub placeholder: String,

    /// Whether the input is multi-line.
    ///
    /// When disabled (the default), the input will only accept a single line of text.
    pub multiline: bool,

    /// How the text should be capitalized.
    ///
    /// This only affects text input from IMEs, eg. on-screen keyboards like the ones on mobile
    /// devices.
    pub capitalize: Capitalize,

    /// The font size of the text.
    #[styled(default = 16.0)]
    pub font_size: Styled<f32>,

    /// The font family of the text.
    #[styled(default)]
    pub font_family: Styled<FontFamily>,

    /// The font weight of the text.
    #[styled(default)]
    pub font_weight: Styled<FontWeight>,

    /// The font stretch of the text.
    #[styled(default)]
    pub font_stretch: Styled<FontStretch>,

    /// The font.into of the text.
    #[styled(default)]
    pub font_style: Styled<FontStyle>,

    /// The color of the text.
    #[styled(default -> Theme::CONTRAST or Color::BLACK)]
    pub color: Styled<Color>,

    /// The color of the placeholder text.
    #[styled(default -> Theme::CONTRAST_LOW or Color::grayscale(0.9))]
    pub placeholder_color: Styled<Color>,

    /// The vertical alignment of the text.
    #[styled(default)]
    pub align: Styled<TextAlign>,

    /// The line height of the text.
    #[styled(default = 1.2)]
    pub line_height: Styled<f32>,

    /// The text wrap of the text.
    #[styled(default)]
    pub wrap: Styled<TextWrap>,
}

impl<T> Default for TextInput<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> TextInput<T> {
    /// Create a new text input view.
    pub fn new() -> Self {
        Self {
            text: None,
            on_input: None,
            on_submit: None,
            placeholder: String::from("..."),
            multiline: false,
            capitalize: Capitalize::Sentences,
            font_size: TextInputStyle::FONT_SIZE.into(),
            font_family: TextInputStyle::FONT_FAMILY.into(),
            font_weight: TextInputStyle::FONT_WEIGHT.into(),
            font_stretch: TextInputStyle::FONT_STRETCH.into(),
            font_style: TextInputStyle::FONT_STYLE.into(),
            color: TextInputStyle::COLOR.into(),
            placeholder_color: TextInputStyle::PLACEHOLDER_COLOR.into(),
            align: TextInputStyle::ALIGN.into(),
            line_height: TextInputStyle::LINE_HEIGHT.into(),
            wrap: TextInputStyle::WRAP.into(),
        }
    }

    /// Set the text of the input.
    pub fn text(mut self, text: impl ToString) -> Self {
        self.text = Some(text.to_string());
        self
    }

    /// Set the callback that is called when an input is received.
    ///
    /// Note that this doesn't trigger a rebuild automatically.
    pub fn on_input(
        mut self,
        on_change: impl FnMut(&mut EventCx, &mut T, String) + 'static,
    ) -> Self {
        self.on_input = Some(Box::new(on_change));
        self
    }

    /// Set the callback that is called when the input is submitted.
    pub fn on_submit(
        mut self,
        on_submit: impl FnMut(&mut EventCx, &mut T, String) + 'static,
    ) -> Self {
        self.on_submit = Some(Box::new(on_submit));
        self
    }
}

#[doc(hidden)]
pub struct TextInputState {}

impl<T> View<T> for TextInput<T> {
    type State = TextInputState;

    fn build(&mut self, cx: &mut BuildCx, _data: &mut T) -> Self::State {
        cx.set_focusable(true);
        todo!()
    }

    fn rebuild(
        &mut self,
        _state: &mut Self::State,
        _cx: &mut RebuildCx,
        _data: &mut T,
        _old: &Self,
    ) {
        todo!()
    }

    fn event(
        &mut self,
        _state: &mut Self::State,
        _cx: &mut EventCx,
        _data: &mut T,
        _event: &Event,
    ) -> bool {
        todo!()
    }

    fn layout(
        &mut self,
        _state: &mut Self::State,
        _cx: &mut LayoutCx,
        _data: &mut T,
        _space: Space,
    ) -> Size {
        todo!()
    }

    fn draw(&mut self, _state: &mut Self::State, _cx: &mut DrawCx, _data: &mut T) {
        todo!()
    }
}
