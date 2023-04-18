use std::ops::{Deref, DerefMut, Range};

use glam::Vec2;
use ily_graphics::{Frame, Rect, TextHit, TextLayout, TextSection};

use crate::{
    FromStyleAttribute, NodeState, StyleAttribute, StyleSelectors, StyleTransition, Stylesheet,
    Unit, WeakCallback,
};

pub struct EventContext<'a> {
    pub style: &'a Stylesheet,
    pub state: &'a mut NodeState,
    pub root_font_size: f32,
    pub selectors: &'a StyleSelectors,
    pub request_redraw: &'a WeakCallback,
}

pub struct LayoutContext<'a> {
    pub style: &'a Stylesheet,
    pub state: &'a mut NodeState,
    pub root_font_size: f32,
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
    pub style: &'a Stylesheet,
    pub state: &'a mut NodeState,
    pub frame: &'a mut Frame,
    pub root_font_size: f32,
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
                root_font_size: self.root_font_size,
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
            fn get_style_value_and_transition<T: FromStyleAttribute + Default + 'static>(
                &self,
                key: &str,
            ) -> Option<(T, Option<StyleTransition>)> {
                if let Some(result) = self.state.style.attributes.get_value_and_transition(key) {
                    return Some(result);
                }

                if let Some(result) = self.style.get_value_and_transition(self.selectors, key) {
                    return Some(result);
                }

                None
            }

            /// Get the value of a style attribute, if it has a transition, the value will be
            /// interpolated between the current value and the new value.
            #[track_caller]
            pub fn style<T: FromStyleAttribute + Default + 'static>(&mut self, key: &str) -> T {
                let (value, transition) =
                    self.get_style_value_and_transition(key).unwrap_or_default();
                self.state.transition(key, value, transition)
            }

            /// Get the value of a style attribute, if it has a transition, the value will be
            /// interpolated between the current value and the new value.
            ///
            /// This is a convenience method for getting a value in pixels, as opposed to
            /// `style` which returns a `Unit`.
            #[track_caller]
            pub fn style_range(&mut self, key: &str, range: Range<f32>) -> f32 {
                let (value, transition) = self
                    .get_style_value_and_transition::<Unit>(key)
                    .unwrap_or_default();

                let pixels = value.pixels(range, self.root_font_size);
                self.state.transition(key, pixels, transition)
            }

            pub fn style_range_or(
                &mut self,
                primary: &str,
                secondary: &str,
                range: Range<f32>,
            ) -> f32 {
                if let Some((value, transition)) =
                    self.get_style_value_and_transition::<Unit>(primary)
                {
                    let pixels = value.pixels(range, self.root_font_size);
                    self.state.transition(primary, pixels, transition)
                } else if let Some((value, transition)) =
                    self.get_style_value_and_transition::<Unit>(secondary)
                {
                    let pixels = value.pixels(range, self.root_font_size);
                    self.state.transition(secondary, pixels, transition)
                } else {
                    0.0
                }
            }

            pub fn style_attribute(&self, key: &str) -> Option<&StyleAttribute> {
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

            #[track_caller]
            pub fn request_redraw(&self) {
                tracing::trace!("requesting redraw at {:?}", std::panic::Location::caller());

                self.request_redraw.emit(&());
            }
        }
    };
}

context!(EventContext);
context!(LayoutContext);
context!(DrawContext);
