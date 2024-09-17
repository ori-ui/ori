use ori_macro::{example, is_mobile, Build, Styled};

use crate::{
    canvas::{BorderRadius, Color},
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    layout::{Axis, Rect, Size, Space, Vector},
    rebuild::Rebuild,
    style::{Styled, Theme},
    transition::Transition,
    view::{Pod, State, View},
};

/// Create a new horizontal [`Scroll`].
pub fn hscroll<V>(view: V) -> Scroll<V> {
    Scroll::new(Axis::Horizontal, view)
}

/// Create a new vertical [`Scroll`].
pub fn vscroll<V>(view: V) -> Scroll<V> {
    Scroll::new(Axis::Vertical, view)
}

/// A scrollable view.
#[example(name = "scroll", width = 400, height = 300)]
#[derive(Styled, Build, Rebuild)]
pub struct Scroll<V> {
    /// The content.
    #[build(ignore)]
    pub content: Pod<V>,

    /// The axis of the scroll.
    #[rebuild(layout)]
    pub axis: Axis,

    /// The transition of the scrollbar.
    #[styled(default = Transition::ease(0.1))]
    pub transition: Styled<Transition>,

    /// The inset of the scrollbar.
    #[rebuild(draw)]
    #[styled(default = 8.0)]
    pub inset: Styled<f32>,

    /// The width of the scrollbar.
    #[rebuild(draw)]
    #[styled(default = 6.0)]
    pub width: Styled<f32>,

    /// The radius of the scrollbar.
    #[rebuild(draw)]
    #[styled(default = BorderRadius::all(3.0))]
    pub border_radius: Styled<BorderRadius>,

    /// The color of the scrollbar.
    #[rebuild(draw)]
    #[styled(default -> Theme::SURFACE_HIGH or Color::grayscale(0.9))]
    pub color: Styled<Color>,

    /// The color of the scrollbar knob.
    #[rebuild(draw)]
    #[styled(default -> Theme::SURFACE_HIGHER or Color::grayscale(0.8))]
    pub knob_color: Styled<Color>,
}

impl<V> Scroll<V> {
    /// Create a new scrollable view.
    pub fn new(axis: Axis, content: V) -> Self {
        Self {
            content: Pod::new(content),
            axis,
            transition: ScrollStyle::TRANSITION.into(),
            inset: ScrollStyle::INSET.into(),
            width: ScrollStyle::WIDTH.into(),
            border_radius: ScrollStyle::BORDER_RADIUS.into(),
            color: ScrollStyle::COLOR.into(),
            knob_color: ScrollStyle::KNOB_COLOR.into(),
        }
    }

    fn scrollbar_rect(&self, style: &ScrollStyle, rect: Rect) -> Rect {
        let (major, minor) = self.axis.unpack(rect.size());

        let length = major - style.inset * 2.0;

        let major_min = style.inset;
        let minor_min = minor - style.width - style.inset;
        let offset = self.axis.pack::<Vector>(major_min, minor_min);

        Rect::min_size(
            rect.top_left() + offset,
            self.axis.pack(length, style.width),
        )
    }

    fn scrollbar_knob_rect(
        &self,
        style: &ScrollStyle,
        rect: Rect,
        overflow: f32,
        scroll: f32,
    ) -> Rect {
        let scrollbar_rect = self.scrollbar_rect(style, rect);

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
pub struct ScrollState {
    style: ScrollStyle,
    dragging: bool,
    scrollbar_hovered: bool,
    scroll: f32,
    t: f32,
}

impl<T, V: View<T>> View<T> for Scroll<V> {
    type State = (ScrollState, State<T, V>);

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        let state = ScrollState {
            style: ScrollStyle::styled(self, cx.styles()),
            dragging: false,
            scrollbar_hovered: false,
            scroll: 0.0,
            t: 0.0,
        };
        let content = self.content.build(cx, data);
        (state, content)
    }

    fn rebuild(
        &mut self,
        (state, content): &mut Self::State,
        cx: &mut RebuildCx,
        data: &mut T,
        old: &Self,
    ) {
        Rebuild::rebuild(self, cx, old);
        state.style.rebuild(self, cx);

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
        if let Event::PointerMoved(e) = event {
            let local = cx.local(e.position);

            let scrollbar_rect = self.scrollbar_rect(&state.style, cx.rect());
            state.scrollbar_hovered = scrollbar_rect.contains(local);

            if cx.is_active() {
                let scroll_start = self.axis.major(scrollbar_rect.min);
                let scroll_end = self.axis.major(scrollbar_rect.max);
                let local_major = self.axis.major(local);

                let scroll_fract = (local_major - scroll_start) / (scroll_end - scroll_start);
                state.scroll = overflow * scroll_fract;
                state.scroll = state.scroll.clamp(0.0, overflow);

                content.translate(self.axis.pack(-state.scroll, 0.0));

                cx.draw();
            } else if state.dragging {
                state.scroll -= self.axis.major(e.delta);
                state.scroll = state.scroll.clamp(0.0, overflow);
                cx.draw();
            }
        }

        if matches!(event, Event::PointerPressed(_)) && cx.has_hovered() && is_mobile!() {
            state.dragging = true;
        }

        if matches!(event, Event::PointerReleased(_)) {
            state.dragging = false;
        }

        if matches!(event, Event::PointerPressed(_)) && state.scrollbar_hovered {
            cx.set_active(true);
            cx.draw();
        }

        if matches!(event, Event::PointerReleased(_)) && cx.is_active() {
            cx.set_active(false);
            cx.draw();
        }

        // propagate event
        self.content.event(content, cx, data, event);

        let on = cx.is_hovered() || cx.has_hovered() || cx.is_active() || state.scrollbar_hovered;

        if !state.style.transition.complete(state.t, on) {
            cx.animate();
        }

        if let Event::Animate(dt) = event {
            if (state.style.transition).step(&mut state.t, on, *dt) {
                cx.animate();
                cx.draw();
            }
        }

        if let Event::PointerScrolled(e) = event {
            if on {
                state.scroll -= e.delta.y * 10.0;
                state.scroll = state.scroll.clamp(0.0, overflow);

                content.translate(self.axis.pack(-state.scroll, 0.0));

                cx.draw();
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
        let min_minor = self.axis.minor(space.min);
        let max_minor = self.axis.minor(space.max);

        let content_space = Space::new(
            self.axis.pack(0.0, min_minor),
            self.axis.pack(f32::INFINITY, max_minor),
        );

        let content_size = self.content.layout(content, cx, data, content_space);

        let size = space.fit(content_size);

        if !size.is_finite() && space.is_finite() {
            tracing::warn!("Contents of a scroll view has an infinite size");
        }

        size
    }

    fn draw(&mut self, (state, content): &mut Self::State, cx: &mut DrawCx, data: &mut T) {
        let overflow = self.overflow(content.size(), cx.size());
        state.scroll = state.scroll.clamp(0.0, overflow);
        content.translate(self.axis.pack(-state.scroll, 0.0));

        cx.trigger(cx.rect());
        cx.masked(cx.rect(), |cx| {
            self.content.draw(content, cx, data);
        });

        let overflow = self.overflow(content.size(), cx.size());

        if overflow == 0.0 {
            return;
        }

        let track_color = state.style.color.fade(0.7);
        let knob_color = state.style.knob_color.fade(0.9);

        cx.quad(
            self.scrollbar_rect(&state.style, cx.rect()),
            track_color.fade(state.style.transition.get(state.t)),
            state.style.border_radius,
            0.0,
            Color::TRANSPARENT,
        );

        cx.quad(
            self.scrollbar_knob_rect(&state.style, cx.rect(), overflow, state.scroll),
            knob_color.fade(state.style.transition.get(state.t)),
            state.style.border_radius,
            0.0,
            Color::TRANSPARENT,
        );
    }
}
