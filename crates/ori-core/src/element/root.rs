use glam::Vec2;
use ori_graphics::{cosmic_text::FontSystem, Frame, ImageCache, Rect, Renderer};
use ori_reactive::{Event, EventSink};
use ori_style::{StyleCache, StyleTree, Stylesheet};

use crate::{
    AvailableSpace, DebugEvent, DrawContext, Element, ElementView, EventContext, LayoutContext,
    Margin, Padding, PointerEvent, RequestRedrawEvent, Window, WindowResizedEvent,
};

impl<T: ElementView> Element<T> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn event_root_inner(
        &self,
        stylesheet: &Stylesheet,
        style_cache: &mut StyleCache,
        renderer: &dyn Renderer,
        window: &mut Window,
        font_system: &mut FontSystem,
        event_sink: &EventSink,
        event: &Event,
        image_cache: &mut ImageCache,
    ) {
        let element_state = &mut self.element_state();
        element_state.style = self.view().style();

        if let Some(pointer_event) = event.get::<PointerEvent>() {
            if Self::handle_pointer_event(element_state, pointer_event, event.is_handled()) {
                event_sink.emit(RequestRedrawEvent);
            }
        }

        if event.is::<WindowResizedEvent>() {
            element_state.needs_layout = true;
        }

        let mut style_tree = StyleTree::new(element_state.selector());
        let mut cx = EventContext {
            state: element_state,
            renderer,
            window,
            font_system,
            stylesheet,
            style_tree: &mut style_tree,
            event_sink,
            style_cache,
            image_cache,
        };

        if let Some(event) = event.get::<DebugEvent>() {
            event.set_element(&mut cx, self);
        }

        self.view().event(&mut self.view_state(), &mut cx, event);
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn layout_root_inner(
        &self,
        stylesheet: &Stylesheet,
        style_cache: &mut StyleCache,
        renderer: &dyn Renderer,
        window: &mut Window,
        font_system: &mut FontSystem,
        event_sink: &EventSink,
        image_cache: &mut ImageCache,
    ) -> Vec2 {
        let element_state = &mut self.element_state();
        element_state.style = self.view().style();
        element_state.needs_layout = false;

        let space = AvailableSpace::new(Vec2::ZERO, window.size.as_vec2());

        let mut style_tree = StyleTree::new(element_state.selector());
        let mut cx = LayoutContext {
            state: element_state,
            renderer,
            window,
            font_system,
            stylesheet,
            style_tree: &mut style_tree,
            event_sink,
            style_cache,
            image_cache,
            parent_space: space,
            space,
        };

        cx.state.margin = Margin::from_style(&mut cx, space);
        cx.state.padding = Padding::from_style(&mut cx, space);

        let space = cx.style_constraints(space);
        cx.space = space;

        let size = self.view().layout(&mut self.view_state(), &mut cx, space);

        element_state.available_space = space;
        element_state.local_rect = Rect::min_size(element_state.local_rect.min, size);
        element_state.global_rect = Rect::min_size(element_state.global_rect.min, size);

        size
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn draw_root_inner(
        &self,
        stylesheet: &Stylesheet,
        style_cache: &mut StyleCache,
        frame: &mut Frame,
        renderer: &dyn Renderer,
        window: &mut Window,
        font_system: &mut FontSystem,
        event_sink: &EventSink,
        image_cache: &mut ImageCache,
    ) {
        let element_state = &mut self.element_state();
        element_state.style = self.view().style();

        let mut style_tree = StyleTree::new(element_state.selector());
        let mut cx = DrawContext {
            state: element_state,
            frame,
            renderer,
            window,
            font_system,
            stylesheet,
            style_tree: &mut style_tree,
            event_sink,
            style_cache,
            image_cache,
        };

        self.view().draw(&mut self.view_state(), &mut cx);

        cx.state.draw();
    }
}
