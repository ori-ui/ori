use std::{any::Any, cell::RefCell};

use glam::Vec2;
use ily_graphics::Rect;

use crate::{
    AnyView, BoxConstraints, DrawContext, Event, EventContext, LayoutContext, NodeState,
    PointerEvent, View,
};

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

    pub fn event(&self, cx: &mut EventContext, event: &Event) {
        let mut node_state = self.node_state.borrow_mut();
        node_state.propagate_parent(cx.state);

        if let Some(event) = event.get::<PointerEvent>() {
            let hovered = node_state.global_rect.contains(event.position);
            if hovered != node_state.hovered {
                node_state.hovered = hovered;
                cx.request_redraw();
            }
        }

        let mut cx = EventContext {
            style: cx.style,
            state: &mut node_state,
            request_redraw: cx.request_redraw,
        };

        let mut state = self.state.borrow_mut();
        self.view.event(&mut **state, &mut cx, event);
    }

    pub fn layout(&self, cx: &mut LayoutContext, bc: BoxConstraints) -> Vec2 {
        let mut cx = LayoutContext {
            style: cx.style,
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

        let mut cx = DrawContext {
            style: cx.style,
            frame: cx.frame,
            state: &mut node_state,
            request_redraw: cx.request_redraw,
        };

        let mut state = self.state.borrow_mut();
        self.view.draw(&mut **state, &mut cx);
    }
}
