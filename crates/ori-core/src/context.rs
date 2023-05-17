use std::{
    any::Any,
    collections::HashMap,
    ops::{Deref, DerefMut, Range},
};

use glam::Vec2;
use ori_graphics::{
    Frame, ImageHandle, ImageSource, Quad, Rect, Renderer, TextHit, TextSection, WeakImageHandle,
};

use crate::{
    BoxConstraints, Cursor, EventSink, FromStyleAttribute, NodeState, RequestRedrawEvent,
    StyleAttribute, StyleCache, StyleSelectors, StyleSelectorsHash, StyleSpecificity, Stylesheet,
    Unit,
};

/// A cache for images.
///
/// This is used to avoid loading the same image multiple times.
#[derive(Clone, Debug, Default)]
pub struct ImageCache {
    images: HashMap<ImageSource, WeakImageHandle>,
}

impl ImageCache {
    /// Creates a new image cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the number of images in the cache.
    pub fn len(&self) -> usize {
        self.images.len()
    }

    /// Returns `true` if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.images.is_empty()
    }

    /// Gets an image from the cache.
    pub fn get(&self, source: &ImageSource) -> Option<ImageHandle> {
        self.images.get(source)?.upgrade()
    }

    /// Inserts an image into the cache.
    pub fn insert(&mut self, source: ImageSource, handle: ImageHandle) {
        self.images.insert(source, handle.downgrade());
    }

    /// Clears the cache.
    pub fn clear(&mut self) {
        self.images.clear();
    }

    /// Removes all images that are no longer in use.
    pub fn clean(&mut self) {
        self.images.retain(|_, v| v.is_alive());
    }
}

/// A context for [`View::event`].
pub struct EventContext<'a> {
    pub state: &'a mut NodeState,
    pub renderer: &'a dyn Renderer,
    pub selectors: &'a StyleSelectors,
    pub selectors_hash: StyleSelectorsHash,
    pub style: &'a Stylesheet,
    pub style_cache: &'a mut StyleCache,
    pub event_sink: &'a EventSink,
    pub image_cache: &'a mut ImageCache,
    pub cursor: &'a mut Cursor,
}

/// A context for [`View::layout`].
pub struct LayoutContext<'a> {
    pub state: &'a mut NodeState,
    pub renderer: &'a dyn Renderer,
    pub selectors: &'a StyleSelectors,
    pub selectors_hash: StyleSelectorsHash,
    pub style: &'a Stylesheet,
    pub style_cache: &'a mut StyleCache,
    pub event_sink: &'a EventSink,
    pub image_cache: &'a mut ImageCache,
    pub cursor: &'a mut Cursor,
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
    z_index: f32,
    clip: Option<Rect>,
}

impl<'a, 'b> DrawLayer<'a, 'b> {
    pub fn z_index(mut self, depth: f32) -> Self {
        self.z_index = depth;
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
            .z_index(self.z_index)
            .clip(self.clip);

        layer.draw(|frame| {
            let mut child = DrawContext {
                state: self.draw_context.state,
                frame,
                renderer: self.draw_context.renderer,
                selectors: self.draw_context.selectors,
                selectors_hash: self.draw_context.selectors_hash,
                style: self.draw_context.style,
                style_cache: self.draw_context.style_cache,
                event_sink: self.draw_context.event_sink,
                image_cache: self.draw_context.image_cache,
                cursor: self.draw_context.cursor,
            };

            f(&mut child);
        });
    }
}

/// A context for [`View::draw`].
pub struct DrawContext<'a> {
    pub state: &'a mut NodeState,
    pub frame: &'a mut Frame,
    pub renderer: &'a dyn Renderer,
    pub selectors: &'a StyleSelectors,
    pub selectors_hash: StyleSelectorsHash,
    pub style: &'a Stylesheet,
    pub style_cache: &'a mut StyleCache,
    pub event_sink: &'a EventSink,
    pub image_cache: &'a mut ImageCache,
    pub cursor: &'a mut Cursor,
}

impl<'a> DrawContext<'a> {
    pub fn frame(&mut self) -> &mut Frame {
        self.frame
    }

    pub fn layer<'b>(&'b mut self) -> DrawLayer<'a, 'b> {
        DrawLayer {
            draw_context: self,
            z_index: 1.0,
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

/// A context that is passed to [`View`](crate::view::View) methods.
///
/// See [`EventContext`], [`DrawContext`] and [`LayoutContext`] for more information.
pub trait Context {
    /// Returns the [`Stylesheet`] of the application.
    fn stylesheet(&self) -> &Stylesheet;

    /// Returns the [`StyleCache`] of the application.
    fn style_cache(&self) -> &StyleCache;

    /// Returns the [`StyleCache`] of the application.
    fn style_cache_mut(&mut self) -> &mut StyleCache;

    /// Returns the [`NodeState`] of the current node.
    fn state(&self) -> &NodeState;

    /// Returns the [`NodeState`] of the current node.
    fn state_mut(&mut self) -> &mut NodeState;

    /// Returns the [`Renderer`] of the application.
    fn renderer(&self) -> &dyn Renderer;

    /// Returns the [`StyleSelectors`] of the current node.
    fn selectors(&self) -> &StyleSelectors;

    /// Returns the [`StyleSelectorsHash`] of the current node.
    fn selectors_hash(&self) -> StyleSelectorsHash;

    /// Returns the [`EventSink`] of the application.
    fn event_sink(&self) -> &EventSink;

    /// Returns the [`ImageCache`] of the application.
    fn image_cache(&self) -> &ImageCache;

    /// Returns the [`ImageCache`] of the application.
    fn image_cache_mut(&mut self) -> &mut ImageCache;

    /// Returns the current [`Cursor`].
    fn cursor(&self) -> Cursor;

    /// Sets the [`Cursor`].
    fn set_cursor(&mut self, icon: Cursor);

    /// Gets the [`StyleAttribute`] for the given `key`.
    fn get_style_attribute(&mut self, key: &str) -> Option<StyleAttribute> {
        self.get_style_attribute_specificity(key)
            .map(|(attribute, _)| attribute)
    }

    /// Gets the [`StyleAttribute`] and [`StyleSpecificity`] for the given `key`.
    fn get_style_attribute_specificity(
        &mut self,
        key: &str,
    ) -> Option<(StyleAttribute, StyleSpecificity)> {
        // get inline style attribute
        if let Some(attribute) = self.state().style.attributes.get(key) {
            return Some((attribute.clone(), StyleSpecificity::INLINE));
        }

        let hash = self.selectors_hash();

        // try to get cached attribute
        if let Some(result) = self.style_cache().get_attribute(hash, key) {
            return result;
        }

        let stylesheet = self.stylesheet();
        let selectors = self.selectors();

        // get attribute from stylesheet
        match stylesheet.get_attribute_specificity(selectors, key) {
            Some((attribute, specificity)) => {
                // cache result
                (self.style_cache_mut()).insert(hash, attribute.clone(), specificity);
                Some((attribute, specificity))
            }
            None => {
                // cache result
                self.style_cache_mut().insert_none(hash, key);
                None
            }
        }
    }

    /// Gets the value of a style attribute for the given `key`.
    ///
    /// This will also transition the value if the attribute has a transition.
    fn get_style<T: FromStyleAttribute + 'static>(&mut self, key: &str) -> Option<T> {
        let attribute = self.get_style_attribute(key)?;
        let value = T::from_attribute(attribute.value().clone())?;
        let transition = attribute.transition();

        Some(self.state_mut().transition(key, value, transition))
    }

    /// Gets the value of a style attribute for the given `key`.
    fn get_style_specificity<T: FromStyleAttribute + 'static>(
        &mut self,
        key: &str,
    ) -> Option<(T, StyleSpecificity)> {
        let (attribute, specificity) = self.get_style_attribute_specificity(key)?;
        let value = T::from_attribute(attribute.value().clone())?;
        let transition = attribute.transition();

        Some((
            self.state_mut().transition(key, value, transition),
            specificity,
        ))
    }

    /// Gets the value of a style attribute for the given `key`, if there is no value, returns `T::default()`.
    ///
    /// This will also transition the value if the attribute has a transition.
    #[track_caller]
    fn style<T: FromStyleAttribute + Default + 'static>(&mut self, key: &str) -> T {
        self.get_style(key).unwrap_or_default()
    }

    /// Takes a `primary_key` and a `secondary_key` and returns the value of the attribute with the highest specificity.
    /// If both attributes have the same specificity, the `primary_key` will be used.
    ///
    /// This will also transition the value if the attribute has a transition.
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

    /// Gets the value of a style attribute in pixels for the given `key`.
    /// `range` is the range from 0% to 100% of the desired value.
    ///
    /// This will also transition the value if the attribute has a transition.
    fn get_style_range(&mut self, key: &str, range: Range<f32>) -> Option<f32> {
        let attribute = self.get_style_attribute(key)?;
        let value = Unit::from_attribute(attribute.value().clone())?;
        let transition = attribute.transition();

        let pixels = value.pixels(
            range,
            self.renderer().scale(),
            self.renderer().window_size(),
        );

        Some((self.state_mut()).transition(key, pixels, transition))
    }

    /// Gets the value of a style attribute in pixels and [`StyleSpecificity`] for the given `key`.
    fn get_style_range_specificity(
        &mut self,
        key: &str,
        range: Range<f32>,
    ) -> Option<(f32, StyleSpecificity)> {
        let (attribute, specificity) = self.get_style_attribute_specificity(key)?;
        let value = Unit::from_attribute(attribute.value().clone())?;
        let transition = attribute.transition();

        let pixels = value.pixels(
            range,
            self.renderer().scale(),
            self.renderer().window_size(),
        );

        Some((
            (self.state_mut()).transition(key, pixels, transition),
            specificity,
        ))
    }

    /// Gets the value of a style attribute in pixels for the given `key`, if there is no value, returns `0.0`.
    /// `range` is the range from 0% to 100% of the desired value.
    ///
    /// This will also transition the value if the attribute has a transition.
    #[track_caller]
    fn style_range(&mut self, key: &str, range: Range<f32>) -> f32 {
        self.get_style_range(key, range).unwrap_or_default()
    }

    /// Takes a `primary_key` and a `secondary_key` and returns the value of the attribute with the highest specificity in pixels.
    /// If both attributes have the same specificity, the `primary_key` will be used.
    /// `range` is the range from 0% to 100% of the desired value.
    ///
    /// This will also transition the value if the attribute has a transition.
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

    /// Tries to downcast the `renderer` to the given type.
    fn downcast_renderer<T: Renderer>(&self) -> Option<&T> {
        self.renderer().downcast_ref()
    }

    /// Loads an image from the given `source` and returns a handle to it.
    fn load_image(&mut self, source: &ImageSource) -> ImageHandle {
        if let Some(handle) = self.image_cache().get(source) {
            return handle;
        }

        let data = source.load();
        let image = self.renderer().create_image(&data);
        self.image_cache_mut().insert(source.clone(), image.clone());
        image
    }

    /// Returns `true` if the node is active.
    fn active(&self) -> bool {
        self.state().active
    }

    /// Returns `true` if the node is hovered.
    fn hovered(&self) -> bool {
        self.state().hovered
    }

    /// Returns `true` if the node is focused.
    fn focused(&self) -> bool {
        self.state().focused
    }

    /// Focuses the node, this will also request a redraw.
    fn focus(&mut self) {
        if self.focused() {
            return;
        }

        self.state_mut().focused = true;
        self.request_redraw();
    }

    /// Unfocuses the node, this will also request a redraw.
    fn unfocus(&mut self) {
        if !self.focused() {
            return;
        }

        self.state_mut().focused = false;
        self.request_redraw();
    }

    /// Hovers the node, this will also request a redraw.
    fn hover(&mut self) {
        if self.hovered() {
            return;
        }

        self.state_mut().hovered = true;
        self.request_redraw();
    }

    /// Unhovers the node, this will also request a redraw.
    fn unhover(&mut self) {
        if !self.hovered() {
            return;
        }

        self.state_mut().hovered = false;
        self.request_redraw();
    }

    /// Activates the node, this will also request a redraw.
    fn activate(&mut self) {
        if self.active() {
            return;
        }

        self.state_mut().active = true;
        self.request_redraw();
    }

    /// Deactivates the node, this will also request a redraw.
    fn deactivate(&mut self) {
        if !self.active() {
            return;
        }

        self.state_mut().active = false;
        self.request_redraw();
    }

    /// Returns the local rect of the node.
    fn local_rect(&self) -> Rect {
        self.state().local_rect
    }

    /// Returns the global rect of the node.
    fn rect(&self) -> Rect {
        self.state().global_rect
    }

    /// Returns the size of the node.
    fn size(&self) -> Vec2 {
        self.state().local_rect.size()
    }

    /// Requests a redraw.
    ///
    /// This is a shortcut for `self.event_sink().send(RequestRedrawEvent)`.
    fn request_redraw(&mut self) {
        self.send_event(RequestRedrawEvent);
    }

    /// Sends an event to the event sink.
    fn send_event(&self, event: impl Any + Send + Sync) {
        self.event_sink().emit(event);
    }

    /// Returns the time in seconds since the last frame.
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

            fn style_cache(&self) -> &StyleCache {
                self.style_cache
            }

            fn style_cache_mut(&mut self) -> &mut StyleCache {
                self.style_cache
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

            fn selectors_hash(&self) -> StyleSelectorsHash {
                self.selectors_hash
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

            fn cursor(&self) -> Cursor {
                *self.cursor
            }

            fn set_cursor(&mut self, cursor: Cursor) {
                *self.cursor = cursor;
            }
        }
    };
}

context!(EventContext);
context!(LayoutContext);
context!(DrawContext);
