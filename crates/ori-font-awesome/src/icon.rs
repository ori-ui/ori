use ori_core::{
    canvas::Color,
    context::{BaseCx, BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    layout::{Size, Space},
    style::{key, Styled},
    text::{FontStretch, FontStyle, TextAttributes, TextBuffer},
    view::View,
};
use ori_macro::{include_font, Build, Styled};

use crate::{IconCode, IconFont};

/// Create a new [`Icon`].
pub fn icon(icon: impl Into<IconCode>) -> Icon {
    Icon::new(icon)
}

/// A view that displays a single icon.
///
/// By default, the icon is rendered using the `icon.font` font family.
/// This uses the [Font Awesome 6 Regular Free](https://fontawesome.com/) font by default.
#[derive(Styled, Build)]
pub struct Icon {
    /// The codepoint of the icon to display.
    pub icon: IconCode,

    /// Whether the icon is solid or regular.
    ///
    /// This only affects the rendering of the icon if the icon is available in both styles.
    pub solid: bool,

    /// The size of the icon.
    #[rebuild(layout)]
    #[styled(default = 16.0)]
    pub size: Styled<f32>,

    /// The color of the icon.
    #[rebuild(draw)]
    #[styled(default -> "palette.contrast" or Color::BLACK)]
    pub color: Styled<Color>,
}

impl Icon {
    /// Create a new icon view.
    pub fn new(icon: impl Into<IconCode>) -> Self {
        Self {
            icon: icon.into(),
            solid: false,
            size: key("icon.size"),
            color: key("icon.color"),
        }
    }

    /// Get the font to use for the icon.
    pub fn font(&self) -> IconFont {
        if self.icon.fonts().contains(&IconFont::Solid)
            && self.icon.fonts().contains(&IconFont::Regular)
        {
            if self.solid {
                return IconFont::Solid;
            } else {
                return IconFont::Regular;
            }
        }

        self.icon.fonts()[0]
    }

    fn set_attributes(&self, cx: &mut BaseCx, buffer: &mut TextBuffer, style: &IconStyle) {
        struct FontsLoaded;

        // ensure that all the fonts are loaded
        if !cx.contains_context::<FontsLoaded>() {
            cx.fonts().load_font(include_font!("font")).unwrap();

            cx.insert_context(FontsLoaded);
        }

        let font = self.font();

        buffer.set_metrics(cx.fonts(), style.size, 1.0);
        buffer.set_text(
            cx.fonts(),
            self.icon.as_str(),
            TextAttributes {
                family: font.family(),
                stretch: FontStretch::Normal,
                weight: font.weight(),
                style: FontStyle::Normal,
            },
        );
    }
}

#[doc(hidden)]
pub struct IconState {
    style: IconStyle,
    buffer: TextBuffer,
}

impl<T> View<T> for Icon {
    type State = IconState;

    fn build(&mut self, cx: &mut BuildCx, _data: &mut T) -> Self::State {
        let style = IconStyle::styled(self, cx.styles());
        let mut buffer = TextBuffer::new(cx.fonts(), style.size, 1.0);

        self.set_attributes(cx, &mut buffer, &style);

        IconState { style, buffer }
    }

    fn rebuild(&mut self, state: &mut Self::State, cx: &mut RebuildCx, _data: &mut T, old: &Self) {
        let size = state.style.size;
        let color = state.style.color;

        state.style.rebuild(self, cx);

        if self.icon != old.icon
            || self.solid != old.solid
            || size != state.style.size
            || color != state.style.color
        {
            self.set_attributes(cx, &mut state.buffer, &state.style);
            cx.layout();
        }
    }

    fn event(
        &mut self,
        _state: &mut Self::State,
        _cx: &mut EventCx,
        _data: &mut T,
        _event: &Event,
    ) {
    }

    fn layout(
        &mut self,
        state: &mut Self::State,
        cx: &mut LayoutCx,
        _data: &mut T,
        space: Space,
    ) -> Size {
        state.buffer.set_bounds(cx.fonts(), space.max);

        Size::all(state.style.size)
    }

    fn draw(&mut self, state: &mut Self::State, cx: &mut DrawCx, _data: &mut T) {
        let offset = cx.rect().center() - state.buffer.rect().center();
        cx.text(&state.buffer, state.style.color, offset);
    }
}
