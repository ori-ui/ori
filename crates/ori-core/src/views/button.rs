use glam::Vec2;

use crate::{
    canvas::{BorderRadius, BorderWidth, Canvas, Color},
    event::{Event, PointerEvent},
    layout::{Padding, Size, Space},
    rebuild::Rebuild,
    style::{button, style},
    transition::Transition,
    view::{BuildCx, Content, DrawCx, EventCx, LayoutCx, RebuildCx, State, View},
};

/// Create a new [`Button`].
pub fn button<T, V: View<T>>(content: V) -> Button<T, V> {
    Button::new(content)
}

/// A button.
#[derive(Rebuild)]
pub struct Button<T, V> {
    /// The content.
    pub content: Content<V>,
    /// The callback for when the button is pressed.
    #[allow(clippy::type_complexity)]
    pub on_press: Option<Box<dyn FnMut(&mut EventCx, &mut T)>>,
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

impl<T, V> Button<T, V> {
    /// Create a new [`Button`].
    pub fn new(content: V) -> Self {
        Self {
            content: Content::new(content),
            on_press: None,
            padding: Padding::all(8.0),
            fancy: 0.0,
            transition: style(button::TRANSITION),
            color: style(button::COLOR),
            border_radius: style(button::BORDER_RADIUS),
            border_width: style(button::BORDER_WIDTH),
            border_color: style(button::BORDER_COLOR),
        }
    }

    /// Set the callback for when the button is pressed.
    pub fn on_press(mut self, on_press: impl FnMut(&mut EventCx, &mut T) + 'static) -> Self {
        self.on_press = Some(Box::new(on_press));
        self
    }

    /// Set the padding.
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }

    /// Set the fancy effect.
    pub fn fancy(mut self, fancy: f32) -> Self {
        self.fancy = fancy;
        self
    }

    /// Set the transition.
    pub fn transition(mut self, transition: impl Into<Transition>) -> Self {
        self.transition = transition.into();
        self
    }

    /// Set the color.
    pub fn color(mut self, color: impl Into<Color>) -> Self {
        self.color = color.into();
        self
    }

    /// Set the border radius.
    pub fn border_radius(mut self, border_radius: impl Into<BorderRadius>) -> Self {
        self.border_radius = border_radius.into();
        self
    }

    /// Set the border width.
    pub fn border_width(mut self, border_width: impl Into<BorderWidth>) -> Self {
        self.border_width = border_width.into();
        self
    }

    /// Set the border color.
    pub fn border_color(mut self, border_color: impl Into<Color>) -> Self {
        self.border_color = border_color.into();
        self
    }
}

impl<T, V: View<T>> View<T> for Button<T, V> {
    type State = (f32, State<T, V>);

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        (0.0, self.content.build(cx, data))
    }

    fn rebuild(
        &mut self,
        (_t, state): &mut Self::State,
        cx: &mut RebuildCx,
        data: &mut T,
        old: &Self,
    ) {
        Rebuild::rebuild(self, cx, old);

        self.content.rebuild(state, cx, data, &old.content);
    }

    fn event(
        &mut self,
        (_t, state): &mut Self::State,
        cx: &mut EventCx,
        data: &mut T,
        event: &Event,
    ) {
        self.content.event(state, cx, data, event);

        if event.is_handled() {
            return;
        }

        if let Some(pointer) = event.get::<PointerEvent>() {
            let local = cx.local(pointer.position);
            let hot = cx.rect().contains(local) && !pointer.left;

            if cx.set_hot(hot) {
                cx.request_draw();
            }

            if hot {
                event.handle();
            }

            if hot && pointer.is_press() {
                if let Some(on_press) = &mut self.on_press {
                    on_press(cx, data);
                    cx.request_rebuild();
                }

                cx.set_active(true);
                cx.request_draw();
            } else if cx.is_active() && pointer.is_release() {
                cx.set_active(false);
                cx.request_draw();

                event.handle();
            }
        }
    }

    fn layout(
        &mut self,
        (_t, state): &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
        let content_space = space.shrink(self.padding.size());
        let content_size = self.content.layout(state, cx, data, content_space);

        state.translate(self.padding.offset());

        space.fit(content_size + self.padding.size())
    }

    fn draw(
        &mut self,
        (t, state): &mut Self::State,
        cx: &mut DrawCx,
        data: &mut T,
        canvas: &mut Canvas,
    ) {
        let on = cx.is_hot() && !cx.is_active();
        if self.transition.step(t, on, cx.dt()) {
            cx.request_draw();
        }

        let bright = self.color.brighten(0.05);
        let dark = self.color.darken(0.1);

        let color = if self.fancy != 0.0 {
            self.color.mix(dark, self.transition.on(*t))
        } else {
            self.color.mix(bright, self.transition.on(*t))
        };

        canvas.draw_quad(
            cx.rect(),
            color,
            self.border_radius,
            self.border_width,
            self.border_color,
        );

        if *t == 0.0 || self.fancy == 0.0 {
            self.content.draw(state, cx, data, canvas);
            return;
        }

        let float = Vec2::Y * -self.transition.on(*t) * 4.0;

        let mut layer = canvas.layer();
        layer.translate(float);

        layer.draw_quad(
            cx.rect(),
            self.color.mix(bright, self.transition.on(*t)),
            self.border_radius,
            self.border_width,
            self.border_color,
        );

        self.content.draw(state, cx, data, &mut layer);
    }
}
