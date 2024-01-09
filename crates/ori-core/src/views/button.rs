use ori_macro::Build;

use crate::{
    canvas::{Background, BorderRadius, BorderWidth, Canvas, Color},
    event::{ActiveChanged, AnimationFrame, Event, HotChanged},
    layout::{Padding, Size, Space, Vector},
    rebuild::Rebuild,
    theme::{button, style},
    transition::Transition,
    view::{BuildCx, DrawCx, EventCx, LayoutCx, Pod, RebuildCx, State, View},
};

/// Create a new [`Button`].
pub fn button<V>(content: V) -> Button<V> {
    Button::new(content)
}

/// A button.
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
    pub color: Background,
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
        Self {
            content: Pod::new(content),
            padding: Padding::all(8.0),
            fancy: 0.0,
            transition: style(button::TRANSITION),
            color: style(button::COLOR),
            border_radius: style(button::BORDER_RADIUS),
            border_width: style(button::BORDER_WIDTH),
            border_color: style(button::BORDER_COLOR),
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

        if event.is::<HotChanged>() || event.is::<ActiveChanged>() {
            cx.request_animation_frame();
        }

        if let Some(AnimationFrame(dt)) = event.get() {
            if self.transition.step(&mut state.hot, cx.is_hot(), *dt) {
                cx.request_animation_frame();
            }

            if self.transition.step(&mut state.active, cx.is_active(), *dt) {
                cx.request_animation_frame();
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

    fn draw(
        &mut self,
        (state, content): &mut Self::State,
        cx: &mut DrawCx,
        data: &mut T,
        canvas: &mut Canvas,
    ) {
        canvas.set_view(cx.id());

        let dark = self.color.color.darken(0.05);
        let dim = self.color.color.darken(0.025);
        let bright = self.color.color.lighten(0.05);

        let hot = self.transition.get(state.hot);
        let active = self.transition.get(state.active);

        let face = self.color.color.mix(bright, hot).mix(dim, active);

        let face = Background {
            texture: self.color.texture.clone(),
            color: face,
        };

        if self.fancy == 0.0 {
            canvas.draw_quad(
                cx.rect(),
                face,
                self.border_radius,
                self.border_width,
                self.border_color,
            );

            self.content.draw(content, cx, data, canvas);
            return;
        }

        let base = dim.mix(dark, 1.0 - active);

        canvas.draw_quad(
            cx.rect(),
            Background {
                texture: self.color.texture.clone(),
                color: base,
            },
            self.border_radius,
            BorderWidth::ZERO,
            Color::TRANSPARENT,
        );

        let float = Vector::NEG_Y * (1.0 - active) * self.fancy;

        let mut layer = canvas.layer();
        layer.translate(float);

        layer.draw_quad(
            cx.rect(),
            face,
            self.border_radius,
            self.border_width,
            self.border_color,
        );

        self.content.draw(content, cx, data, &mut layer);
    }
}
