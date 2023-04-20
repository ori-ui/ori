use std::{any::Any, time::Instant};

use glam::Vec2;
use ily_graphics::{Frame, Rect, Renderer};
use uuid::Uuid;

use crate::{
    AnyView, BoxConstraints, Context, DrawContext, Event, EventContext, EventSink, LayoutContext,
    Lock, Lockable, PointerEvent, RequestRedrawEvent, Style, StyleSelector, StyleSelectors,
    StyleStates, StyleTransition, Stylesheet, TransitionStates, View,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NodeId {
    uuid: Uuid,
}

impl NodeId {
    pub fn new() -> Self {
        Self {
            uuid: Uuid::new_v4(),
        }
    }
}

impl Default for NodeId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
pub struct NodeState {
    pub id: NodeId,
    pub local_rect: Rect,
    pub global_rect: Rect,
    pub active: bool,
    pub focused: bool,
    pub hovered: bool,
    pub last_draw: Instant,
    pub style: Style,
    pub transitions: TransitionStates,
}

impl Default for NodeState {
    fn default() -> Self {
        Self {
            id: NodeId::new(),
            local_rect: Rect::ZERO,
            global_rect: Rect::ZERO,
            active: false,
            focused: false,
            hovered: false,
            last_draw: Instant::now(),
            style: Style::default(),
            transitions: TransitionStates::new(),
        }
    }
}

impl NodeState {
    pub fn new(style: Style) -> Self {
        Self {
            style,
            ..Default::default()
        }
    }

    pub fn propagate_parent(&mut self, parent: &NodeState) {
        self.global_rect = self.local_rect.translate(parent.global_rect.min);
    }

    pub fn propagate_child(&mut self, _child: &NodeState) {}

    pub fn style_states(&self) -> StyleStates {
        let mut states = StyleStates::new();

        if self.active {
            states.push("active");
        }

        if self.focused {
            states.push("focus");
        }

        if self.hovered {
            states.push("hover");
        }

        states
    }

    pub fn selector(&self) -> StyleSelector {
        StyleSelector {
            element: self.style.element.map(Into::into),
            classes: self.style.classes.clone(),
            states: self.style_states(),
        }
    }

    pub fn delta(&self) -> f32 {
        self.last_draw.elapsed().as_secs_f32()
    }

    pub fn transition<T: 'static>(
        &mut self,
        name: &str,
        mut value: T,
        transition: Option<StyleTransition>,
    ) -> T {
        (self.transitions).transition_any(name, &mut value, transition);
        value
    }

    pub fn update_transitions(&mut self) -> bool {
        self.transitions.update(self.delta())
    }

    fn draw(&mut self) {
        self.last_draw = Instant::now();
    }
}

impl<T: View> From<T> for Node {
    fn from(view: T) -> Self {
        Self::new(view)
    }
}

/// A node in the [`View`](crate::View) tree.
pub struct Node {
    #[cfg(feature = "multithread")]
    view_state: Lock<Box<dyn Any + Send + Sync>>,
    #[cfg(not(feature = "multithread"))]
    view_state: Lock<Box<dyn Any>>,
    node_state: Lock<NodeState>,
    view: Box<dyn AnyView>,
}

impl Node {
    pub fn new(view: impl View) -> Self {
        let view_state = Box::new(View::build(&view));
        let node_state = NodeState::new(View::style(&view));

        Self {
            view_state: Lock::new(view_state),
            node_state: Lock::new(node_state),
            view: Box::new(view),
        }
    }

    pub fn set_offset(&self, offset: Vec2) {
        let mut node_state = self.node_state.lock_mut();

        let size = node_state.local_rect.size();
        node_state.local_rect = Rect::min_size(offset, size);
    }

    pub fn states(&self) -> StyleStates {
        self.node_state.lock_mut().style_states()
    }

    pub fn local_rect(&self) -> Rect {
        self.node_state.lock_mut().local_rect
    }

    pub fn global_rect(&self) -> Rect {
        self.node_state.lock_mut().global_rect
    }

    pub fn size(&self) -> Vec2 {
        self.node_state.lock_mut().local_rect.size()
    }
}

impl Node {
    fn handle_pointer_event(&self, node_state: &mut NodeState, event: &PointerEvent) -> bool {
        let hovered = node_state.global_rect.contains(event.position) && !event.left;
        if hovered != node_state.hovered {
            node_state.hovered = hovered;
            true
        } else {
            false
        }
    }

    pub fn event(&self, cx: &mut EventContext, event: &Event) {
        let mut node_state = self.node_state.lock_mut();
        node_state.style = self.view.style();
        node_state.propagate_parent(cx.state);

        if let Some(event) = event.get::<PointerEvent>() {
            if self.handle_pointer_event(&mut node_state, event) {
                cx.request_redraw();
            }
        }

        {
            let selector = node_state.selector();
            let mut cx = EventContext {
                style: cx.style,
                state: &mut node_state,
                renderer: cx.renderer,
                selectors: &cx.selectors.clone().with(selector),
                event_sink: cx.event_sink,
            };

            let mut view_state = self.view_state.lock_mut();
            self.view.event(&mut **view_state, &mut cx, event);
        }

        cx.state.propagate_child(&node_state);
    }

    pub fn layout(&self, cx: &mut LayoutContext, bc: BoxConstraints) -> Vec2 {
        let mut node_state = self.node_state.lock_mut();
        node_state.style = self.view.style();

        let size = {
            let selector = node_state.selector();
            let mut cx = LayoutContext {
                style: cx.style,
                state: &mut node_state,
                renderer: cx.renderer,
                selectors: &cx.selectors.clone().with(selector),
                event_sink: cx.event_sink,
            };

            let mut view_state = self.view_state.lock_mut();
            self.view.layout(&mut **view_state, &mut cx, bc)
        };

        node_state.local_rect = Rect::min_size(node_state.local_rect.min, size);
        node_state.global_rect = Rect::min_size(node_state.global_rect.min, size);

        cx.state.propagate_child(&node_state);

        size
    }

    pub fn draw(&self, cx: &mut DrawContext) {
        let mut node_state = self.node_state.lock_mut();
        node_state.style = self.view.style();
        node_state.propagate_parent(cx.state);

        {
            let selector = node_state.selector();
            let mut cx = DrawContext {
                style: cx.style,
                state: &mut node_state,
                frame: cx.frame,
                renderer: cx.renderer,
                selectors: &cx.selectors.clone().with(selector),
                event_sink: cx.event_sink,
            };

            let mut view_state = self.view_state.lock_mut();
            self.view.draw(&mut **view_state, &mut cx);

            if cx.state.update_transitions() {
                cx.request_redraw();
            }

            cx.state.draw();
        }

        cx.state.propagate_child(&node_state);
    }
}

impl Node {
    pub fn event_root(
        &self,
        style: &Stylesheet,
        renderer: &dyn Renderer,
        event_sink: &EventSink,
        event: &Event,
    ) {
        let mut node_state = self.node_state.lock_mut();
        node_state.style = self.view.style();

        if let Some(event) = event.get::<PointerEvent>() {
            if self.handle_pointer_event(&mut node_state, event) {
                event_sink.send(RequestRedrawEvent);
            }
        }

        let selector = node_state.selector();
        let mut cx = EventContext {
            style,
            state: &mut node_state,
            renderer,
            selectors: &StyleSelectors::new().with(selector),
            event_sink,
        };

        let mut view_state = self.view_state.lock_mut();
        self.view.event(&mut **view_state, &mut cx, event);
    }

    pub fn layout_root(
        &self,
        style: &Stylesheet,
        renderer: &dyn Renderer,
        window_size: Vec2,
        event_sink: &EventSink,
    ) -> Vec2 {
        let mut node_state = self.node_state.lock_mut();
        node_state.style = self.view.style();

        let selector = node_state.selector();
        let mut cx = LayoutContext {
            style,
            state: &mut node_state,
            renderer,
            selectors: &StyleSelectors::new().with(selector),
            event_sink,
        };

        let bc = BoxConstraints::new(Vec2::ZERO, window_size);
        let mut view_state = self.view_state.lock_mut();
        let size = self.view.layout(&mut **view_state, &mut cx, bc);

        node_state.local_rect = Rect::min_size(node_state.local_rect.min, size);
        node_state.global_rect = Rect::min_size(node_state.global_rect.min, size);

        size
    }

    pub fn draw_root(
        &self,
        style: &Stylesheet,
        frame: &mut Frame,
        renderer: &dyn Renderer,
        event_sink: &EventSink,
    ) {
        let mut node_state = self.node_state.lock_mut();
        node_state.style = self.view.style();

        let selector = node_state.selector();
        let mut cx = DrawContext {
            style,
            state: &mut node_state,
            frame,
            renderer,
            selectors: &StyleSelectors::new().with(selector),
            event_sink,
        };

        let mut view_state = self.view_state.lock_mut();
        self.view.draw(&mut **view_state, &mut cx);

        cx.state.draw();
    }
}
