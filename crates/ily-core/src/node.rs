use std::{any::Any, cell::RefCell, time::Instant};

use glam::Vec2;
use ily_graphics::{Frame, Rect, TextLayout};
use uuid::Uuid;

use crate::{
    AnyView, Attributes, BoxConstraints, DrawContext, Event, EventContext, LayoutContext,
    PointerEvent, SharedSignal, Style, StyleElement, StyleElements, StyleSelectors, StyleStates,
    Styleable, Transition, TransitionStates, View, WeakCallback,
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

#[derive(Debug)]
pub struct NodeState {
    pub id: NodeId,
    pub local_rect: Rect,
    pub global_rect: Rect,
    pub active: bool,
    pub focused: bool,
    pub hovered: bool,
    pub transitions: TransitionStates,
    pub last_draw: Instant,
    pub attributes: SharedSignal<Attributes>,
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
            transitions: TransitionStates::new(),
            last_draw: Instant::now(),
            attributes: SharedSignal::new(Attributes::new()),
        }
    }
}

impl NodeState {
    pub fn styled(attributes: Attributes) -> Self {
        Self::styled_signal(SharedSignal::new(attributes))
    }

    pub fn styled_signal(attributes: SharedSignal<Attributes>) -> Self {
        Self {
            attributes,
            ..Self::default()
        }
    }

    pub fn propagate_parent(&mut self, parent: &NodeState) {
        self.global_rect = self.local_rect.translate(parent.global_rect.min);
    }

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

    pub fn delta(&self) -> f32 {
        self.last_draw.elapsed().as_secs_f32()
    }

    pub fn transition<T: 'static>(
        &mut self,
        name: &str,
        mut value: T,
        transition: Option<Transition>,
    ) -> (T, bool) {
        let delta = self.delta();
        let redraw = (self.transitions).transition_any(name, &mut value, transition, delta);
        (value, redraw)
    }

    fn draw(&mut self) {
        self.last_draw = Instant::now();
    }
}

/// A node in the [`View`](crate::View) tree.
pub struct Node {
    state: RefCell<Box<dyn Any>>,
    node_state: RefCell<NodeState>,
    view: Box<dyn AnyView>,
}

impl Node {
    pub fn new(view: impl View) -> Self {
        Self {
            state: RefCell::new(Box::new(view.build())),
            node_state: RefCell::new(NodeState::default()),
            view: Box::new(view),
        }
    }

    /// Creates a new [`Node`] from a [`Styleable`] view.
    pub fn styled<T: View>(styleable: impl Styleable<T>) -> Self {
        let styled = styleable.styled();
        Self {
            state: RefCell::new(Box::new(styled.build())),
            node_state: RefCell::new(NodeState::styled(styled.attributes)),
            view: Box::new(styled.value),
        }
    }

    pub fn styled_signal(view: impl View, attributes: SharedSignal<Attributes>) -> Self {
        Self {
            state: RefCell::new(Box::new(view.build())),
            node_state: RefCell::new(NodeState::styled_signal(attributes)),
            view: Box::new(view),
        }
    }

    pub fn set_offset(&self, offset: Vec2) {
        let mut state = self.node_state.borrow_mut();

        let size = state.local_rect.size();
        state.local_rect = Rect::min_size(offset, size);
    }

    pub fn states(&self) -> StyleStates {
        self.node_state.borrow().style_states()
    }

    pub fn selectors(&self, ancestors: &StyleElements, states: StyleStates) -> StyleSelectors {
        let mut elements = ancestors.clone();

        if let Some(element) = self.view.element() {
            elements.push(StyleElement::new(Some(element.into()), states));
        }

        StyleSelectors {
            elements,
            classes: self.view.classes(),
        }
    }

    pub fn local_rect(&self) -> Rect {
        self.node_state.borrow().local_rect
    }

    pub fn size(&self) -> Vec2 {
        self.node_state.borrow().local_rect.size()
    }
}

impl Node {
    fn handle_pointer_event(&self, node_state: &mut NodeState, event: &PointerEvent) -> bool {
        let hovered = node_state.global_rect.contains(event.position);
        if hovered != node_state.hovered {
            node_state.hovered = hovered;
            true
        } else {
            false
        }
    }

    pub fn event(&self, cx: &mut EventContext, event: &Event) {
        let mut node_state = self.node_state.borrow_mut();
        node_state.propagate_parent(cx.state);

        if let Some(event) = event.get::<PointerEvent>() {
            if self.handle_pointer_event(&mut node_state, event) {
                cx.request_redraw();
            }
        }

        let mut cx = EventContext {
            style: cx.style,
            selectors: &self.selectors(&cx.selectors.elements, node_state.style_states()),
            state: &mut node_state,
            request_redraw: cx.request_redraw,
        };

        let mut state = self.state.borrow_mut();
        self.view.event(&mut **state, &mut cx, event);
    }

    pub fn layout(&self, cx: &mut LayoutContext, bc: BoxConstraints) -> Vec2 {
        let mut node_state = self.node_state.borrow_mut();
        let selectors = self.selectors(&cx.selectors.elements, node_state.style_states());
        let mut cx = LayoutContext {
            style: cx.style,
            state: &mut node_state,
            selectors: &selectors,
            text_layout: cx.text_layout,
            request_redraw: cx.request_redraw,
        };

        let mut view_state = self.state.borrow_mut();
        let size = self.view.layout(&mut **view_state, &mut cx, bc);

        node_state.local_rect = Rect::min_size(node_state.local_rect.min, size);
        node_state.global_rect = Rect::min_size(node_state.global_rect.min, size);

        size
    }

    pub fn draw(&self, cx: &mut DrawContext) {
        let mut node_state = self.node_state.borrow_mut();
        node_state.propagate_parent(cx.state);

        let selectors = self.selectors(&cx.selectors.elements, node_state.style_states());
        let mut cx = DrawContext {
            style: cx.style,
            selectors: &selectors,
            frame: cx.frame,
            state: &mut node_state,
            request_redraw: cx.request_redraw,
        };

        let mut state = self.state.borrow_mut();
        self.view.draw(&mut **state, &mut cx);

        cx.state.draw();
    }
}

impl Node {
    pub fn event_root(&self, style: &Style, request_redraw: &WeakCallback, event: &Event) {
        let mut node_state = self.node_state.borrow_mut();

        if let Some(event) = event.get::<PointerEvent>() {
            if self.handle_pointer_event(&mut node_state, event) {
                request_redraw.emit(&());
            }
        }

        let selectors = self.selectors(&StyleElements::new(), node_state.style_states());
        let mut cx = EventContext {
            style,
            selectors: &selectors,
            state: &mut node_state,
            request_redraw,
        };

        let mut state = self.state.borrow_mut();
        self.view.event(&mut **state, &mut cx, event);
    }

    pub fn layout_root(
        &self,
        style: &Style,
        text_layout: &mut dyn TextLayout,
        window_size: Vec2,
        request_redraw: &WeakCallback,
    ) -> Vec2 {
        let mut node_state = self.node_state.borrow_mut();
        let selectors = self.selectors(&StyleElements::new(), node_state.style_states());
        let mut cx = LayoutContext {
            style,
            state: &mut node_state,
            selectors: &selectors,
            text_layout,
            request_redraw,
        };

        let bc = BoxConstraints::new(Vec2::ZERO, window_size);
        let mut state = self.state.borrow_mut();
        let size = self.view.layout(&mut **state, &mut cx, bc);

        node_state.local_rect = Rect::min_size(node_state.local_rect.min, size);
        node_state.global_rect = Rect::min_size(node_state.global_rect.min, size);

        size
    }

    pub fn draw_root(&self, style: &Style, frame: &mut Frame, request_redraw: &WeakCallback) {
        let mut node_state = self.node_state.borrow_mut();

        let selectors = self.selectors(&StyleElements::new(), node_state.style_states());
        let mut cx = DrawContext {
            style,
            selectors: &selectors,
            frame,
            state: &mut node_state,
            request_redraw,
        };

        let mut state = self.state.borrow_mut();
        self.view.draw(&mut **state, &mut cx);

        cx.state.draw();
    }
}

impl<T: View> From<T> for Node {
    fn from(view: T) -> Self {
        Self::new(view)
    }
}
