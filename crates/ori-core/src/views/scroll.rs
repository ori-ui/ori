use crate::{
    canvas::{BorderRadius, Canvas, Color},
    event::{AnimationFrame, Event, HotChanged, PointerEvent},
    layout::{Axis, Rect, Size, Space, Vector},
    log::warn_internal,
    rebuild::Rebuild,
    theme::{scroll, style},
    transition::Transition,
    view::{BuildCx, DrawCx, EventCx, LayoutCx, Pod, RebuildCx, State, View},
};

/// Create a new horizontal [`Scroll`].
pub fn hscroll<V>(content: V) -> Scroll<V> {
    Scroll::new(Axis::Horizontal, content)
}

/// Create a new vertical [`Scroll`].
pub fn vscroll<V>(content: V) -> Scroll<V> {
    Scroll::new(Axis::Vertical, content)
}

/// A scrollable view.
#[derive(Rebuild)]
pub struct Scroll<V> {
    /// The content.
    pub content: Pod<V>,
    /// The axis of the scroll.
    #[rebuild(layout)]
    pub axis: Axis,
    /// The transition of the scrollbar.
    pub transition: Transition,
    /// The inset of the scrollbar.
    #[rebuild(draw)]
    pub inset: f32,
    /// The width of the scrollbar.
    #[rebuild(draw)]
    pub width: f32,
    /// The radius of the scrollbar.
    #[rebuild(draw)]
    pub border_radius: BorderRadius,
    /// The color of the scrollbar.
    #[rebuild(draw)]
    pub color: Color,
    /// The color of the scrollbar knob.
    #[rebuild(draw)]
    pub knob_color: Color,
}

impl<V> Scroll<V> {
    /// Create a new scrollable view.
    pub fn new(axis: Axis, content: V) -> Self {
        Self {
            content: Pod::new(content),
            axis,
            transition: style(scroll::TRANSITION),
            width: style(scroll::WIDTH),
            inset: style(scroll::INSET),
            border_radius: style(scroll::BORDER_RADIUS),
            color: style(scroll::COLOR),
            knob_color: style(scroll::KNOB_COLOR),
        }
    }

    fn scrollbar_rect(&self, rect: Rect) -> Rect {
        let (major, minor) = self.axis.unpack(rect.size());

        let length = major - self.inset * 2.0;

        let major_min = self.inset;
        let minor_min = minor - self.width - self.inset;
        let offset = self.axis.pack::<Vector>(major_min, minor_min);

        Rect::min_size(rect.top_left() + offset, self.axis.pack(length, self.width))
    }

    fn scrollbar_knob_rect(&self, rect: Rect, overflow: f32, scroll: f32) -> Rect {
        let scrollbar_rect = self.scrollbar_rect(rect);

        let (major_min, minor_min) = self.axis.unpack(scrollbar_rect.min);
        let (major_size, minor_size) = self.axis.unpack(scrollbar_rect.size());

        let knob_length = major_size / 4.0;

        let scroll_fract = scroll / overflow;

        let major_min = major_min + scroll_fract * (major_size - knob_length);

        Rect::min_size(
            self.axis.pack(major_min, minor_min),
            self.axis.pack(knob_length, minor_size),
        )
    }

    fn overflow(&self, content: Size, size: Size) -> f32 {
        self.axis.major(content - size).max(0.0)
    }
}

#[doc(hidden)]
#[derive(Default)]
pub struct ScrollState {
    scrollbar_hot: bool,
    scroll: f32,
    t: f32,
}

impl<T, V: View<T>> View<T> for Scroll<V> {
    type State = (ScrollState, State<T, V>);

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        let state = ScrollState::default();
        let content = self.content.build(cx, data);
        (state, content)
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
        let overflow = self.overflow(content.size(), cx.size());

        // handle ponter event
        if let Some(pointer) = event.get::<PointerEvent>() {
            let local = cx.local(pointer.position);

            let scrollbar_rect = self.scrollbar_rect(cx.rect());
            state.scrollbar_hot = scrollbar_rect.contains(local);

            cx.request_animation_frame();

            if cx.is_active() || cx.has_hot() || state.scrollbar_hot {
                // handle pointer events when scrollbar is pressed
                if state.scrollbar_hot && pointer.is_press() {
                    cx.set_active(true);
                    cx.request_draw();
                } else if cx.is_active() && pointer.is_release() {
                    cx.set_active(false);
                    cx.request_draw();
                }

                // handle pointer events when scrollbar is dragged
                if cx.is_active() {
                    let scroll_start = self.axis.major(scrollbar_rect.min);
                    let scroll_end = self.axis.major(scrollbar_rect.max);
                    let local_major = self.axis.major(local);

                    let scroll_fract = (local_major - scroll_start) / (scroll_end - scroll_start);
                    state.scroll = overflow * scroll_fract;
                    state.scroll = state.scroll.clamp(0.0, overflow);

                    content.translate(self.axis.pack(-state.scroll, 0.0));

                    cx.request_draw();
                }
            }
        }

        // propagate event
        self.content.event(content, cx, data, event);

        // animate the scrollbar
        if event.is::<HotChanged>() {
            cx.request_animation_frame();
        }

        if let Some(AnimationFrame(dt)) = event.get() {
            let on = cx.has_hot() || cx.is_active() || state.scrollbar_hot;

            if (self.transition).step(&mut state.t, on, *dt) {
                cx.request_animation_frame();
            }

            cx.request_draw();
        }

        // handle scroll
        if event.is_handled() || !(cx.has_hot() || cx.is_active()) {
            return;
        }

        if let Some(pointer) = event.get::<PointerEvent>() {
            if pointer.is_scroll() {
                state.scroll -= pointer.scroll.y * 10.0;
                state.scroll = state.scroll.clamp(0.0, overflow);

                content.translate(self.axis.pack(-state.scroll, 0.0));

                cx.request_draw();
            }
        }
    }

    fn layout(
        &mut self,
        (_state, content): &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
        let minor_max = self.axis.minor(space.max);
        let max = self.axis.pack(f32::INFINITY, minor_max);

        let content_space = Space::new(Size::ZERO, max);
        let content_size = self.content.layout(content, cx, data, content_space);

        let size = space.fit(content_size);

        if !size.is_finite() {
            warn_internal!("Contents of a scroll view has an infinite size");
        }

        size
    }

    fn draw(
        &mut self,
        (state, content): &mut Self::State,
        cx: &mut DrawCx,
        data: &mut T,
        canvas: &mut Canvas,
    ) {
        canvas.view(cx.id());
        canvas.trigger(cx.rect());

        let overflow = self.overflow(content.size(), cx.size());
        state.scroll = state.scroll.clamp(0.0, overflow);
        content.translate(self.axis.pack(-state.scroll, 0.0));

        let mut content_layer = canvas.layer();
        content_layer.clip &= cx.rect().transform(content_layer.transform);

        self.content.draw(content, cx, data, &mut content_layer);

        let overflow = self.overflow(content.size(), cx.size());

        if overflow == 0.0 {
            return;
        }

        let mut scrollbar_layer = canvas.layer();
        scrollbar_layer.depth += 100.0;

        scrollbar_layer.draw_quad(
            self.scrollbar_rect(cx.rect()),
            self.color.fade(0.7).fade(self.transition.get(state.t)),
            self.border_radius,
            0.0,
            Color::TRANSPARENT,
        );

        scrollbar_layer.draw_quad(
            self.scrollbar_knob_rect(cx.rect(), overflow, state.scroll),
            self.knob_color.fade(0.9).fade(self.transition.get(state.t)),
            self.border_radius,
            0.0,
            Color::TRANSPARENT,
        );
    }
}
