use std::ops::{Deref, DerefMut, Range};

use glam::Vec2;
use ily_graphics::{Frame, Quad, Rect, Renderer, TextHit, TextSection};

use crate::{
    FromStyleAttribute, NodeState, StyleAttribute, StyleSelectors, StyleTransition, Stylesheet,
    Unit, WeakCallback,
};

pub struct EventContext<'a> {
    pub style: &'a Stylesheet,
    pub state: &'a mut NodeState,
    pub renderer: &'a dyn Renderer,
    pub selectors: &'a StyleSelectors,
    pub request_redraw: &'a WeakCallback,
}

pub struct LayoutContext<'a> {
    pub style: &'a Stylesheet,
    pub state: &'a mut NodeState,
    pub renderer: &'a dyn Renderer,
    pub selectors: &'a StyleSelectors,
    pub request_redraw: &'a WeakCallback,
}

impl<'a> LayoutContext<'a> {
    pub fn messure_text(&self, section: &TextSection) -> Option<Rect> {
        self.renderer.messure_text(section)
    }

    pub fn hit_text(&self, section: &TextSection, pos: Vec2) -> Option<TextHit> {
        self.renderer.hit_text(section, pos)
    }
}

pub struct DrawContext<'a> {
    pub style: &'a Stylesheet,
    pub state: &'a mut NodeState,
    pub frame: &'a mut Frame,
    pub renderer: &'a dyn Renderer,
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
                state: self.state,
                frame,
                renderer: self.renderer,
                selectors: self.selectors,
                request_redraw: self.request_redraw,
            };

            callback(&mut child);
        });
    }

    /// Draws the quad at the current layout rect.
    ///
    /// This will use the following style attributes:
    /// - `background-color`: The background color of the quad.
    /// - `border-radius`: The border radius of the quad overwritten by the more specific
    /// attributes.
    /// - `border-top-left-radius`: The top left border radius of the quad.
    /// - `border-top-right-radius`: The top right border radius of the quad.
    /// - `border-bottom-right-radius`: The bottom right border radius of the quad.
    /// - `border-bottom-left-radius`: The bottom left border radius of the quad.
    /// - `border-width`: The border width of the quad.
    pub fn draw_quad(&mut self) {
        let range = 0.0..self.rect().max.min_element() / 2.0;

        let tl = self.style_range_or("border-top-left-radius", "border-radius", range.clone());
        let tr = self.style_range_or("border-top-right-radius", "border-radius", range.clone());
        let br = self.style_range_or("border-bottom-right-radius", "border-radius", range.clone());
        let bl = self.style_range_or("border-bottom-left-radius", "border-radius", range.clone());

        let quad = Quad {
            rect: self.rect(),
            background: self.style("background-color"),
            border_radius: [tl, tr, br, bl],
            border_width: self.style_range("border-width", range),
            border_color: self.style("border-color"),
        };

        self.draw_primitive(quad);
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

pub trait Context {
    fn stylesheet(&self) -> &Stylesheet;
    fn state(&self) -> &NodeState;
    fn state_mut(&mut self) -> &mut NodeState;
    fn renderer(&self) -> &dyn Renderer;
    fn selectors(&self) -> &StyleSelectors;
    fn request_redraw_callback(&self) -> &WeakCallback;

    fn get_style_value_and_transition<T: FromStyleAttribute + Default + 'static>(
        &self,
        key: &str,
    ) -> Option<(T, Option<StyleTransition>)> {
        if let Some(result) = self.state().style.attributes.get_value_and_transition(key) {
            return Some(result);
        }

        let style = self.stylesheet();

        if let Some(result) = style.get_value_and_transition(self.selectors(), key) {
            return Some(result);
        }

        None
    }

    /// Get the value of a style attribute, if it has a transition, the value will be
    /// interpolated between the current value and the new value.
    #[track_caller]
    fn style<T: FromStyleAttribute + Default + 'static>(&mut self, key: &str) -> T {
        let (value, transition) = self.get_style_value_and_transition(key).unwrap_or_default();
        self.state_mut().transition(key, value, transition)
    }

    fn style_or<T: FromStyleAttribute + Default + 'static>(
        &mut self,
        primary: &str,
        secondary: &str,
    ) -> T {
        let (value, transition) = self
            .get_style_value_and_transition(primary)
            .or_else(|| self.get_style_value_and_transition(secondary))
            .unwrap_or_default();

        self.state_mut().transition(primary, value, transition)
    }

    /// Get the value of a style attribute, if it has a transition, the value will be
    /// interpolated between the current value and the new value.
    ///
    /// This is a convenience method for getting a value in pixels, as opposed to
    /// `style` which returns a `Unit`.
    #[track_caller]
    fn style_range(&mut self, key: &str, range: Range<f32>) -> f32 {
        let (value, transition) = self
            .get_style_value_and_transition::<Unit>(key)
            .unwrap_or_default();

        let pixels = value.pixels(range, self.renderer().scale());
        self.state_mut().transition(key, pixels, transition)
    }

    fn style_range_or(&mut self, primary: &str, secondary: &str, range: Range<f32>) -> f32 {
        let (value, transition) = self
            .get_style_value_and_transition::<Unit>(primary)
            .or_else(|| self.get_style_value_and_transition::<Unit>(secondary))
            .unwrap_or_default();

        let pixels = value.pixels(range, self.renderer().scale());
        self.state_mut().transition(primary, pixels, transition)
    }

    fn style_attribute(&self, key: &str) -> Option<&StyleAttribute> {
        self.state().style.attributes.get(key)
    }

    fn downcast_renderer<T: Renderer>(&self) -> Option<&T> {
        self.renderer().downcast_ref()
    }

    fn active(&self) -> bool {
        self.state().active
    }

    fn hovered(&self) -> bool {
        self.state().hovered
    }

    fn focused(&self) -> bool {
        self.state().focused
    }

    fn focus(&mut self) {
        self.state_mut().focused = true;
        self.request_redraw();
    }

    fn unfocus(&mut self) {
        self.state_mut().focused = false;
        self.request_redraw();
    }

    fn local_rect(&self) -> Rect {
        self.state().local_rect
    }

    fn rect(&self) -> Rect {
        self.state().global_rect
    }

    fn request_redraw(&self) {
        self.request_redraw_callback().emit(&());
    }

    fn delta_time(&self) -> f32 {
        self.state().delta()
    }
}

macro_rules! context {
    ($name:ident) => {
        impl<'a> Context for $name<'a> {
            fn stylesheet(&self) -> &Stylesheet {
                self.style
            }

            fn state(&self) -> &NodeState {
                self.state
            }

            fn state_mut(&mut self) -> &mut NodeState {
                self.state
            }

            fn renderer(&self) -> &dyn Renderer {
                self.renderer
            }

            fn selectors(&self) -> &StyleSelectors {
                &self.selectors
            }

            fn request_redraw_callback(&self) -> &WeakCallback {
                &self.request_redraw
            }
        }
    };
}

context!(EventContext);
context!(LayoutContext);
context!(DrawContext);
