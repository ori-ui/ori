use std::{
    any::{self, Any},
    ops::{Deref, DerefMut},
};

use glam::Vec2;
use ily_graphics::{Frame, Rect, TextLayout, TextSection};
use uuid::Uuid;

use crate::{BoxConstraints, Event, SharedSignal, Style, WeakCallback};

pub struct EventContext<'a> {
    pub style: &'a Style,
    pub state: &'a mut NodeState,
    pub request_redraw: &'a WeakCallback,
}

impl<'a> EventContext<'a> {
    pub fn request_redraw(&self) {
        self.request_redraw.emit(&());
    }

    pub fn active(&self) -> bool {
        self.state.active
    }

    pub fn hovered(&self) -> bool {
        self.state.hovered
    }

    pub fn focused(&self) -> bool {
        self.state.focused
    }

    pub fn local_rect(&self) -> Rect {
        self.state.local_rect
    }

    pub fn rect(&self) -> Rect {
        self.state.global_rect
    }
}

pub struct LayoutContext<'a> {
    pub style: &'a Style,
    pub text_layout: &'a dyn TextLayout,
}

impl<'a> LayoutContext<'a> {
    pub fn text_bounds(&self, section: &TextSection) -> Option<Rect> {
        self.text_layout.bounds(section)
    }
}

pub struct DrawContext<'a> {
    pub style: &'a Style,
    pub frame: &'a mut Frame,
    pub state: &'a mut NodeState,
    pub request_redraw: &'a WeakCallback,
}

impl<'a> DrawContext<'a> {
    pub fn request_redraw(&self) {
        self.request_redraw.emit(&());
    }

    pub fn active(&self) -> bool {
        self.state.active
    }

    pub fn hovered(&self) -> bool {
        self.state.hovered
    }

    pub fn focused(&self) -> bool {
        self.state.focused
    }

    pub fn frame(&mut self) -> &mut Frame {
        self.frame
    }

    pub fn local_rect(&self) -> Rect {
        self.state.local_rect
    }

    pub fn rect(&self) -> Rect {
        self.state.global_rect
    }

    pub fn layer(&mut self, callback: impl FnOnce(&mut DrawContext)) {
        self.frame.layer(|frame| {
            let mut child = DrawContext {
                style: self.style,
                frame,
                state: self.state,
                request_redraw: self.request_redraw,
            };

            callback(&mut child);
        });
    }
}

impl<'a> Deref for DrawContext<'a> {
    type Target = Frame;

    fn deref(&self) -> &Self::Target {
        self.frame
    }
}

impl<'a> DerefMut for DrawContext<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.frame
    }
}

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
}

#[allow(unused_variables)]
pub trait View: 'static {
    type State: 'static;

    fn build(&self) -> Self::State;

    fn element(&self) -> Option<&'static str> {
        None
    }

    fn classes(&self) -> Vec<String> {
        Vec::new()
    }

    fn event(&self, state: &mut Self::State, cx: &mut EventContext, event: &Event) {}

    fn layout(&self, state: &mut Self::State, cx: &mut LayoutContext, bc: BoxConstraints) -> Vec2 {
        bc.min
    }

    fn draw(&self, state: &mut Self::State, cx: &mut DrawContext) {}
}

pub trait AnyView {
    fn build(&self) -> Box<dyn Any>;

    fn element(&self) -> Option<&'static str>;

    fn classes(&self) -> Vec<String>;

    fn event(&self, state: &mut dyn Any, cx: &mut EventContext, event: &Event);

    fn layout(&self, state: &mut dyn Any, cx: &mut LayoutContext, bc: BoxConstraints) -> Vec2;

    fn draw(&self, state: &mut dyn Any, cx: &mut DrawContext);
}

impl<T: View> AnyView for T {
    fn build(&self) -> Box<dyn Any> {
        Box::new(self.build())
    }

    fn element(&self) -> Option<&'static str> {
        self.element()
    }

    fn classes(&self) -> Vec<String> {
        self.classes()
    }

    fn event(&self, state: &mut dyn Any, cx: &mut EventContext, event: &Event) {
        if let Some(state) = state.downcast_mut::<T::State>() {
            self.event(state, cx, event);
        } else {
            tracing::warn!("invalid state type on {}", any::type_name::<T>());
        }
    }

    fn layout(&self, state: &mut dyn Any, cx: &mut LayoutContext, bc: BoxConstraints) -> Vec2 {
        if let Some(state) = state.downcast_mut::<T::State>() {
            self.layout(state, cx, bc)
        } else {
            tracing::warn!("invalid state type on {}", any::type_name::<T>());
            bc.min
        }
    }

    fn draw(&self, state: &mut dyn Any, cx: &mut DrawContext) {
        if let Some(state) = state.downcast_mut::<T::State>() {
            self.draw(state, cx);
        } else {
            tracing::warn!("invalid state type on {}", any::type_name::<T>());
        }
    }
}

pub trait Parent {
    fn add_child(&mut self, child: impl View);
}

pub trait Properties {
    type Setter<'a>
    where
        Self: 'a;

    fn setter(&mut self) -> Self::Setter<'_>;
}

pub trait Events {
    type Setter<'a>
    where
        Self: 'a;

    fn setter(&mut self) -> Self::Setter<'_>;
}

/// When a view is wrapped in a signal, the view will be redrawn when the signal
/// changes.
impl<V: View> View for SharedSignal<V> {
    type State = V::State;

    fn build(&self) -> Self::State {
        self.get_untracked().build()
    }

    fn element(&self) -> Option<&'static str> {
        self.get().element()
    }

    fn classes(&self) -> Vec<String> {
        self.get().classes()
    }

    fn event(&self, state: &mut Self::State, cx: &mut EventContext, event: &Event) {
        self.get().event(state, cx, event);
    }

    fn layout(&self, state: &mut Self::State, cx: &mut LayoutContext, bc: BoxConstraints) -> Vec2 {
        self.get().layout(state, cx, bc)
    }

    fn draw(&self, state: &mut Self::State, cx: &mut DrawContext) {
        // redraw when the signal changes
        self.emitter().subscribe_weak(cx.request_redraw.clone());
        self.get().draw(state, cx);
    }
}
