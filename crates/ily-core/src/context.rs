use std::ops::{Deref, DerefMut, Range};

use glam::Vec2;
use ily_graphics::{Frame, Rect, TextHit, TextLayout, TextSection};

use crate::{Attribute, FromAttribute, NodeState, Style, StyleSelectors, Unit, WeakCallback};

pub struct EventContext<'a> {
    pub style: &'a Style,
    pub state: &'a mut NodeState,
    pub selectors: &'a StyleSelectors,
    pub request_redraw: &'a WeakCallback,
}

pub struct LayoutContext<'a> {
    pub style: &'a Style,
    pub state: &'a mut NodeState,
    pub selectors: &'a StyleSelectors,
    pub text_layout: &'a dyn TextLayout,
    pub request_redraw: &'a WeakCallback,
}

impl<'a> LayoutContext<'a> {
    pub fn text_bounds(&self, section: &TextSection) -> Option<Rect> {
        self.text_layout.bounds(section)
    }

    pub fn text_hit_test(&self, section: &TextSection, pos: Vec2) -> Option<TextHit> {
        self.text_layout.hit(section, pos)
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
    pub fn frame(&mut self) -> &mut Frame {
        self.frame
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

macro_rules! context {
    ($name:ident) => {
        impl<'a> $name<'a> {
            /// Get the value of a style attribute, if it has a transition, the value will be
            /// interpolated between the current value and the new value.
            pub fn style<T: FromAttribute + Default + 'static>(&mut self, key: &str) -> T {
                let result = self.style.get_value_and_transition(self.selectors, key);
                let (value, transition) = result.unwrap_or_default();
                let (value, redraw) = self.state.transition(key, value, transition);

                if redraw {
                    self.request_redraw();
                }

                value
            }

            /// Get the value of a style attribute, if it has a transition, the value will be
            /// interpolated between the current value and the new value.
            ///
            /// This is a convenience method for getting a value in pixels, as opposed to
            /// `style` which returns a `Unit`.
            pub fn style_range(&mut self, key: &str, range: Range<f32>) -> f32 {
                let result = self.style.get_value_and_transition(self.selectors, key);
                let (value, transition): (Unit, _) = result.unwrap_or_default();

                let pixels = value.pixels(range);
                let (pixels, redraw) = self.state.transition(key, pixels, transition);

                if redraw {
                    self.request_redraw();
                }

                pixels
            }

            pub fn style_attribute(&self, key: &str) -> Option<&Attribute> {
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
    };
}

context!(EventContext);
context!(LayoutContext);
context!(DrawContext);
