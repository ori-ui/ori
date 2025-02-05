use ori_macro::{example, Build};

use crate::{
    canvas::{BorderRadius, BorderWidth, Color, Curve, FillRule, Mask},
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    layout::{Padding, Size, Space},
    rebuild::Rebuild,
    style::{Stylable, Styled, Theme},
    view::{Pod, PodState, View},
};

/// Create a new [`Container`].
pub fn container<V>(view: V) -> Container<V> {
    Container::new(view)
}

/// Create a new [`Container`] with background.
///
/// # Examples
/// ```
/// # use ori_core::{canvas::Color, view::*, views::*};
/// pub fn ui<T>(_data: T) -> impl View<T> {
///     background(Color::RED, text("Hello, World!"))
/// }
/// ````
pub fn background<V>(background: impl Into<Styled<Color>>, view: V) -> Container<V> {
    Container::new(view).background(background)
}

/// A container view.
#[example(name = "container", width = 400, height = 300)]
#[derive(Stylable, Build, Rebuild)]
pub struct Container<V> {
    /// The content.
    #[build(ignore)]
    pub content: Pod<V>,

    /// The background color.
    #[rebuild(draw)]
    #[style(default -> Theme::SURFACE or Color::WHITE)]
    pub background: Styled<Color>,

    /// The border radius.
    #[rebuild(draw)]
    #[style(default)]
    pub border_radius: Styled<BorderRadius>,

    /// The border width.
    #[rebuild(draw)]
    #[style(default)]
    pub border_width: Styled<BorderWidth>,

    /// The border color.
    #[rebuild(draw)]
    #[style(default -> Theme::OUTLINE or Color::BLACK)]
    pub border_color: Styled<Color>,

    /// Whether to mask the content.
    #[rebuild(draw)]
    #[style(default = false)]
    pub mask: Styled<bool>,

    /// The padding.
    #[rebuild(layout)]
    #[style(default)]
    pub padding: Styled<Padding>,
}

impl<V> Container<V> {
    /// Create a new [`Container`].
    pub fn new(content: V) -> Self {
        Self {
            content: Pod::new(content),
            background: Styled::style("container.background"),
            border_radius: Styled::style("container.border-radius"),
            border_width: Styled::style("container.border-width"),
            border_color: Styled::style("container.border-color"),
            mask: Styled::style("container.mask"),
            padding: Styled::style("container.padding"),
        }
    }
}

impl<T, V: View<T>> View<T> for Container<V> {
    type State = (ContainerStyle<V>, PodState<T, V>);

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        cx.set_class("container");

        let style = self.style(cx.styles());
        (style, self.content.build(cx, data))
    }

    fn rebuild(
        &mut self,
        (style, state): &mut Self::State,
        cx: &mut RebuildCx,
        data: &mut T,
        old: &Self,
    ) {
        Rebuild::rebuild(self, cx, old);
        style.rebuild(self, cx);

        self.content.rebuild(state, cx, data, &old.content);
    }

    fn event(
        &mut self,
        (_, state): &mut Self::State,
        cx: &mut EventCx,
        data: &mut T,
        event: &Event,
    ) -> bool {
        self.content.event(state, cx, data, event)
    }

    fn layout(
        &mut self,
        (style, state): &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
        let content_space = space.shrink(style.padding.size());
        let content_size = self.content.layout(state, cx, data, content_space);

        state.translate(style.padding.offset());

        space.fit(content_size + style.padding.size())
    }

    fn draw(&mut self, (style, state): &mut Self::State, cx: &mut DrawCx, data: &mut T) {
        cx.quad(
            cx.rect(),
            style.background,
            style.border_radius,
            style.border_width,
            style.border_color,
        );

        match style.mask {
            true => {
                let mut mask = Curve::new();
                mask.push_rect_with_radius(cx.rect(), style.border_radius);

                cx.masked(Mask::new(mask, FillRule::NonZero), |cx| {
                    self.content.draw(state, cx, data);
                });
            }
            false => {
                self.content.draw(state, cx, data);
            }
        }
    }
}
