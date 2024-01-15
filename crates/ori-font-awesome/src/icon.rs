use ori_core::{
    canvas::{Canvas, Color},
    event::Event,
    layout::{Size, Space},
    rebuild::Rebuild,
    text::{FontStretch, FontStyle, FontWeight, TextAttributes, TextBuffer},
    view::{BaseCx, BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx, View},
};

use crate::{IconFont, IconKind};

const REGULAR: &[u8] = include_bytes!("../font/Font Awesome 6 Free-Regular-400.otf");
const SOLID: &[u8] = include_bytes!("../font/Font Awesome 6 Free-Solid-900.otf");
const BRAND: &[u8] = include_bytes!("../font/Font Awesome 6 Brands-Regular-400.otf");

#[derive(Default)]
struct IconFonts {
    regular: bool,
    solid: bool,
    brand: bool,
}

/// Create a new [`Icon`].
pub fn icon(icon: impl Into<IconKind>) -> Icon {
    Icon::new(icon)
}

/// A view that displays a single icon.
///
/// By default, the icon is rendered using the `icon.font` font family.
/// This uses the [Font Awesome 6 Regular Free](https://fontawesome.com/) font by default.
#[derive(Rebuild)]
pub struct Icon {
    /// The codepoint of the icon to display.
    #[rebuild(layout)]
    pub icon: IconKind,
    /// The size of the icon.
    #[rebuild(layout)]
    pub size: f32,
    /// The color of the icon.
    #[rebuild(layout)]
    pub color: Color,
}

impl Icon {
    /// Create a new icon view.
    pub fn new(icon: impl Into<IconKind>) -> Self {
        Self {
            icon: icon.into(),
            size: 16.0,
            color: Color::BLACK,
        }
    }

    /// Set the size of the icon.
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Set the color of the icon.
    pub fn color(mut self, color: impl Into<Color>) -> Self {
        self.color = color.into();
        self
    }

    fn set_attributes(&self, cx: &mut BaseCx, buffer: &mut TextBuffer) {
        let mut code_point = [0; 4];
        self.icon.code_point().encode_utf8(&mut code_point);

        let family = self.icon.font().family();

        let fonts = cx.context_or_default::<IconFonts>();

        match self.icon.font() {
            IconFont::Regular if !fonts.regular => {
                fonts.regular = true;
                cx.fonts().load_font(REGULAR).unwrap();
            }
            IconFont::Solid if !fonts.solid => {
                fonts.solid = true;
                cx.fonts().load_font(SOLID).unwrap();
            }
            IconFont::Brand if !fonts.brand => {
                fonts.brand = true;
                cx.fonts().load_font(BRAND).unwrap();
            }
            _ => {}
        }

        buffer.set_metrics(cx.fonts(), self.size, 1.0);
        buffer.set_text(
            cx.fonts(),
            std::str::from_utf8(&code_point).unwrap(),
            TextAttributes {
                family,
                stretch: FontStretch::Normal,
                weight: FontWeight::NORMAL,
                style: FontStyle::Normal,
                color: self.color,
            },
        );
    }
}

impl<T> View<T> for Icon {
    type State = TextBuffer;

    fn build(&mut self, cx: &mut BuildCx, _data: &mut T) -> Self::State {
        let mut buffer = TextBuffer::new(cx.fonts(), self.size, 1.0);

        self.set_attributes(cx, &mut buffer);

        buffer
    }

    fn rebuild(&mut self, state: &mut Self::State, cx: &mut RebuildCx, _data: &mut T, old: &Self) {
        Rebuild::rebuild(self, cx, old);
        self.set_attributes(cx, state);
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
        state.set_bounds(cx.fonts(), space.max);
        Size::all(self.size)
    }

    fn draw(
        &mut self,
        state: &mut Self::State,
        cx: &mut DrawCx,
        _data: &mut T,
        canvas: &mut Canvas,
    ) {
        let offset = cx.rect().center() - state.rect().center();

        let mesh = cx.rasterize_text(state, cx.rect());
        canvas.translate(offset);
        canvas.draw(mesh);
    }
}
