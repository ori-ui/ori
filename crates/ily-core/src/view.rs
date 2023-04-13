use std::{
    any::{self, Any},
    ops::{Deref, DerefMut},
};

use glam::Vec2;
use ily_graphics::{Frame, Rect, TextLayout, TextSection};

use crate::{
    AttributeValue, BoxConstraints, Event, NodeState, SharedSignal, Style, StyleClasses,
    StyleSelectors, WeakCallback,
};

pub struct EventContext<'a> {
    pub style: &'a Style,
    pub state: &'a mut NodeState,
    pub selectors: &'a StyleSelectors,
    pub request_redraw: &'a WeakCallback,
}

impl<'a> EventContext<'a> {
    pub fn style_value<T>(&self, key: &str) -> Option<T>
    where
        Option<T>: From<AttributeValue>,
    {
        self.style.get_value(self.selectors, key)
    }

    pub fn style_attribute(&self, key: &str) -> Option<&AttributeValue> {
        self.style.get_attribute(self.selectors, key)
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

    pub fn request_redraw(&self) {
        self.request_redraw.emit(&());
    }
}

pub struct LayoutContext<'a> {
    pub style: &'a Style,
    pub state: &'a mut NodeState,
    pub selector: &'a StyleSelectors,
    pub text_layout: &'a dyn TextLayout,
    pub request_redraw: &'a WeakCallback,
}

impl<'a> LayoutContext<'a> {
    pub fn style_value<T>(&self, key: &str) -> Option<T>
    where
        Option<T>: From<AttributeValue>,
    {
        self.style.get_value(self.selector, key)
    }

    pub fn style_attribute(&self, key: &str) -> Option<&AttributeValue> {
        self.style.get_attribute(self.selector, key)
    }

    pub fn text_bounds(&self, section: &TextSection) -> Option<Rect> {
        self.text_layout.bounds(section)
    }

    pub fn request_redraw(&self) {
        self.request_redraw.emit(&());
    }
}

pub struct DrawContext<'a> {
    pub style: &'a Style,
    pub state: &'a mut NodeState,
    pub frame: &'a mut Frame,
    pub selectors: &'a StyleSelectors,
    pub request_redraw: &'a WeakCallback,
}

impl<'a> DrawContext<'a> {
    pub fn style_value<T>(&self, key: &str) -> Option<T>
    where
        Option<T>: From<AttributeValue>,
    {
        self.style.get_value(self.selectors, key)
    }

    pub fn style_attribute(&self, key: &str) -> Option<&AttributeValue> {
        self.style.get_attribute(self.selectors, key)
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

    pub fn request_redraw(&self) {
        self.request_redraw.emit(&());
    }

    pub fn layer(&mut self, callback: impl FnOnce(&mut DrawContext)) {
        self.frame.layer(|frame| {
            let mut child = DrawContext {
                style: self.style,
                selectors: self.selectors,
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

/// A [`View`] is a component that can be rendered to the screen.
#[allow(unused_variables)]
pub trait View: 'static {
    /// The state of the view.
    type State: 'static;

    /// Builds the state of the view.
    fn build(&self) -> Self::State;

    /// Returns the element name of the view.
    fn element(&self) -> Option<&'static str> {
        None
    }

    /// Returns the classes of the view.
    fn classes(&self) -> StyleClasses {
        StyleClasses::new()
    }

    /// Handles an event.
    fn event(&self, state: &mut Self::State, cx: &mut EventContext, event: &Event) {}

    /// Handle layout and returns the size of the view.
    ///
    /// This method should return a size that fits the [`BoxConstraints`].
    ///
    /// The default implementation returns the minimum size.
    fn layout(&self, state: &mut Self::State, cx: &mut LayoutContext, bc: BoxConstraints) -> Vec2 {
        bc.min
    }

    /// Draws the view.
    fn draw(&self, state: &mut Self::State, cx: &mut DrawContext) {}
}

/// A [`View`] that with an unknown state.
///
/// This is used to store a [`View`] in a [`Node`].
pub trait AnyView {
    fn build(&self) -> Box<dyn Any>;

    fn element(&self) -> Option<&'static str>;

    fn classes(&self) -> StyleClasses;

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

    fn classes(&self) -> StyleClasses {
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

    fn classes(&self) -> StyleClasses {
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
