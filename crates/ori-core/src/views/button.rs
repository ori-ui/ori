use glam::Vec2;

use crate::{
    button, style, Canvas, Color, DrawCx, Event, EventCx, LayoutCx, Padding, Pod, PodState,
    PointerEvent, Rebuild, RebuildCx, Size, Space, Transition, View,
};

pub fn button<T, V: View<T>>(content: V, on_click: impl Fn(&mut T) + 'static) -> Button<T, V> {
    Button::new(content, on_click)
}

#[derive(Rebuild)]
pub struct Button<T, V> {
    pub content: Pod<T, V>,
    pub on_press: Box<dyn Fn(&mut T)>,
    #[rebuild(layout)]
    pub padding: Padding,
    #[rebuild(draw)]
    pub fancy: f32,
    #[rebuild(draw)]
    pub transition: Transition,
    #[rebuild(draw)]
    pub color: Color,
    #[rebuild(draw)]
    pub border_radius: [f32; 4],
    #[rebuild(draw)]
    pub border_width: [f32; 4],
    #[rebuild(draw)]
    pub border_color: Color,
}

impl<T, V: View<T>> Button<T, V> {
    pub fn new(content: V, on_click: impl Fn(&mut T) + 'static) -> Self {
        Self {
            content: Pod::new(content),
            on_press: Box::new(on_click),
            padding: Padding::all(8.0),
            fancy: 0.0,
            transition: style(button::TRANSITION),
            color: style(button::COLOR),
            border_radius: style(button::BORDER_RADIUS),
            border_width: style(button::BORDER_WIDTH),
            border_color: style(button::BORDER_COLOR),
        }
    }

    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }

    pub fn fancy(mut self, fancy: f32) -> Self {
        self.fancy = fancy;
        self
    }

    pub fn color(mut self, color: impl Into<Color>) -> Self {
        self.color = color.into();
        self
    }

    pub fn border_radius(mut self, border_radius: impl Into<[f32; 4]>) -> Self {
        self.border_radius = border_radius.into();
        self
    }

    pub fn border_width(mut self, border_width: impl Into<[f32; 4]>) -> Self {
        self.border_width = border_width.into();
        self
    }

    pub fn border_color(mut self, border_color: impl Into<Color>) -> Self {
        self.border_color = border_color.into();
        self
    }

    fn handle_pointer_event(&self, cx: &mut EventCx, data: &mut T, event: &PointerEvent) -> bool {
        let local = cx.local(event.position);
        let over = cx.rect().contains(local) && !event.left;

        if cx.set_hot(over) {
            cx.request_draw();
        }

        if over && event.is_press() {
            (self.on_press)(data);

            cx.set_active(true);
            cx.request_rebuild();

            return true;
        } else if cx.is_active() && event.is_release() {
            cx.set_active(false);
            cx.request_draw();

            return true;
        }

        false
    }
}

impl<T, V: View<T>> View<T> for Button<T, V> {
    type State = (f32, PodState<T, V>);

    fn build(&self) -> Self::State {
        (0.0, self.content.build())
    }

    fn rebuild(&mut self, cx: &mut RebuildCx, old: &Self, (_t, state): &mut Self::State) {
        Rebuild::rebuild(self, cx, old);

        self.content.rebuild(cx, &old.content, state);
    }

    fn event(
        &mut self,
        cx: &mut EventCx,
        (_t, state): &mut Self::State,
        data: &mut T,
        event: &Event,
    ) {
        self.content.event(cx, state, data, event);

        if event.is_handled() {
            return;
        }

        if let Some(pointer_event) = event.get::<PointerEvent>() {
            if self.handle_pointer_event(cx, data, pointer_event) {
                event.handle();
            }
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx, (_t, state): &mut Self::State, space: Space) -> Size {
        let content_space = space.shrink(self.padding.size());
        let content_size = self.content.layout(cx, state, content_space);

        state.translate(self.padding.offset());

        space.fit(content_size + self.padding.size())
    }

    fn draw(&mut self, cx: &mut DrawCx, (t, state): &mut Self::State, canvas: &mut Canvas) {
        let on = cx.is_hot() && !cx.is_active();
        if self.transition.step(t, on, cx.dt()) {
            cx.request_draw();
        }

        let bright = self.color.brighten(0.05);
        let dark = self.color.darken(0.1);

        let float = Vec2::Y * -self.transition.on(*t) * 4.0;

        let color = if self.fancy != 0.0 {
            self.color.mix(dark, self.transition.on(*t))
        } else {
            self.color.mix(bright, self.transition.on(*t))
        };

        canvas.draw_quad(cx.rect(), color, [6.0; 4], [0.0; 4], Color::TRANSPARENT);

        if *t == 0.0 || self.fancy == 0.0 {
            self.content.draw(cx, state, canvas);
            return;
        }

        let mut layer = canvas.layer();
        layer.translate(float);

        layer.draw_quad(
            cx.rect() + float,
            self.color.mix(bright, self.transition.on(*t)),
            [6.0; 4],
            [0.0; 4],
            Color::TRANSPARENT,
        );

        self.content.draw(cx, state, &mut layer);
    }
}
