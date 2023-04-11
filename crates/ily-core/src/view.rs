use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
};

use glam::Vec2;
use ily_graphics::{Frame, Rect, TextLayout, TextSection};
use uuid::Uuid;

use crate::{BoxConstraints, Event, PointerEvent, SharedSignal, WeakCallback};

pub struct EventContext<'a> {
    pub state: &'a mut ViewState,
    pub request_redraw: &'a WeakCallback,
}

pub struct LayoutContext<'a> {
    pub text_layout: &'a dyn TextLayout,
}

impl<'a> LayoutContext<'a> {
    pub fn text_bounds(&self, section: &TextSection) -> Option<Rect> {
        self.text_layout.bounds(section)
    }
}

pub struct DrawContext<'a> {
    pub frame: &'a mut Frame,
    pub state: &'a mut ViewState,
    pub request_redraw: &'a WeakCallback,
}

impl<'a> DrawContext<'a> {
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

    pub fn rect(&self) -> Rect {
        self.state.global_rect
    }

    pub fn layer(&mut self, callback: impl FnOnce(&mut DrawContext)) {
        self.frame.layer(|frame| {
            let mut child = DrawContext {
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
pub struct ViewId {
    uuid: Uuid,
}

impl ViewId {
    pub fn new() -> Self {
        Self {
            uuid: Uuid::new_v4(),
        }
    }
}

impl Default for ViewId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Default)]
pub struct ViewState {
    pub id: ViewId,
    pub local_rect: Rect,
    pub global_rect: Rect,
    pub active: bool,
    pub focused: bool,
    pub hovered: bool,
}

impl ViewState {
    pub fn propagate_parent(&mut self, parent: &ViewState) {
        self.global_rect = self.local_rect.translate(parent.global_rect.min);
    }
}

#[allow(unused_variables)]
pub trait View: 'static {
    fn build(&self) -> ViewState {
        ViewState::default()
    }

    fn element(&self) -> Option<&'static str> {
        None
    }

    fn event(&self, cx: &mut EventContext, event: &Event) {}

    fn layout(&self, cx: &mut LayoutContext, bc: BoxConstraints) -> Vec2 {
        bc.min
    }

    fn draw(&self, cx: &mut DrawContext) {}
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
    fn build(&self) -> ViewState {
        self.get_untracked().build()
    }

    fn element(&self) -> Option<&'static str> {
        self.get().element()
    }

    fn event(&self, cx: &mut EventContext, event: &Event) {
        self.get().event(cx, event)
    }

    fn layout(&self, cx: &mut LayoutContext, bc: BoxConstraints) -> Vec2 {
        self.get().layout(cx, bc)
    }

    fn draw(&self, cx: &mut DrawContext) {
        self.emitter().subscribe_weak(cx.request_redraw.clone());
        self.get().draw(cx)
    }
}

pub struct Child {
    pub view: Box<dyn View>,
    pub state: RefCell<ViewState>,
}

impl Child {
    pub fn new(view: impl View) -> Self {
        Self {
            state: RefCell::new(view.build()),
            view: Box::new(view),
        }
    }

    pub fn set_offset(&self, offset: Vec2) {
        let mut state = self.state.borrow_mut();

        let size = state.local_rect.size();
        state.local_rect = Rect::min_size(offset, size);
    }

    pub fn event(&self, event: &Event, request_redraw: &WeakCallback) {
        let mut state = self.state.borrow_mut();

        if let Some(event) = event.get::<PointerEvent>() {
            let hovered = state.global_rect.contains(event.position);
            if hovered != state.hovered {
                state.hovered = hovered;
                request_redraw.emit();
            }
        }

        let mut cx = EventContext {
            state: &mut state,
            request_redraw,
        };

        self.view.event(&mut cx, event);
    }

    pub fn layout(&self, text_layout: &dyn TextLayout, bc: BoxConstraints) -> Vec2 {
        let mut cx = LayoutContext { text_layout };

        let size = self.view.layout(&mut cx, bc);

        let mut state = self.state.borrow_mut();
        state.local_rect = Rect::min_size(state.local_rect.min, size);
        state.global_rect = Rect::min_size(state.global_rect.min, size);

        size
    }

    pub fn draw(&self, frame: &mut Frame, request_redraw: &WeakCallback) {
        let mut cx = DrawContext {
            frame,
            state: &mut self.state.borrow_mut(),
            request_redraw,
        };

        self.view.draw(&mut cx);
    }
}

impl View for Child {
    fn build(&self) -> ViewState {
        self.view.build()
    }

    fn element(&self) -> Option<&'static str> {
        None
    }

    fn event(&self, cx: &mut EventContext, event: &Event) {
        self.state.borrow_mut().propagate_parent(cx.state);

        self.event(event, cx.request_redraw);
    }

    fn layout(&self, cx: &mut LayoutContext, bc: BoxConstraints) -> Vec2 {
        self.layout(cx.text_layout, bc)
    }

    fn draw(&self, cx: &mut DrawContext) {
        let mut state = self.state.borrow_mut();
        state.propagate_parent(cx.state);

        let mut cx = DrawContext {
            frame: cx.frame,
            state: &mut state,
            request_redraw: cx.request_redraw,
        };

        self.view.draw(&mut cx);
    }
}
