use std::{
    cell::{Cell, RefCell},
    ops::{Deref, DerefMut},
    sync::Arc,
};

use glam::Vec2;
use ily_graphics::{Frame, Rect};
use ily_reactive::{BoundedScope, Scope, SharedSignal, WeakCallback};

use crate::{BoxConstraints, Event, PointerPress};

pub struct PaintContext<'a> {
    pub frame: &'a mut Frame,
    pub request_redraw: WeakCallback,
    pub rect: Rect,
}

impl<'a> PaintContext<'a> {
    pub fn frame(&mut self) -> &mut Frame {
        self.frame
    }

    pub fn rect(&self) -> Rect {
        self.rect
    }

    pub fn child(&mut self, child: Rect, callback: impl FnOnce(&mut PaintContext)) {
        let mut child = PaintContext {
            frame: self.frame,
            request_redraw: self.request_redraw.clone(),
            rect: child,
        };

        callback(&mut child);
    }

    pub fn layer(&mut self, callback: impl FnOnce(&mut PaintContext)) {
        self.frame.layer(|frame| {
            let mut child = PaintContext {
                frame,
                request_redraw: self.request_redraw.clone(),
                rect: self.rect,
            };

            callback(&mut child);
        });
    }
}

impl<'a> Deref for PaintContext<'a> {
    type Target = Frame;

    fn deref(&self) -> &Self::Target {
        self.frame
    }
}

impl<'a> DerefMut for PaintContext<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.frame
    }
}

#[allow(unused_variables)]
pub trait View: 'static {
    fn classes(&self) -> Vec<String> {
        Vec::new()
    }

    fn event(&self, event: &Event) {}

    fn layout(&self, bc: BoxConstraints) -> Vec2 {
        bc.min
    }

    fn paint(&self, cx: &mut PaintContext) {}
}

impl View for () {}

impl<T: View> View for SharedSignal<T> {
    fn classes(&self) -> Vec<String> {
        Vec::new()
    }

    fn event(&self, event: &Event) {
        self.get().event(event)
    }

    fn layout(&self, bc: BoxConstraints) -> Vec2 {
        self.get().layout(bc)
    }

    fn paint(&self, cx: &mut PaintContext) {
        self.emitter().subscribe_weak(cx.request_redraw.clone());
        self.get().paint(cx)
    }
}

pub struct AnyView {
    view: SharedSignal<dyn View>,
    rect: Cell<Rect>,
}

impl AnyView {
    pub fn new<'a, V: View>(
        cx: Scope<'a>,
        mut f: impl FnMut(BoundedScope<'_, 'a>) -> V + 'a,
    ) -> Self {
        let signal = cx.alloc(RefCell::new(None::<SharedSignal<dyn View>>));

        cx.effect_scoped(move |cx| {
            let view = Arc::new(f(cx));

            if signal.borrow().is_some() {
                signal.borrow().as_ref().unwrap().set_arc(view);
            } else {
                *signal.borrow_mut() = Some(SharedSignal::new_arc(view));
            }
        });

        let view = signal.borrow().as_ref().unwrap().clone();

        Self {
            view,
            rect: Cell::new(Rect::ZERO),
        }
    }

    pub fn new_static(view: impl View) -> Self {
        Self {
            view: SharedSignal::new_arc(Arc::new(view)),
            rect: Cell::new(Rect::ZERO),
        }
    }

    pub fn rect(&self) -> Rect {
        self.rect.get()
    }

    pub fn set_rect(&self, rect: Rect) {
        self.rect.set(rect);
    }
}

impl View for AnyView {
    fn classes(&self) -> Vec<String> {
        self.view.get().classes()
    }

    fn event(&self, event: &Event) {
        if let Some(pointer_press) = event.get::<PointerPress>() {
            if !self.rect.get().contains(pointer_press.position) {
                return;
            }
        }

        self.view.get().event(event)
    }

    fn layout(&self, bc: BoxConstraints) -> Vec2 {
        self.view.get().layout(bc)
    }

    fn paint(&self, cx: &mut PaintContext) {
        (self.view.emitter()).subscribe_weak(cx.request_redraw.clone());

        cx.child(self.rect(), |cx| {
            self.view.get().paint(cx);
        });
    }
}
