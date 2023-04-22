use glam::Vec2;
use ily_graphics::{Quad, Rect};

use crate::{
    BoxConstraints, Children, Context, DrawContext, Event, EventContext, FlexLayout, LayoutContext,
    Parent, PointerEvent, Style, View,
};

#[derive(Default)]
pub struct Scroll {
    content: Children,
}

impl Scroll {
    fn scrollbar_rect(&self, state: &ScrollState, cx: &mut impl Context) -> Rect {
        let width = cx.style_range("scrollbar-width", 0.0..cx.rect().width());
        let padding = cx.style_range("scrollbar-padding", 0.0..cx.rect().width() - width);
        let height = cx.style_range("scrollbar-height", 0.0..cx.rect().height() - padding * 2.0);

        let scrollbar_size = Vec2::new(width, height);
        let range = cx.rect().height() - scrollbar_size.y - padding * 2.0;

        Rect::min_size(
            Vec2::new(
                cx.rect().right() - scrollbar_size.x - padding,
                cx.rect().top() + range * state.scroll.y + padding,
            ),
            scrollbar_size,
        )
    }

    fn scrollbar_track_rect(&self, cx: &mut impl Context) -> Rect {
        let width = cx.style_range("scrollbar-width", 0.0..cx.rect().width());
        let padding = cx.style_range("scrollbar-padding", 0.0..cx.rect().width() - width);

        Rect::min_size(
            Vec2::new(
                cx.rect().right() - width - padding,
                cx.rect().top() + padding,
            ),
            Vec2::new(width, cx.rect().height() - padding * 2.0),
        )
    }

    fn overflow(&self, cx: &mut impl Context) -> Vec2 {
        self.content.size() - cx.size()
    }

    fn should_show_scrollbar(&self, cx: &mut impl Context) -> bool {
        self.overflow(cx).max_element() > 1.0
    }

    fn handle_pointer_event(
        &self,
        state: &mut ScrollState,
        cx: &mut EventContext,
        event: &PointerEvent,
    ) -> bool {
        let mut handled = false;

        if event.scroll_delta != Vec2::ZERO && cx.hovered() {
            let overflow = self.overflow(cx);
            state.scroll -= event.scroll_delta / overflow * 10.0;
            state.scroll = state.scroll.clamp(Vec2::ZERO, Vec2::ONE);

            cx.request_redraw();

            handled = true;
        }

        if !self.should_show_scrollbar(cx) {
            return handled;
        }

        let scrollbar_rect = self.scrollbar_track_rect(cx);

        if scrollbar_rect.contains(event.position) && event.is_press() {
            cx.activate();
        }

        if event.is_release() {
            cx.deactivate();
        }

        if cx.active() {
            let start = scrollbar_rect.top();
            let end = scrollbar_rect.bottom();
            let range = end - start;

            let scroll = (event.position.y - start) / range;
            state.scroll.y = scroll.clamp(0.0, 1.0);

            cx.request_redraw();

            handled = true;
        }

        handled
    }
}

impl Parent for Scroll {
    fn add_child(&mut self, child: impl View) {
        self.content.add_child(child);
    }
}

#[derive(Default)]
pub struct ScrollState {
    scroll: Vec2,
}

impl View for Scroll {
    type State = ScrollState;

    fn build(&self) -> Self::State {
        ScrollState::default()
    }

    fn style(&self) -> Style {
        Style::new("scroll")
    }

    fn event(&self, state: &mut Self::State, cx: &mut EventContext, event: &Event) {
        if let Some(pointer_event) = event.get::<PointerEvent>() {
            if self.handle_pointer_event(state, cx, pointer_event) {
                event.handle();
            }
        }

        self.content.event(cx, event);
    }

    fn layout(&self, _state: &mut Self::State, cx: &mut LayoutContext, bc: BoxConstraints) -> Vec2 {
        let flex = FlexLayout::vertical();
        let size = self.content.flex_layout(cx, bc.loose_y(), flex);

        cx.style_constraints(bc).constrain(size)
    }

    fn draw(&self, state: &mut Self::State, cx: &mut DrawContext) {
        let overflow = self.overflow(cx);
        self.content.set_offset(-state.scroll * overflow);

        let container_rect = cx.rect();
        cx.layer().clip(container_rect).draw(|cx| {
            self.content.draw(cx);
        });

        if !self.should_show_scrollbar(cx) {
            return;
        }

        // draw scrollbar track
        let rect = self.scrollbar_track_rect(cx);

        let max_radius = rect.width() * 0.5;
        let radius = cx.style_range("scrollbar-border-radius", 0.0..max_radius);

        let quad = Quad {
            rect,
            background: cx.style("scrollbar-track-color"),
            border_radius: [radius; 4],
            border_width: cx.style_range("scrollbar-track-border-width", 0.0..max_radius),
            border_color: cx.style("scrollbar-track-border-color"),
        };

        cx.layer().depth(100.0).draw(|cx| {
            cx.draw(quad);
        });

        // draw scrollbar
        let rect = self.scrollbar_rect(state, cx);

        let quad = Quad {
            rect,
            background: cx.style("scrollbar-color"),
            border_radius: [radius; 4],
            border_width: cx.style_range("scrollbar-border-width", 0.0..max_radius),
            border_color: cx.style("scrollbar-border-color"),
        };

        cx.layer().depth(100.0).draw(|cx| {
            cx.draw(quad);
        });
    }
}
