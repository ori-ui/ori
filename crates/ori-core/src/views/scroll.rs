use ori_macro::{example, is_mobile, Build};

use crate::{
    canvas::{BorderRadius, Color},
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    layout::{Axis, Rect, Size, Space, Vector},
    rebuild::Rebuild,
    style::{Stylable, Style, StyleBuilder, Theme},
    transition::Transition,
    view::{Pod, PodState, View},
};

/// Create a new horizontal [`Scroll`].
pub fn hscroll<V>(view: V) -> Scroll<V> {
    Scroll::new(Axis::Horizontal, view)
}

/// Create a new vertical [`Scroll`].
pub fn vscroll<V>(view: V) -> Scroll<V> {
    Scroll::new(Axis::Vertical, view)
}

/// The style of a scroll view.
#[derive(Clone, Rebuild)]
pub struct ScrollStyle {
    /// The transition of the scrollbar.
    #[rebuild(draw)]
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

impl Style for ScrollStyle {
    fn default_style() -> StyleBuilder<Self> {
        StyleBuilder::new(|theme: &Theme| ScrollStyle {
            transition: Transition::ease(0.1),
            inset: 8.0,
            width: 6.0,
            border_radius: BorderRadius::all(3.0),
            color: theme.contrast_low(),
            knob_color: theme.contrast,
        })
    }
}

/// A scrollable view.
#[example(name = "scroll", width = 400, height = 300)]
#[derive(Build, Rebuild)]
pub struct Scroll<V> {
    /// The content.
    #[build(ignore)]
    pub content: Pod<V>,

    /// The axis of the scroll.
    #[rebuild(layout)]
    pub axis: Axis,

    /// The transition of the scrollbar.
    pub transition: Option<Transition>,

    /// The inset of the scrollbar.
    pub inset: Option<f32>,

    /// The width of the scrollbar.
    pub width: Option<f32>,

    /// The radius of the scrollbar.
    pub border_radius: Option<BorderRadius>,

    /// The color of the scrollbar.
    pub color: Option<Color>,

    /// The color of the scrollbar knob.
    pub knob_color: Option<Color>,
}

impl<V> Scroll<V> {
    /// Create a new scrollable view.
    pub fn new(axis: Axis, content: V) -> Self {
        Self {
            content: Pod::new(content),
            axis,
            transition: None,
            inset: None,
            width: None,
            border_radius: None,
            color: None,
            knob_color: None,
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

impl<V> Stylable for Scroll<V> {
    type Style = ScrollStyle;

    fn style(&self, style: &Self::Style) -> Self::Style {
        ScrollStyle {
            transition: self.transition.unwrap_or(style.transition),
            inset: self.inset.unwrap_or(style.inset),
            width: self.width.unwrap_or(style.width),
            border_radius: self.border_radius.unwrap_or(style.border_radius),
            color: self.color.unwrap_or(style.color),
            knob_color: self.knob_color.unwrap_or(style.knob_color),
        }
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
    type State = (ScrollState, PodState<T, V>);

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        let state = ScrollState {
            style: self.style(cx.style()),
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
        self.rebuild_style(cx, &mut state.style);

        self.content.rebuild(content, cx, data, &old.content);
    }

    fn event(
        &mut self,
        (state, content): &mut Self::State,
        cx: &mut EventCx,
        data: &mut T,
        event: &Event,
    ) -> bool {
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

        let mut handled = false;

        if matches!(event, Event::PointerPressed(_)) && state.scrollbar_hovered {
            handled = true;
            cx.set_active(true);
            cx.draw();
        }

        if matches!(event, Event::PointerReleased(_)) && cx.is_active() {
            handled = true;
            cx.set_active(false);
            cx.draw();
        }

        // propagate event
        handled = self.content.event_maybe(handled, content, cx, data, event);

        if is_mobile!() && !handled {
            if matches!(event, Event::PointerPressed(_)) && cx.has_hovered() {
                state.dragging = true;
            }

            if matches!(event, Event::PointerReleased(_)) && state.dragging {
                state.dragging = false;
            }
        }

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
            if on && !handled {
                handled = true;

                state.scroll -= e.delta.y;
                state.scroll = state.scroll.clamp(0.0, overflow);

                content.translate(self.axis.pack(-state.scroll, 0.0));

                cx.draw();
            }
        }

        handled
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

        cx.hoverable(|cx| {
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
        });
    }
}
