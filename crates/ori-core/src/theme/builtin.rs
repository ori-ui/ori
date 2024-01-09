//! Builtin styles.

use crate::canvas::{Background, BorderRadius, BorderWidth, Color};

use super::Theme;

/// Styles for [`Text`](crate::views::Text)s.
pub mod text {
    use crate::{
        canvas::Color,
        text::{FontFamily, FontStretch, FontStyle, FontWeight, TextAlign, TextWrap},
        theme::{Key, Palette, Theme},
    };

    /// The font size.
    pub const FONT_SIZE: Key<f32> = Key::new("text.font_size");
    /// The font family.
    pub const FONT_FAMILY: Key<FontFamily> = Key::new("text.font_family");
    /// The font weight.
    pub const FONT_WEIGHT: Key<FontWeight> = Key::new("text.font_weight");
    /// The font stretch.
    pub const FONT_STRETCH: Key<FontStretch> = Key::new("text.font_stretch");
    /// The font style.
    pub const FONT_STYLE: Key<FontStyle> = Key::new("text.font_style");
    /// The color.
    pub const COLOR: Key<Color> = Key::new("text.color");
    /// The vertical alignment.
    pub const V_ALIGN: Key<TextAlign> = Key::new("text.v_align");
    /// The horizontal alignment.
    pub const ALIGN: Key<TextAlign> = Key::new("text.h_align");
    /// The line height.
    pub const LINE_HEIGHT: Key<f32> = Key::new("text.line_height");
    /// The text wrap.
    pub const WRAP: Key<TextWrap> = Key::new("text.wrap");

    pub(super) fn builtin(theme: &mut Theme) {
        theme.set(FONT_SIZE, 16.0);
        theme.set(FONT_FAMILY, FontFamily::SansSerif);
        theme.set(FONT_WEIGHT, FontWeight::NORMAL);
        theme.set(FONT_STRETCH, FontStretch::Normal);
        theme.set(FONT_STYLE, FontStyle::Normal);
        theme.map(COLOR, |theme| theme.get(Palette::TEXT));
        theme.set(V_ALIGN, TextAlign::Top);
        theme.set(ALIGN, TextAlign::Left);
        theme.set(LINE_HEIGHT, 1.0);
        theme.set(WRAP, TextWrap::Word);
    }
}

/// Styles for [`TextInput`](crate::views::TextInput)s.
pub mod text_input {
    use crate::{
        canvas::Color,
        text::{FontFamily, FontStretch, FontStyle, FontWeight, TextAlign, TextWrap},
        theme::{Key, Palette, Theme},
    };

    /// The font size.
    pub const FONT_SIZE: Key<f32> = Key::new("text.font_size");
    /// The font family.
    pub const FONT_FAMILY: Key<FontFamily> = Key::new("text.font_family");
    /// The font weight.
    pub const FONT_WEIGHT: Key<FontWeight> = Key::new("text.font_weight");
    /// The font stretch.
    pub const FONT_STRETCH: Key<FontStretch> = Key::new("text.font_stretch");
    /// The font style.
    pub const FONT_STYLE: Key<FontStyle> = Key::new("text.font_style");
    /// The color.
    pub const COLOR: Key<Color> = Key::new("text.color");
    /// The vertical alignment.
    pub const V_ALIGN: Key<TextAlign> = Key::new("text.v_align");
    /// The horizontal alignment.
    pub const H_ALIGN: Key<TextAlign> = Key::new("text.h_align");
    /// The line height.
    pub const LINE_HEIGHT: Key<f32> = Key::new("text.line_height");
    /// The text wrap.
    pub const WRAP: Key<TextWrap> = Key::new("text.wrap");

    pub(super) fn builtin(theme: &mut Theme) {
        theme.set(FONT_SIZE, 16.0);
        theme.set(FONT_FAMILY, FontFamily::SansSerif);
        theme.set(FONT_WEIGHT, FontWeight::NORMAL);
        theme.set(FONT_STRETCH, FontStretch::Normal);
        theme.set(FONT_STYLE, FontStyle::Normal);
        theme.map(COLOR, |theme| theme.get(Palette::TEXT));
        theme.set(V_ALIGN, TextAlign::Top);
        theme.set(H_ALIGN, TextAlign::Left);
        theme.set(LINE_HEIGHT, 1.0);
        theme.set(WRAP, TextWrap::Word);
    }
}

/// Styles for [`Alt`](crate::views::Alt)s.
pub mod alt {
    use crate::{
        canvas::{BorderRadius, BorderWidth, Color},
        layout::Padding,
        theme::{Key, Palette, Theme},
    };

    /// The padding.
    pub const PADDING: Key<Padding> = Key::new("alt.padding");
    /// The background color.
    pub const BACKGROUND: Key<Color> = Key::new("alt.background");
    /// The border radius.
    pub const BORDER_RADIUS: Key<BorderRadius> = Key::new("alt.border_radius");
    /// The border width.
    pub const BORDER_WIDTH: Key<BorderWidth> = Key::new("alt.border_width");
    /// The border color.
    pub const BORDER_COLOR: Key<Color> = Key::new("alt.border_color");

    pub(super) fn builtin(theme: &mut Theme) {
        theme.set(PADDING, [4.0, 2.0]);
        theme.map(BACKGROUND, |theme| theme.get(Palette::BACKGROUND_DARKER));
        theme.set(BORDER_RADIUS, BorderRadius::all(4.0));
        theme.set(BORDER_WIDTH, BorderWidth::all(0.0));
        theme.set(BORDER_COLOR, Color::TRANSPARENT);
    }
}

/// Styles for [`Container`](crate::views::Container)s.
pub mod container {
    use crate::{
        canvas::{Background, BorderRadius, BorderWidth, Color},
        theme::{Key, Theme},
    };

    /// The background color.
    pub const BACKGROUND: Key<Background> = Key::new("container.background");
    /// The border radius.
    pub const BORDER_RADIUS: Key<BorderRadius> = Key::new("container.border_radius");
    /// The border width.
    pub const BORDER_WIDTH: Key<BorderWidth> = Key::new("container.border_width");
    /// The border color.
    pub const BORDER_COLOR: Key<Color> = Key::new("container.border_color");

    pub(super) fn builtin(theme: &mut Theme) {
        theme.set(BACKGROUND, Color::TRANSPARENT);
        theme.set(BORDER_RADIUS, BorderRadius::all(0.0));
        theme.set(BORDER_WIDTH, BorderWidth::all(0.0));
        theme.set(BORDER_COLOR, Color::TRANSPARENT);
    }
}

/// Styles for [`Scroll`](crate::views::Scroll)s.
pub mod scroll {
    use crate::{
        canvas::{BorderRadius, Color},
        theme::{Key, Palette, Theme},
        transition::Transition,
    };

    /// The transition when the scrollbar is hovered.
    pub const TRANSITION: Key<Transition> = Key::new("scroll.transition");
    /// The width of the scrollbar.
    pub const WIDTH: Key<f32> = Key::new("scroll.width");
    /// The padding of the scrollbar.
    pub const INSET: Key<f32> = Key::new("scroll.inset");
    /// The border radius of the scrollbar.
    pub const BORDER_RADIUS: Key<BorderRadius> = Key::new("scroll.border_radius");
    /// The color of the scrollbar.
    pub const COLOR: Key<Color> = Key::new("scroll.color");
    /// The color of the scrollbar knob.
    pub const KNOB_COLOR: Key<Color> = Key::new("scroll.knob_color");

    pub(super) fn builtin(theme: &mut Theme) {
        theme.set(TRANSITION, Transition::ease(0.1));
        theme.set(WIDTH, 8.0);
        theme.set(INSET, 6.0);
        theme.set(BORDER_RADIUS, BorderRadius::all(4.0));
        theme.map(COLOR, |theme| theme.get(Palette::SECONDARY_DARK));
        theme.map(KNOB_COLOR, |theme| theme.get(Palette::SECONDARY_DARKER));
    }
}

/// Styles for [`Button`](crate::views::Button)s.
pub mod button {
    use crate::{
        canvas::{Background, BorderRadius, BorderWidth, Color},
        theme::{Key, Palette, Theme},
        transition::Transition,
    };

    /// The transition when the button is hovered.
    pub const TRANSITION: Key<Transition> = Key::new("button.transition");
    /// The color.
    pub const COLOR: Key<Background> = Key::new("button.color");
    /// The border radius.
    pub const BORDER_RADIUS: Key<BorderRadius> = Key::new("button.border_radius");
    /// The border width.
    pub const BORDER_WIDTH: Key<BorderWidth> = Key::new("button.border_width");
    /// The border color.
    pub const BORDER_COLOR: Key<Color> = Key::new("button.border_color");

    pub(super) fn builtin(theme: &mut Theme) {
        theme.set(TRANSITION, Transition::ease(0.1));
        theme.map(COLOR, |theme| Background::new(theme.get(Palette::PRIMARY)));
        theme.set(BORDER_RADIUS, BorderRadius::all(8.0));
        theme.set(BORDER_WIDTH, BorderWidth::all(0.0));
        theme.set(BORDER_COLOR, Color::TRANSPARENT);
    }
}

/// Styles for [`Checkbox`](crate::views::Checkbox)s.
pub mod checkbox {
    use crate::{
        theme::{Key, Palette, Theme},
        transition::Transition,
    };

    use super::{Background, BorderRadius, BorderWidth, Color};

    /// The transition when the checkbox is hovered.
    pub const TRANSITION: Key<Transition> = Key::new("checkbox.transition");
    /// The size of the checkbox.
    pub const SIZE: Key<f32> = Key::new("checkbox.size");
    /// The color of the checkmark.
    pub const COLOR: Key<Color> = Key::new("checkbox.color");
    /// The stroke width of the checkmark.
    pub const STROKE: Key<f32> = Key::new("checkbox.stroke");
    /// The background color.
    pub const BACKGROUND: Key<Background> = Key::new("checkbox.background");
    /// The border radius.
    pub const BORDER_RADIUS: Key<BorderRadius> = Key::new("checkbox.border_radius");
    /// The border width.
    pub const BORDER_WIDTH: Key<BorderWidth> = Key::new("checkbox.border_width");
    /// The border color.
    pub const BORDER_COLOR: Key<Color> = Key::new("checkbox.border_color");

    pub(super) fn builtin(theme: &mut Theme) {
        theme.set(TRANSITION, Transition::ease(0.1));
        theme.set(SIZE, 24.0);
        theme.map(COLOR, |theme| theme.get(Palette::ACCENT));
        theme.set(STROKE, 2.0);
        theme.set(BACKGROUND, Color::TRANSPARENT);
        theme.set(BORDER_RADIUS, BorderRadius::all(6.0));
        theme.set(BORDER_WIDTH, BorderWidth::all(2.0));
        theme.map(BORDER_COLOR, |theme| theme.get(Palette::TEXT_LIGHTER));
    }
}

impl Theme {
    /// Get the builtin theme.
    pub fn builtin() -> Self {
        let mut theme = Self::new();

        text::builtin(&mut theme);
        text_input::builtin(&mut theme);
        alt::builtin(&mut theme);
        container::builtin(&mut theme);
        scroll::builtin(&mut theme);
        button::builtin(&mut theme);
        checkbox::builtin(&mut theme);

        theme
    }
}
