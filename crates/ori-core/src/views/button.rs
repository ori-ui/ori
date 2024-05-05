use ori_macro::{example, Build};

use crate::{
    canvas::{BorderRadius, BorderWidth, Color},
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    layout::{Padding, Size, Space, Vector},
    rebuild::Rebuild,
    style::{style, Style, Styles},
    transition::Transition,
    view::{Pod, State, View},
};

/// Create a new [`Button`].
pub fn button<V>(content: V) -> Button<V> {
    Button::new(content)
}

/// The style of a button.
#[derive(Clone, Debug)]
pub struct ButtonStyle {
    /// The padding of the button.
    pub padding: Padding,

    /// The distance of the fancy effect.
    pub fancy: f32,

    /// The transition of the button.
    pub transition: Transition,

    /// The color of the button.
    pub color: Color,

    /// The border radius of the button.
    pub border_radius: BorderRadius,

    /// The border width of the button.
    pub border_width: BorderWidth,

    /// The border color of the button.
    pub border_color: Color,
}

impl Style for ButtonStyle {
    fn style(style: &Styles) -> Self {
        let palette = style.palette();

        Self {
            padding: Padding::all(8.0),
            fancy: 0.0,
            transition: Transition::ease(0.1),
            color: palette.primary,
            border_radius: BorderRadius::all(4.0),
            border_width: BorderWidth::all(0.0),
            border_color: palette.surface_higher,
        }
    }
}

/// A button.
///
/// Can be styled using the [`ButtonStyle`].
#[example(name = "button", width = 400, height = 300)]
#[derive(Build, Rebuild)]
pub struct Button<V> {
    /// The content.
    #[build(ignore)]
    pub content: Pod<V>,

    /// The padding.
    #[rebuild(layout)]
    pub padding: Padding,

    /// The distance of the fancy effect.
    #[rebuild(draw)]
    pub fancy: f32,

    /// The transition of the button.
    #[rebuild(draw)]
    pub transition: Transition,

    /// The color of the button.
    #[rebuild(draw)]
    pub color: Color,

    /// The border radius.
    #[rebuild(draw)]
    pub border_radius: BorderRadius,

    /// The border width.
    #[rebuild(draw)]
    pub border_width: BorderWidth,

    /// The border color.
    #[rebuild(draw)]
    pub border_color: Color,
}

impl<V> Button<V> {
    /// Create a new [`Button`].
    pub fn new(content: V) -> Self {
        Self::styled(content, style())
    }

    /// Create a new [`Button`] with a style.
    pub fn styled(content: V, style: ButtonStyle) -> Self {
        Self {
            content: Pod::new(content),
            padding: style.padding,
            fancy: style.fancy,
            transition: style.transition,
            color: style.color,
            border_radius: style.border_radius,
            border_width: style.border_width,
            border_color: style.border_color,
        }
    }

    /// Set the border width of the top edge.
    pub fn border_top(mut self, width: f32) -> Self {
        self.border_width.top = width;
        self
    }

    /// Set the border width of the right edge.
    pub fn border_right(mut self, width: f32) -> Self {
        self.border_width.right = width;
        self
    }

    /// Set the border width of the bottom edge.
    pub fn border_bottom(mut self, width: f32) -> Self {
        self.border_width.bottom = width;
        self
    }

    /// Set the border width of the left edge.
    pub fn border_left(mut self, width: f32) -> Self {
        self.border_width.left = width;
        self
    }
}

#[doc(hidden)]
pub struct ButtonState {
    pub hot: f32,
    pub active: f32,
}

impl<T, V: View<T>> View<T> for Button<V> {
    type State = (ButtonState, State<T, V>);

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        let state = ButtonState {
            hot: 0.0,
            active: 0.0,
        };

        (state, self.content.build(cx, data))
    }

    fn rebuild(
        &mut self,
        (_state, content): &mut Self::State,
        cx: &mut RebuildCx,
        data: &mut T,
        old: &Self,
    ) {
        Rebuild::rebuild(self, cx, old);

        self.content.rebuild(content, cx, data, &old.content);
    }

    fn event(
        &mut self,
        (state, content): &mut Self::State,
        cx: &mut EventCx,
        data: &mut T,
        event: &Event,
    ) {
        self.content.event(content, cx, data, event);

        if cx.hot_changed() || cx.active_changed() {
            cx.animate();
        }

        if let Event::Animate(dt) = event {
            if self.transition.step(&mut state.hot, cx.is_hot(), *dt) {
                cx.animate();
            }

            if self.transition.step(&mut state.active, cx.is_active(), *dt) {
                cx.animate();
            }

            cx.request_draw();
        }
    }

    fn layout(
        &mut self,
        (_state, content): &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
        let content_space = space.shrink(self.padding.size());
        let content_size = self.content.layout(content, cx, data, content_space);

        content.translate(self.padding.offset());

        space.fit(content_size + self.padding.size())
    }

    fn draw(&mut self, (state, content): &mut Self::State, cx: &mut DrawCx, data: &mut T) {
        cx.hoverable(|cx| {
            let dark = self.color.darken(0.05);
            let dim = self.color.darken(0.025);
            let bright = self.color.lighten(0.05);

            let hot = self.transition.get(state.hot);
            let active = self.transition.get(state.active);

            let face = self.color.mix(bright, hot).mix(dim, active);

            if self.fancy == 0.0 {
                cx.quad(
                    cx.rect(),
                    face,
                    self.border_radius,
                    self.border_width,
                    self.border_color,
                );

                self.content.draw(content, cx, data);
                return;
            }

            let base = dim.mix(dark, 1.0 - active);

            cx.quad(
                cx.rect(),
                base,
                self.border_radius,
                BorderWidth::ZERO,
                Color::TRANSPARENT,
            );

            let float = Vector::NEG_Y * (1.0 - active) * self.fancy;

            cx.translate(float, |cx| {
                cx.quad(
                    cx.rect(),
                    face,
                    self.border_radius,
                    self.border_width,
                    self.border_color,
                );

                self.content.draw(content, cx, data);
            });
        });
    }
}
