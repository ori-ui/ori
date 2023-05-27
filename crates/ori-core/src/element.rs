use std::{any::Any, fmt::Debug, sync::Arc, time::Instant};

use glam::Vec2;
use ori_graphics::{Frame, Rect, Renderer};
use ori_reactive::{Event, EventSink};
use parking_lot::{Mutex, MutexGuard};
use uuid::Uuid;

use crate::{
    AnyView, AvailableSpace, Context, Cursor, DebugEvent, DrawContext, EmptyView, EventContext,
    FromStyleAttribute, ImageCache, LayoutContext, Margin, Padding, PointerEvent,
    RequestRedrawEvent, Style, StyleAttribute, StyleCache, StyleSelector, StyleSelectors,
    StyleSpecificity, StyleStates, StyleTransition, Stylesheet, TransitionStates, View,
    WindowResizeEvent,
};

/// An element identifier. This uses a UUID to ensure that elements are unique.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ElementId {
    uuid: Uuid,
}

impl ElementId {
    /// Create a new element identifier, using uuid v4.
    pub fn new() -> Self {
        Self {
            uuid: Uuid::new_v4(),
        }
    }

    /// Gets the inner uuid.
    pub const fn uuid(self) -> Uuid {
        self.uuid
    }
}

impl Default for ElementId {
    fn default() -> Self {
        Self::new()
    }
}

/// The state of a element, which is used to store information about the element.
///
/// This should almost never be used directly, and instead should be used through the [`Element`]
/// struct.
#[derive(Clone, Debug)]
pub struct ElementState {
    pub id: ElementId,
    pub margin: Margin,
    pub padding: Padding,
    pub local_rect: Rect,
    pub global_rect: Rect,
    pub active: bool,
    pub focused: bool,
    pub hovered: bool,
    pub last_draw: Instant,
    pub style: Style,
    pub needs_layout: bool,
    pub available_space: AvailableSpace,
    pub transitions: TransitionStates,
}

impl Default for ElementState {
    fn default() -> Self {
        Self {
            id: ElementId::new(),
            margin: Margin::ZERO,
            padding: Padding::ZERO,
            local_rect: Rect::ZERO,
            global_rect: Rect::ZERO,
            active: false,
            focused: false,
            hovered: false,
            last_draw: Instant::now(),
            style: Style::default(),
            needs_layout: true,
            available_space: AvailableSpace::ZERO,
            transitions: TransitionStates::new(),
        }
    }
}

impl ElementState {
    /// Create a new [`ElementState`] with the given style.
    pub fn new(style: Style) -> Self {
        Self {
            style,
            ..Default::default()
        }
    }

    /// Propagate the [`ElementState`] up to the parent.
    ///
    /// This is called before events are propagated.
    pub fn propagate_up(&mut self, parent: &mut ElementState) {
        self.global_rect = self.local_rect.translate(parent.global_rect.min);
    }

    /// Propagate the [`ElementState`] down to the child.
    ///
    /// This is called after events are propagated.
    pub fn propagate_down(&mut self, child: &mut ElementState) {
        self.needs_layout |= child.needs_layout;
    }

    /// Returns the [`StyleStates´].
    pub fn style_states(&self) -> StyleStates {
        let mut states = StyleStates::new();

        if self.active {
            states.push("active");
        }

        if self.focused {
            states.push("focus");
        }

        if self.hovered {
            states.push("hover");
        }

        states
    }

    /// Returns the [`StyleSelector`].
    pub fn selector(&self) -> StyleSelector {
        StyleSelector {
            element: self.style.element.map(Into::into),
            classes: self.style.classes.clone(),
            states: self.style_states(),
        }
    }

    /// Returns the time in seconds since the last draw.
    pub fn delta_time(&self) -> f32 {
        self.last_draw.elapsed().as_secs_f32()
    }

    pub fn get_style_attribyte(
        &mut self,
        cx: &mut impl Context,
        key: &str,
    ) -> Option<StyleAttribute> {
        self.get_style_attribute_specificity(cx, key)
            .map(|(attribute, _)| attribute)
    }

    pub fn get_style_attribute_specificity(
        &mut self,
        cx: &mut impl Context,
        key: &str,
    ) -> Option<(StyleAttribute, StyleSpecificity)> {
        if let Some(attribute) = self.style.attributes.get(key) {
            return Some((attribute.clone(), StyleSpecificity::INLINE));
        }

        let selectors = cx.selectors().clone().with(self.selector());
        let hash = selectors.hash();

        if let Some(result) = cx.style_cache().get_attribute(hash, key) {
            return result;
        }

        let stylesheet = cx.stylesheet();

        match stylesheet.get_attribute_specificity(&selectors, key) {
            Some((attribute, specificity)) => {
                (cx.style_cache_mut()).insert(hash, attribute.clone(), specificity);
                Some((attribute, specificity))
            }
            None => {
                cx.style_cache_mut().insert_none(hash, key);
                None
            }
        }
    }

    pub fn get_style_specificity<T: FromStyleAttribute + 'static>(
        &mut self,
        cx: &mut impl Context,
        key: &str,
    ) -> Option<(T, StyleSpecificity)> {
        let (attribute, specificity) = self.get_style_attribute_specificity(cx, key)?;
        let value = T::from_attribute(attribute.value().clone())?;
        let transition = attribute.transition();

        Some((self.transition(key, value, transition), specificity))
    }

    pub fn get_style<T: FromStyleAttribute + 'static>(
        &mut self,
        cx: &mut impl Context,
        key: &str,
    ) -> Option<T> {
        self.get_style_specificity(cx, key).map(|(value, _)| value)
    }

    pub fn style<T: FromStyleAttribute + Default + 'static>(
        &mut self,
        cx: &mut impl Context,
        key: &str,
    ) -> T {
        self.get_style(cx, key).unwrap_or_default()
    }

    pub fn style_group<T: FromStyleAttribute + Default + 'static>(
        &mut self,
        cx: &mut impl Context,
        keys: &[&str],
    ) -> T {
        let mut specificity = None;
        let mut result = None;

        for key in keys {
            if let Some((v, s)) = self.get_style_specificity(cx, key) {
                if specificity.is_none() || s > specificity.unwrap() {
                    specificity = Some(s);
                    result = Some(v);
                }
            }
        }

        result.unwrap_or_default()
    }

    /// Transition a value.
    ///
    /// If the value is an [`f32`], or a [`Color`](ori_graphics::Color), then it will be transitioned.
    pub fn transition<T: 'static>(
        &mut self,
        name: &str,
        mut value: T,
        transition: Option<StyleTransition>,
    ) -> T {
        (self.transitions).transition_any(name, &mut value, transition);
        value
    }

    /// Update the transitions.
    pub fn update_transitions(&mut self) -> bool {
        self.transitions.update(self.delta_time())
    }

    pub fn space_changed(&mut self, space: AvailableSpace) -> bool {
        self.available_space != space
    }

    /// Updates `self.last_draw` to the current time.
    fn draw(&mut self) {
        self.last_draw = Instant::now();
    }
}

pub trait ElementView: Send + Sync + 'static {
    type State: Send + Sync + 'static;

    fn build(&self) -> Self::State;

    fn style(&self) -> Style;

    fn event(&self, state: &mut Self::State, cx: &mut EventContext, event: &Event);

    fn layout(
        &self,
        state: &mut Self::State,
        cx: &mut LayoutContext,
        space: AvailableSpace,
    ) -> Vec2;

    fn draw(&self, state: &mut Self::State, cx: &mut DrawContext);
}

impl<T: View> ElementView for T {
    type State = T::State;

    fn build(&self) -> Self::State {
        self.build()
    }

    fn style(&self) -> Style {
        self.style()
    }

    fn event(&self, state: &mut Self::State, cx: &mut EventContext, event: &Event) {
        self.event(state, cx, event)
    }

    fn layout(
        &self,
        state: &mut Self::State,
        cx: &mut LayoutContext,
        space: AvailableSpace,
    ) -> Vec2 {
        self.layout(state, cx, space)
    }

    fn draw(&self, state: &mut Self::State, cx: &mut DrawContext) {
        self.draw(state, cx)
    }
}

impl ElementView for Box<dyn AnyView> {
    type State = Box<dyn Any + Send + Sync>;

    fn build(&self) -> Self::State {
        self.as_ref().build()
    }

    fn style(&self) -> Style {
        self.as_ref().style()
    }

    fn event(&self, state: &mut Self::State, cx: &mut EventContext, event: &Event) {
        self.as_ref().event(state.as_mut(), cx, event)
    }

    fn layout(
        &self,
        state: &mut Self::State,
        cx: &mut LayoutContext,
        space: AvailableSpace,
    ) -> Vec2 {
        self.as_ref().layout(state.as_mut(), cx, space)
    }

    fn draw(&self, state: &mut Self::State, cx: &mut DrawContext) {
        self.as_ref().draw(state.as_mut(), cx)
    }
}

struct ElementInner<T: ElementView> {
    view_state: Mutex<T::State>,
    element_state: Mutex<ElementState>,
    view: Mutex<T>,
}

impl<T: ElementView> Debug for ElementInner<T>
where
    T: Debug,
    T::State: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ElementInner")
            .field("view_state", &self.view_state)
            .field("element_state", &self.element_state)
            .field("view", &self.view.type_id())
            .finish()
    }
}

pub trait IntoElement<V: ElementView = Box<dyn AnyView>> {
    fn into_element(self) -> Element<V>;
}

impl<T: View> IntoElement<T> for T {
    fn into_element(self) -> Element<T> {
        Element::from_view(self)
    }
}

impl<T: View> IntoElement for T {
    fn into_element(self) -> Element {
        Element::from_view(Box::new(self))
    }
}

impl<T: ElementView> IntoElement<T> for Element<T> {
    fn into_element(self) -> Element<T> {
        self
    }
}

pub trait DowncastElement<T: ElementView> {
    fn downcast_ref(&self) -> Option<&T>;
    fn downcast_mut(&mut self) -> Option<&mut T>;
}

impl<T: View> DowncastElement<T> for T {
    fn downcast_ref(&self) -> Option<&T> {
        Some(self)
    }

    fn downcast_mut(&mut self) -> Option<&mut T> {
        Some(self)
    }
}

impl<T: View> DowncastElement<T> for Box<dyn AnyView> {
    fn downcast_ref(&self) -> Option<&T> {
        self.as_ref().downcast_ref()
    }

    fn downcast_mut(&mut self) -> Option<&mut T> {
        self.as_mut().downcast_mut()
    }
}

/// A element in the ui tree.
///
/// A element is a wrapper around a view, and contains the state of the view.
pub struct Element<T: ElementView = Box<dyn AnyView>> {
    inner: Arc<ElementInner<T>>,
}

impl<T: ElementView> Clone for Element<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl Element {
    pub fn empty() -> Self {
        Self::new(EmptyView)
    }
}

impl<T: ElementView> Element<T> {
    /// Create a new element with the given [`View`].
    pub fn new(view: impl IntoElement<T>) -> Self {
        view.into_element()
    }

    pub fn from_view(view: T) -> Self {
        let view_state = ElementView::build(&view);
        let element_state = ElementState::new(ElementView::style(&view));

        Self {
            inner: Arc::new(ElementInner {
                view_state: Mutex::new(view_state),
                element_state: Mutex::new(element_state),
                view: Mutex::new(view),
            }),
        }
    }

    /// Returns a [`MutexGuard`] to the state of the `T`.
    ///
    /// Be careful when using this, as it can cause deadlocks.
    pub fn view_state(&self) -> MutexGuard<'_, T::State> {
        self.inner.view_state.lock()
    }

    /// Returns a [`MutexGuard`] to the [`ElementState`].
    ///
    /// Be careful when using this, as it can cause deadlocks.
    pub fn element_state(&self) -> MutexGuard<'_, ElementState> {
        self.inner.element_state.lock()
    }

    /// Returns a [`MutexGuard`] to the `T`.
    pub fn view(&self) -> MutexGuard<'_, T> {
        self.inner.view.lock()
    }

    pub fn with_view<U: ElementView>(&self, f: impl FnOnce(&mut U))
    where
        T: DowncastElement<U>,
    {
        if let Some(mut view) = self.view().downcast_mut() {
            f(&mut view);
        } else {
            tracing::error!("Failed to downcast view");
        }

        self.request_layout();
    }

    /// Sets the offset of the element, relative to the parent.
    pub fn set_offset(&self, offset: Vec2) {
        let mut element_state = self.element_state();

        let size = element_state.local_rect.size();
        element_state.local_rect = Rect::min_size(element_state.margin.top_left() + offset, size);
    }

    pub fn get_style<S: FromStyleAttribute + 'static>(
        &self,
        cx: &mut impl Context,
        key: &str,
    ) -> Option<S> {
        self.element_state().get_style(cx, key)
    }

    pub fn style<S: FromStyleAttribute + Default + 'static>(
        &self,
        cx: &mut impl Context,
        key: &str,
    ) -> S {
        self.get_style(cx, key).unwrap_or_default()
    }

    pub fn style_group<S: FromStyleAttribute + Default + 'static>(
        &self,
        cx: &mut impl Context,
        key: &[&str],
    ) -> S {
        self.element_state().style_group(cx, key)
    }

    /// Returns the [`StyleStates`].
    pub fn style_states(&self) -> StyleStates {
        self.element_state().style_states()
    }

    /// Returns true if the element needs to be laid out.
    pub fn needs_layout(&self) -> bool {
        self.element_state().needs_layout
    }

    pub fn available_space(&self) -> AvailableSpace {
        self.element_state().available_space
    }

    pub fn set_available_space(&self, space: AvailableSpace) {
        self.element_state().available_space = space;
    }

    pub fn space_changed(&self, space: AvailableSpace) -> bool {
        self.element_state().space_changed(space)
    }

    /// Requests a layout.
    pub fn request_layout(&self) {
        self.element_state().needs_layout = true;
    }

    /// Gets the local [`Rect`] of the element.
    pub fn local_rect(&self) -> Rect {
        self.element_state().local_rect
    }

    /// Gets the global [`Rect`] of the element.
    pub fn global_rect(&self) -> Rect {
        self.element_state().global_rect
    }

    /// Gets the size of the element.
    pub fn size(&self) -> Vec2 {
        let element_state = self.element_state();
        element_state.local_rect.size() + element_state.margin.size()
    }
}

impl<T: ElementView> Element<T> {
    /// Returns true if the element should be redrawn.
    fn handle_pointer_event(
        element_state: &mut ElementState,
        event: &PointerEvent,
        is_handled: bool,
    ) -> bool {
        let is_over =
            element_state.global_rect.contains(event.position) && !event.left && !is_handled;
        if is_over != element_state.hovered && event.is_motion() {
            element_state.hovered = is_over;
            true
        } else {
            false
        }
    }

    /// Update the cursor.
    fn update_cursor(cx: &mut impl Context) {
        let Some(cursor) = cx.style("cursor") else {
            return;
        };

        if cx.hovered() || cx.active() {
            cx.set_cursor(cursor);
        }
    }

    fn with_inner<C: Context, O>(
        &self,
        cx: &mut C,
        f: impl FnOnce(&mut ElementState, &mut C) -> O,
    ) -> O {
        let element_state = &mut self.element_state();
        element_state.style = self.view().style();
        element_state.propagate_up(cx.state_mut());

        if element_state.needs_layout {
            cx.request_redraw();
        }

        let res = f(element_state, cx);

        cx.state_mut().propagate_down(element_state);

        res
    }

    fn event_inner(&self, state: &mut ElementState, cx: &mut EventContext, event: &Event) {
        if let Some(pointer_event) = event.get::<PointerEvent>() {
            if Self::handle_pointer_event(state, pointer_event, event.is_handled()) {
                cx.request_redraw();
            }
        }

        let selector = state.selector();
        let selectors = cx.selectors.clone().with(selector);
        let mut cx = EventContext {
            state,
            renderer: cx.renderer,
            selectors: &selectors,
            selectors_hash: selectors.hash(),
            stylesheet: cx.stylesheet,
            style_cache: cx.style_cache,
            event_sink: cx.event_sink,
            image_cache: cx.image_cache,
            cursor: cx.cursor,
        };

        if let Some(event) = event.get::<DebugEvent>() {
            event.with_element(&mut cx, self);
            return;
        }

        self.view().event(&mut self.view_state(), &mut cx, event);

        Self::update_cursor(&mut cx);
    }

    /// Handle an event.
    pub fn event(&self, cx: &mut EventContext, event: &Event) {
        self.with_inner(cx, |element_state, cx| {
            self.event_inner(element_state, cx, event);
        });
    }

    /// Layout the element.
    pub fn layout(&self, cx: &mut LayoutContext, space: AvailableSpace) -> Vec2 {
        let size = self.relayout(cx, space);
        self.set_available_space(space);
        size
    }

    fn relayout_inner(
        &self,
        state: &mut ElementState,
        cx: &mut LayoutContext,
        space: AvailableSpace,
    ) -> Vec2 {
        state.needs_layout = false;

        let selector = state.selector();
        let selectors = cx.selectors.clone().with(selector);
        let mut cx = LayoutContext {
            state,
            renderer: cx.renderer,
            selectors: &selectors,
            selectors_hash: selectors.hash(),
            stylesheet: cx.stylesheet,
            style_cache: cx.style_cache,
            event_sink: cx.event_sink,
            image_cache: cx.image_cache,
            cursor: cx.cursor,
            parent_space: cx.space,
            space,
        };

        cx.state.margin = Margin::from_style(&mut cx, space);
        cx.state.padding = Padding::from_style(&mut cx, space);

        let space = space.apply_margin(cx.state.margin);
        let space = cx.style_constraints(space);
        cx.space = space;

        let size = self.view().layout(&mut self.view_state(), &mut cx, space);

        Self::update_cursor(&mut cx);

        let local_offset = state.local_rect.min + state.margin.top_left();
        let global_offset = state.global_rect.min + state.margin.top_left();
        state.local_rect = Rect::min_size(local_offset, size);
        state.global_rect = Rect::min_size(global_offset, size);

        size + state.margin.size()
    }

    pub fn relayout(&self, cx: &mut LayoutContext, space: AvailableSpace) -> Vec2 {
        self.with_inner(cx, |element_state, cx| {
            self.relayout_inner(element_state, cx, space)
        })
    }

    fn draw_inner(&self, state: &mut ElementState, cx: &mut DrawContext) {
        let selector = state.selector();
        let selectors = cx.selectors.clone().with(selector);
        let mut cx = DrawContext {
            state,
            frame: cx.frame,
            renderer: cx.renderer,
            selectors: &selectors,
            selectors_hash: selectors.hash(),
            stylesheet: cx.stylesheet,
            style_cache: cx.style_cache,
            event_sink: cx.event_sink,
            image_cache: cx.image_cache,
            cursor: cx.cursor,
        };

        self.view().draw(&mut self.view_state(), &mut cx);

        if cx.state.update_transitions() {
            cx.request_redraw();
            cx.request_layout();
        }

        cx.state.draw();

        Self::update_cursor(&mut cx);
    }

    /// Draw the element.
    pub fn draw(&self, cx: &mut DrawContext) {
        self.with_inner(cx, |element_state, cx| {
            self.draw_inner(element_state, cx);
        });
    }
}

impl<T: ElementView> Element<T> {
    pub(crate) fn event_root_inner(
        &self,
        stylesheet: &Stylesheet,
        style_cache: &mut StyleCache,
        renderer: &dyn Renderer,
        event_sink: &EventSink,
        event: &Event,
        image_cache: &mut ImageCache,
        cursor: &mut Cursor,
    ) {
        let element_state = &mut self.element_state();
        element_state.style = self.view().style();

        if let Some(pointer_event) = event.get::<PointerEvent>() {
            if Self::handle_pointer_event(element_state, pointer_event, event.is_handled()) {
                event_sink.emit(RequestRedrawEvent);
            }
        }

        if event.is::<WindowResizeEvent>() {
            element_state.needs_layout = true;
        }

        let selector = element_state.selector();
        let selectors = StyleSelectors::new().with(selector);
        let mut cx = EventContext {
            state: element_state,
            renderer,
            selectors: &selectors,
            selectors_hash: selectors.hash(),
            stylesheet,
            event_sink,
            style_cache,
            image_cache,
            cursor,
        };

        if let Some(event) = event.get::<DebugEvent>() {
            event.set_element(&mut cx, self);
        }

        self.view().event(&mut self.view_state(), &mut cx, event);
    }

    pub(crate) fn layout_root_inner(
        &self,
        stylesheet: &Stylesheet,
        style_cache: &mut StyleCache,
        renderer: &dyn Renderer,
        event_sink: &EventSink,
        image_cache: &mut ImageCache,
        cursor: &mut Cursor,
    ) -> Vec2 {
        let element_state = &mut self.element_state();
        element_state.style = self.view().style();
        element_state.needs_layout = false;

        let space = AvailableSpace::new(Vec2::ZERO, renderer.window_size());

        let selector = element_state.selector();
        let selectors = StyleSelectors::new().with(selector);
        let mut cx = LayoutContext {
            state: element_state,
            renderer,
            selectors: &selectors,
            selectors_hash: selectors.hash(),
            stylesheet,
            event_sink,
            style_cache,
            image_cache,
            cursor,
            parent_space: space,
            space,
        };

        cx.state.margin = Margin::from_style(&mut cx, space);
        cx.state.padding = Padding::from_style(&mut cx, space);

        let space = cx.style_constraints(space);
        cx.space = space;

        let size = self.view().layout(&mut self.view_state(), &mut cx, space);

        element_state.available_space = space;
        element_state.local_rect = Rect::min_size(element_state.local_rect.min, size);
        element_state.global_rect = Rect::min_size(element_state.global_rect.min, size);

        size
    }

    pub(crate) fn draw_root_inner(
        &self,
        stylesheet: &Stylesheet,
        style_cache: &mut StyleCache,
        frame: &mut Frame,
        renderer: &dyn Renderer,
        event_sink: &EventSink,
        image_cache: &mut ImageCache,
        cursor: &mut Cursor,
    ) {
        let element_state = &mut self.element_state();
        element_state.style = self.view().style();

        let selector = element_state.selector();
        let selectors = StyleSelectors::new().with(selector);
        let mut cx = DrawContext {
            state: element_state,
            frame,
            renderer,
            selectors: &selectors,
            selectors_hash: selectors.hash(),
            stylesheet,
            event_sink,
            style_cache,
            image_cache,
            cursor,
        };

        self.view().draw(&mut self.view_state(), &mut cx);

        cx.state.draw();
    }
}

impl<T: ElementView> Debug for Element<T>
where
    T: Debug,
    T::State: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node").field("inner", &self.inner).finish()
    }
}
