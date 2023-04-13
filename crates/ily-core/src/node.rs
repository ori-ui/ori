use std::{any::Any, cell::RefCell};

use glam::Vec2;
use ily_graphics::{Frame, Rect, TextLayout};
use uuid::Uuid;

use crate::{
    AnyView, BoxConstraints, DrawContext, Event, EventContext, LayoutContext, PointerEvent, Style,
    StyleElement, StyleElements, StyleSelector, StyleStates, View, WeakCallback,
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

#[derive(Clone, Debug, Default)]
pub struct NodeState {
    pub id: NodeId,
    pub local_rect: Rect,
    pub global_rect: Rect,
    pub active: bool,
    pub focused: bool,
    pub hovered: bool,
}

impl NodeState {
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
}

/// A node in the [`View`] tree.
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

    pub fn set_offset(&self, offset: Vec2) {
        let mut state = self.node_state.borrow_mut();

        let size = state.local_rect.size();
        state.local_rect = Rect::min_size(offset, size);
    }

    pub fn states(&self) -> StyleStates {
        self.node_state.borrow().style_states()
    }

    pub fn selector(&self, ancestors: &StyleElements, states: StyleStates) -> StyleSelector {
        let mut elements = ancestors.clone();

        if let Some(element) = self.view.element() {
            elements.push(StyleElement::new(element, states));
        }

        StyleSelector {
            elements,
            classes: self.view.classes(),
        }
    }

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
            selector: &self.selector(&cx.selector.elements, node_state.style_states()),
            state: &mut node_state,
            request_redraw: cx.request_redraw,
        };

        let mut state = self.state.borrow_mut();
        self.view.event(&mut **state, &mut cx, event);
    }

    pub fn layout(&self, cx: &mut LayoutContext, bc: BoxConstraints) -> Vec2 {
        let mut cx = LayoutContext {
            style: cx.style,
            selector: &self.selector(&cx.selector.elements, self.states()),
            text_layout: cx.text_layout,
        };

        let mut view_state = self.state.borrow_mut();
        let size = self.view.layout(&mut **view_state, &mut cx, bc);

        let mut node_state = self.node_state.borrow_mut();
        node_state.local_rect = Rect::min_size(node_state.local_rect.min, size);
        node_state.global_rect = Rect::min_size(node_state.global_rect.min, size);

        size
    }

    pub fn draw(&self, cx: &mut DrawContext) {
        let mut node_state = self.node_state.borrow_mut();
        node_state.propagate_parent(cx.state);

        let selector = self.selector(&cx.selector.elements, node_state.style_states());
        let mut cx = DrawContext {
            style: cx.style,
            selector: &selector,
            frame: cx.frame,
            state: &mut node_state,
            request_redraw: cx.request_redraw,
        };

        let mut state = self.state.borrow_mut();
        self.view.draw(&mut **state, &mut cx);
    }

    pub fn event_root(&self, style: &Style, request_redraw: &WeakCallback, event: &Event) {
        let mut node_state = self.node_state.borrow_mut();

        if let Some(event) = event.get::<PointerEvent>() {
            if self.handle_pointer_event(&mut node_state, event) {
                request_redraw.emit(&());
            }
        }

        let selector = self.selector(&StyleElements::new(), node_state.style_states());
        let mut cx = EventContext {
            style,
            selector: &selector,
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
    ) -> Vec2 {
        let selector = self.selector(&StyleElements::new(), self.states());
        let mut cx = LayoutContext {
            style,
            selector: &selector,
            text_layout,
        };

        let bc = BoxConstraints::new(Vec2::ZERO, window_size);
        self.layout(&mut cx, bc)
    }

    pub fn draw_root(&self, style: &Style, frame: &mut Frame, request_redraw: &WeakCallback) {
        let mut node_state = self.node_state.borrow_mut();

        let selector = self.selector(&StyleElements::new(), node_state.style_states());
        let mut cx = DrawContext {
            style,
            selector: &selector,
            frame,
            state: &mut node_state,
            request_redraw,
        };

        let mut state = self.state.borrow_mut();
        self.view.draw(&mut **state, &mut cx);
    }
}
