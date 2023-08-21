use ori_core::{
    canvas::{Canvas, Color},
    event::Event,
    layout::{Size, Space},
    rebuild::Rebuild,
    style::em,
    text::{
        FontQuery, FontStretch, FontStyle, FontWeight, Glyphs, TextAlign, TextSection, TextWrap,
    },
    view::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx, View},
};

use crate::{IconFont, IconKind};

const REGULAR: &[u8] = include_bytes!("../font/Font Awesome 6 Free-Regular-400.otf");
const SOLID: &[u8] = include_bytes!("../font/Font Awesome 6 Free-Solid-900.otf");
const BRAND: &[u8] = include_bytes!("../font/Font Awesome 6 Brands-Regular-400.otf");

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
            size: em(1.0),
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
}

impl<T> View<T> for Icon {
    type State = Option<Glyphs>;

    fn build(&mut self, _cx: &mut BuildCx, _data: &mut T) -> Self::State {
        None
    }

    fn rebuild(&mut self, _state: &mut Self::State, cx: &mut RebuildCx, _data: &mut T, old: &Self) {
        Rebuild::rebuild(self, cx, old);
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
        _space: Space,
    ) -> Size {
        let query = FontQuery {
            family: self.icon.font().family(),
            weight: FontWeight::NORMAL,
            stretch: FontStretch::Normal,
            style: FontStyle::Normal,
        };

        if cx.fonts().query(&query).is_none() {
            match self.icon.font() {
                IconFont::Regular => cx.fonts().load_font(REGULAR).unwrap(),
                IconFont::Solid => cx.fonts().load_font(SOLID).unwrap(),
                IconFont::Brand => cx.fonts().load_font(BRAND).unwrap(),
            }
        }

        let mut buffer = [0; 4];

        let section = TextSection {
            text: self.icon.code_point().encode_utf8(&mut buffer),
            font_size: self.size,
            font_family: self.icon.font().family(),
            font_weight: FontWeight::NORMAL,
            font_stretch: FontStretch::Normal,
            font_style: FontStyle::Normal,
            color: self.color,
            v_align: TextAlign::Center,
            h_align: TextAlign::Center,
            line_height: 1.0,
            wrap: TextWrap::None,
            bounds: Size::splat(self.size),
        };

        *state = cx.layout_text(&section);
        Size::splat(self.size)
    }

    fn draw(
        &mut self,
        state: &mut Self::State,
        cx: &mut DrawCx,
        _data: &mut T,
        canvas: &mut Canvas,
    ) {
        if let Some(glyphs) = state {
            if let Some(mesh) = cx.text_mesh(glyphs, cx.rect()) {
                canvas.draw(mesh);
            }
        }
    }
}
