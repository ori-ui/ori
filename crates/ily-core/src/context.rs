use std::{
    any::Any,
    collections::HashMap,
    ops::{Deref, DerefMut, Range},
};

use glam::Vec2;
use ily_graphics::{
    Frame, ImageHandle, ImageSource, Quad, Rect, Renderer, TextHit, TextSection, WeakImageHandle,
};

use crate::{
    BoxConstraints, EventSink, FromStyleAttribute, NodeState, RequestRedrawEvent, SendSync,
    StyleAttribute, StyleSelectors, StyleSpecificity, Stylesheet, Unit,
};

#[derive(Clone, Debug, Default)]
pub struct ImageCache {
    images: HashMap<ImageSource, WeakImageHandle>,
}

impl ImageCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.images.len()
    }

    pub fn is_empty(&self) -> bool {
        self.images.is_empty()
    }

    pub fn get(&self, source: &ImageSource) -> Option<ImageHandle> {
        self.images.get(source)?.upgrade()
    }

    pub fn insert(&mut self, source: ImageSource, handle: ImageHandle) {
        self.images.insert(source, handle.downgrade());
    }

    pub fn clear(&mut self) {
        self.images.clear();
    }

    pub fn clean(&mut self) {
        self.images.retain(|_, v| v.is_alive());
    }
}

pub struct EventContext<'a> {
    pub style: &'a Stylesheet,
    pub state: &'a mut NodeState,
    pub renderer: &'a dyn Renderer,
    pub selectors: &'a StyleSelectors,
    pub event_sink: &'a EventSink,
    pub image_cache: &'a mut ImageCache,
}

pub struct LayoutContext<'a> {
    pub style: &'a Stylesheet,
    pub state: &'a mut NodeState,
    pub renderer: &'a dyn Renderer,
    pub selectors: &'a StyleSelectors,
    pub event_sink: &'a EventSink,
    pub image_cache: &'a mut ImageCache,
}

impl<'a> LayoutContext<'a> {
    pub fn style_constraints(&mut self, bc: BoxConstraints) -> BoxConstraints {
        let min_width = self.style_range_group("min-width", "width", bc.width());
        let max_width = self.style_range_group("max-width", "width", bc.width());

        let min_height = self.style_range_group("min-height", "height", bc.height());
        let max_height = self.style_range_group("max-height", "height", bc.height());

        let min_size = bc.constrain(Vec2::new(min_width, min_height));
        let max_size = bc.constrain(Vec2::new(max_width, max_height));

        BoxConstraints::new(min_size, max_size)
    }

    pub fn messure_text(&self, section: &TextSection) -> Option<Rect> {
        self.renderer.messure_text(section)
    }

    pub fn hit_text(&self, section: &TextSection, pos: Vec2) -> Option<TextHit> {
        self.renderer.hit_text(section, pos)
    }
}

pub struct DrawLayer<'a, 'b> {
    draw_context: &'b mut DrawContext<'a>,
    depth: f32,
    clip: Option<Rect>,
}

impl<'a, 'b> DrawLayer<'a, 'b> {
    pub fn depth(mut self, depth: f32) -> Self {
        self.depth = depth;
        self
    }

    pub fn clip(mut self, clip: Rect) -> Self {
        self.clip = Some(clip.round());
        self
    }

    pub fn draw(self, f: impl FnOnce(&mut DrawContext)) {
        let layer = self
            .draw_context
            .frame
            .layer()
            .depth(self.depth)
            .clip(self.clip);

        layer.draw(|frame| {
            let mut child = DrawContext {
                style: self.draw_context.style,
                state: self.draw_context.state,
                frame,
                renderer: self.draw_context.renderer,
                selectors: self.draw_context.selectors,
                event_sink: self.draw_context.event_sink,
                image_cache: self.draw_context.image_cache,
            };

            f(&mut child);
        });
    }
}

pub struct DrawContext<'a> {
    pub style: &'a Stylesheet,
    pub state: &'a mut NodeState,
    pub frame: &'a mut Frame,
    pub renderer: &'a dyn Renderer,
    pub selectors: &'a StyleSelectors,
    pub event_sink: &'a EventSink,
    pub image_cache: &'a mut ImageCache,
}

impl<'a> DrawContext<'a> {
    pub fn frame(&mut self) -> &mut Frame {
        self.frame
    }

    pub fn layer<'b>(&'b mut self) -> DrawLayer<'a, 'b> {
        DrawLayer {
            draw_context: self,
            depth: 1.0,
            clip: None,
        }
    }

    /// Runs the given callback on a new layer offset by the given amount.
    ///
    /// `offset` should almost always be `1.0`.
    pub fn draw_layer(&mut self, f: impl FnOnce(&mut DrawContext)) {
        self.layer().draw(f);
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

        let tl = "border-top-left-radius";
        let tr = "border-top-right-radius";
        let br = "border-bottom-right-radius";
        let bl = "border-bottom-left-radius";

        let tl = self.style_range_group(tl, "border-radius", range.clone());
        let tr = self.style_range_group(tr, "border-radius", range.clone());
        let br = self.style_range_group(br, "border-radius", range.clone());
        let bl = self.style_range_group(bl, "border-radius", range.clone());

        let quad = Quad {
            rect: self.rect(),
            background: self.style("background-color"),
            border_radius: [tl, tr, br, bl],
            border_width: self.style_range("border-width", range),
            border_color: self.style("border-color"),
        };

        self.draw(quad);
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
    fn event_sink(&self) -> &EventSink;
    fn image_cache(&self) -> &ImageCache;
    fn image_cache_mut(&mut self) -> &mut ImageCache;

    fn get_style_attribute(&self, key: &str) -> Option<StyleAttribute> {
        if let Some(attribute) = self.state().style.attributes.get(key) {
            return Some(attribute.clone());
        }

        let attribute = self.stylesheet().get_attribute(self.selectors(), key)?;
        Some(attribute.clone())
    }

    fn get_style_attribute_specificity(
        &self,
        key: &str,
    ) -> Option<(StyleAttribute, StyleSpecificity)> {
        if let Some(attribute) = self.state().style.attributes.get(key) {
            return Some((attribute.clone(), StyleSpecificity::INLINE));
        }

        let stylesheet = self.stylesheet();
        let selectors = self.selectors();
        let (attribute, specificity) = stylesheet.get_attribute_specificity(selectors, key)?;
        Some((attribute.clone(), specificity))
    }

    fn get_style<T: FromStyleAttribute + 'static>(&mut self, key: &str) -> Option<T> {
        let attribute = self.get_style_attribute(key)?;
        let value = T::from_attribute(attribute.value)?;
        let transition = attribute.transition;

        Some(self.state_mut().transition(key, value, transition))
    }

    fn get_style_specificity<T: FromStyleAttribute + 'static>(
        &mut self,
        key: &str,
    ) -> Option<(T, StyleSpecificity)> {
        let (attribute, specificity) = self.get_style_attribute_specificity(key)?;
        let value = T::from_attribute(attribute.value)?;
        let transition = attribute.transition;

        Some((
            self.state_mut().transition(key, value, transition),
            specificity,
        ))
    }

    /// Get the value of a style attribute, if it has a transition, the value will be
    /// interpolated between the current value and the new value.
    #[track_caller]
    fn style<T: FromStyleAttribute + Default + 'static>(&mut self, key: &str) -> T {
        self.get_style(key).unwrap_or_default()
    }

    fn style_group<T: FromStyleAttribute + Default + 'static>(
        &mut self,
        primary_key: &str,
        secondary_key: &str,
    ) -> T {
        let primary = self.get_style_specificity(primary_key);
        let secondary = self.get_style_specificity(secondary_key);

        match (primary, secondary) {
            (Some((primary, primary_specificity)), Some((secondary, secondary_specificity))) => {
                if primary_specificity >= secondary_specificity {
                    primary
                } else {
                    secondary
                }
            }
            (Some((value, _)), None) | (None, Some((value, _))) => value,
            _ => T::default(),
        }
    }

    fn get_style_range(&mut self, key: &str, range: Range<f32>) -> Option<f32> {
        let attribute = self.get_style_attribute(key)?;
        let value = Unit::from_attribute(attribute.value)?;
        let transition = attribute.transition;

        let pixels = value.pixels(range, self.renderer().scale());

        Some((self.state_mut()).transition(key, pixels, transition))
    }

    fn get_style_range_specificity(
        &mut self,
        key: &str,
        range: Range<f32>,
    ) -> Option<(f32, StyleSpecificity)> {
        let (attribute, specificity) = self.get_style_attribute_specificity(key)?;
        let value = Unit::from_attribute(attribute.value)?;
        let transition = attribute.transition;

        let pixels = value.pixels(range, self.renderer().scale());

        Some((
            (self.state_mut()).transition(key, pixels, transition),
            specificity,
        ))
    }

    /// Get the value of a style attribute, if it has a transition, the value will be
    /// interpolated between the current value and the new value.
    ///
    /// This is a convenience method for getting a value in pixels, as opposed to
    /// `style` which returns a `Unit`.
    #[track_caller]
    fn style_range(&mut self, key: &str, range: Range<f32>) -> f32 {
        self.get_style_range(key, range).unwrap_or_default()
    }

    fn style_range_group(
        &mut self,
        primary_key: &str,
        secondary_key: &str,
        range: Range<f32>,
    ) -> f32 {
        let primary = self.get_style_range_specificity(primary_key, range.clone());
        let secondary = self.get_style_range_specificity(secondary_key, range);

        match (primary, secondary) {
            (Some((primary, primary_specificity)), Some((secondary, secondary_specificity))) => {
                if primary_specificity >= secondary_specificity {
                    primary
                } else {
                    secondary
                }
            }
            (Some((value, _)), None) | (None, Some((value, _))) => value,
            _ => 0.0,
        }
    }

    fn downcast_renderer<T: Renderer>(&self) -> Option<&T> {
        self.renderer().downcast_ref()
    }

    fn load_image(&mut self, source: &ImageSource) -> ImageHandle {
        if let Some(handle) = self.image_cache().get(source) {
            return handle;
        }

        let data = source.load();
        let image = self.renderer().create_image(&data);
        self.image_cache_mut().insert(source.clone(), image.clone());
        image
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
        if self.focused() {
            return;
        }

        self.state_mut().focused = true;
        self.request_redraw();
    }

    fn unfocus(&mut self) {
        if !self.focused() {
            return;
        }

        self.state_mut().focused = false;
        self.request_redraw();
    }

    fn activate(&mut self) {
        if self.active() {
            return;
        }

        self.state_mut().active = true;
        self.request_redraw();
    }

    fn deactivate(&mut self) {
        if !self.active() {
            return;
        }

        self.state_mut().active = false;
        self.request_redraw();
    }

    fn local_rect(&self) -> Rect {
        self.state().local_rect
    }

    fn rect(&self) -> Rect {
        self.state().global_rect
    }

    fn size(&self) -> Vec2 {
        self.state().local_rect.size()
    }

    fn request_redraw(&mut self) {
        self.send_event(RequestRedrawEvent);
    }

    fn send_event(&self, event: impl Any + SendSync) {
        self.event_sink().send(event);
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

            fn event_sink(&self) -> &EventSink {
                &self.event_sink
            }

            fn image_cache(&self) -> &ImageCache {
                &self.image_cache
            }

            fn image_cache_mut(&mut self) -> &mut ImageCache {
                &mut self.image_cache
            }
        }
    };
}

context!(EventContext);
context!(LayoutContext);
context!(DrawContext);
