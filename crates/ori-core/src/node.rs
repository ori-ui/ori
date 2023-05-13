use std::{any::Any, fmt::Debug, time::Instant};

use glam::Vec2;
use ori_graphics::{Frame, Rect, Renderer};
use uuid::Uuid;

use crate::{
    AnyView, BoxConstraints, Context, Cursor, DrawContext, Event, EventContext, EventSink, Guard,
    ImageCache, LayoutContext, Lock, Lockable, PointerEvent, RequestRedrawEvent, Shared,
    SharedSignal, Style, StyleSelector, StyleSelectors, StyleStates, StyleTransition, Stylesheet,
    TransitionStates, View,
};

/// A node identifier. This uses a UUID to ensure that nodes are unique.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NodeId {
    uuid: Uuid,
}

impl NodeId {
    /// Create a new node identifier, using uuid v4.
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

impl Default for NodeId {
    fn default() -> Self {
        Self::new()
    }
}

/// The state of a node, which is used to store information about the node.
///
/// This should almost never be used directly, and instead should be used through the [`Node`]
/// struct.
#[derive(Clone, Debug)]
pub struct NodeState {
    pub id: NodeId,
    pub local_rect: Rect,
    pub global_rect: Rect,
    pub active: bool,
    pub focused: bool,
    pub hovered: bool,
    pub last_draw: Instant,
    pub style: Style,
    pub recreated: SharedSignal<bool>,
    pub transitions: TransitionStates,
}

impl Default for NodeState {
    fn default() -> Self {
        Self {
            id: NodeId::new(),
            local_rect: Rect::ZERO,
            global_rect: Rect::ZERO,
            active: false,
            focused: false,
            hovered: false,
            last_draw: Instant::now(),
            style: Style::default(),
            recreated: SharedSignal::new(true),
            transitions: TransitionStates::new(),
        }
    }
}

impl NodeState {
    /// Create a new node state with the given style.
    pub fn new(style: Style) -> Self {
        Self {
            style,
            ..Default::default()
        }
    }

    /// Propagate the node state up to the parent.
    ///
    /// This is called before events are propagated.
    pub fn propagate_up(&mut self, parent: &NodeState) {
        self.global_rect = self.local_rect.translate(parent.global_rect.min);
    }

    /// Propagate the node state down to the child.
    ///
    /// This is called after events are propagated.
    pub fn propagate_down(&mut self, _child: &NodeState) {}

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
    pub fn delta(&self) -> f32 {
        self.last_draw.elapsed().as_secs_f32()
    }

    /// Transition a value.
    ///
    /// If the value is an [`f32`], or a [`Color`], then it will be transitioned.
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
        self.transitions.update(self.delta())
    }

    /// Updates `self.last_draw` to the current time.
    fn draw(&mut self) {
        self.last_draw = Instant::now();
    }
}

impl<T: View> From<T> for Node {
    fn from(view: T) -> Self {
        Self::new(view)
    }
}

#[cfg(feature = "multi-thread")]
type AnyViewState = Box<dyn Any + Send + Sync>;
#[cfg(not(feature = "multi-thread"))]
type AnyViewState = Box<dyn Any>;

struct NodeInner {
    view_state: Lock<AnyViewState>,
    node_state: Lock<NodeState>,
    view: Box<dyn AnyView>,
}

impl NodeInner {
    fn view_state(&self) -> Guard<AnyViewState> {
        self.view_state.lock_mut()
    }

    fn node_state(&self) -> Guard<NodeState> {
        self.node_state.lock_mut()
    }
}

impl Debug for NodeInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodeInner")
            .field("view_state", &self.view_state)
            .field("node_state", &self.node_state)
            .field("view", &self.view.type_id())
            .finish()
    }
}

/// A node in the ui tree.
///
/// A node is a wrapper around a view, and contains the state of the view.
#[derive(Clone, Debug)]
pub struct Node {
    inner: Shared<NodeInner>,
}

impl Node {
    /// Create a new empty node.
    pub fn empty() -> Self {
        Self::new(())
    }

    /// Create a new node with the given [`View`].
    pub fn new(view: impl View) -> Self {
        let view_state = Box::new(View::build(&view));
        let node_state = NodeState::new(View::style(&view));

        Self {
            inner: Shared::new(NodeInner {
                view_state: Lock::new(view_state),
                node_state: Lock::new(node_state),
                view: Box::new(view),
            }),
        }
    }

    /// Returns a [`Guard`] to the [`AnyViewState`].
    ///
    /// Be careful when using this, as it can cause deadlocks.
    pub fn view_state(&self) -> Guard<AnyViewState> {
        self.inner.view_state.lock_mut()
    }

    /// Returns a [`Guard`] to the [`NodeState`].
    ///
    /// Be careful when using this, as it can cause deadlocks.
    pub fn node_state(&self) -> Guard<NodeState> {
        self.inner.node_state.lock_mut()
    }

    /// Returns a reference to the [`View`].
    pub fn get<T: View>(&self) -> Option<&T> {
        self.inner.view.downcast_ref()
    }

    /// Sets the offset of the node, relative to the parent.
    pub fn set_offset(&self, offset: Vec2) {
        let mut node_state = self.node_state();

        let size = node_state.local_rect.size();
        node_state.local_rect = Rect::min_size(offset, size);
    }

    /// Returns the [`StyleStates`].
    pub fn style_states(&self) -> StyleStates {
        self.node_state().style_states()
    }

    /// Gets the local [`Rect`] of the node.
    pub fn local_rect(&self) -> Rect {
        self.node_state().local_rect
    }

    /// Gets the global [`Rect`] of the node.
    pub fn global_rect(&self) -> Rect {
        self.node_state().global_rect
    }

    /// Gets the size of the node.
    pub fn size(&self) -> Vec2 {
        self.local_rect().size()
    }
}

impl Node {
    /// Returns true if the node should be redrawn.
    fn handle_pointer_event(
        node_state: &mut NodeState,
        event: &PointerEvent,
        is_handled: bool,
    ) -> bool {
        let is_over = node_state.global_rect.contains(event.position) && !event.left && !is_handled;
        if is_over != node_state.hovered && event.is_motion() {
            node_state.hovered = is_over;
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

    fn event_inner(inner: &NodeInner, cx: &mut EventContext, event: &Event) {
        let node_state = &mut inner.node_state();
        node_state.style = inner.view.style();
        node_state.propagate_up(cx.state);

        if let Some(pointer_event) = event.get::<PointerEvent>() {
            if Self::handle_pointer_event(node_state, pointer_event, event.is_handled()) {
                cx.request_redraw();
            }
        }

        {
            let selector = node_state.selector();
            let selectors = cx.selectors.clone().with(selector);
            let mut cx = EventContext {
                style: cx.style,
                state: node_state,
                renderer: cx.renderer,
                selectors: &selectors,
                hash: selectors.hash(),
                event_sink: cx.event_sink,
                image_cache: cx.image_cache,
                cursor: cx.cursor,
            };

            (inner.view).event(inner.view_state().as_mut(), &mut cx, event);
            Self::update_cursor(&mut cx);
        }

        cx.state.propagate_down(&node_state);
    }

    /// Handle an event.
    pub fn event(&self, cx: &mut EventContext, event: &Event) {
        Self::event_inner(&self.inner, cx, event);
    }

    fn layout_inner(inner: &NodeInner, cx: &mut LayoutContext, bc: BoxConstraints) -> Vec2 {
        let node_state = &mut inner.node_state();
        node_state.style = inner.view.style();
        node_state.propagate_up(cx.state);

        let size = {
            let selector = node_state.selector();
            let selectors = cx.selectors.clone().with(selector);
            let mut cx = LayoutContext {
                style: cx.style,
                state: node_state,
                renderer: cx.renderer,
                selectors: &selectors,
                hash: selectors.hash(),
                event_sink: cx.event_sink,
                image_cache: cx.image_cache,
                cursor: cx.cursor,
            };

            let size = inner.view.layout(inner.view_state().as_mut(), &mut cx, bc);

            Self::update_cursor(&mut cx);

            size
        };

        node_state.local_rect = Rect::min_size(node_state.local_rect.min, size);
        node_state.global_rect = Rect::min_size(node_state.global_rect.min, size);

        cx.state.propagate_down(&node_state);

        size
    }

    /// Layout the node.
    pub fn layout(&self, cx: &mut LayoutContext, bc: BoxConstraints) -> Vec2 {
        Self::layout_inner(&self.inner, cx, bc)
    }

    fn draw_inner(inner: &NodeInner, cx: &mut DrawContext) {
        let node_state = &mut inner.node_state();
        node_state.style = inner.view.style();
        node_state.propagate_up(cx.state);

        {
            let selector = node_state.selector();
            let selectors = cx.selectors.clone().with(selector);
            let mut cx = DrawContext {
                style: cx.style,
                state: node_state,
                frame: cx.frame,
                renderer: cx.renderer,
                selectors: &selectors,
                hash: selectors.hash(),
                event_sink: cx.event_sink,
                image_cache: cx.image_cache,
                cursor: cx.cursor,
            };

            inner.view.draw(inner.view_state().as_mut(), &mut cx);

            if cx.state.update_transitions() {
                cx.request_redraw();
            }

            cx.state.draw();

            Self::update_cursor(&mut cx);
        }

        cx.state.propagate_down(&node_state);
    }

    /// Draw the node.
    pub fn draw(&self, cx: &mut DrawContext) {
        Self::draw_inner(&self.inner, cx);
    }
}

impl Node {
    fn event_root_inner(
        inner: &NodeInner,
        style: &Stylesheet,
        renderer: &dyn Renderer,
        event_sink: &EventSink,
        event: &Event,
        image_cache: &mut ImageCache,
        cursor_icon: &mut Cursor,
    ) {
        let node_state = &mut inner.node_state();
        node_state.style = inner.view.style();

        if let Some(pointer_event) = event.get::<PointerEvent>() {
            if Self::handle_pointer_event(node_state, pointer_event, event.is_handled()) {
                event_sink.send(RequestRedrawEvent);
            }
        }

        let selector = node_state.selector();
        let selectors = StyleSelectors::new().with(selector);
        let mut cx = EventContext {
            style,
            state: node_state,
            renderer,
            selectors: &selectors,
            hash: selectors.hash(),
            event_sink,
            image_cache,
            cursor: cursor_icon,
        };

        (inner.view).event(inner.view_state().as_mut(), &mut cx, event);
    }

    /// Handle an event on the root node.
    pub fn event_root(
        &self,
        style: &Stylesheet,
        renderer: &dyn Renderer,
        event_sink: &EventSink,
        event: &Event,
        image_cache: &mut ImageCache,
        cursor_icon: &mut Cursor,
    ) {
        Self::event_root_inner(
            &self.inner,
            style,
            renderer,
            event_sink,
            event,
            image_cache,
            cursor_icon,
        );
    }

    fn layout_root_inner(
        inner: &NodeInner,
        style: &Stylesheet,
        renderer: &dyn Renderer,
        window_size: Vec2,
        event_sink: &EventSink,
        image_cache: &mut ImageCache,
        cursor_icon: &mut Cursor,
    ) -> Vec2 {
        let node_state = &mut inner.node_state();
        node_state.style = inner.view.style();

        let selector = node_state.selector();
        let selectors = StyleSelectors::new().with(selector);
        let mut cx = LayoutContext {
            style,
            state: node_state,
            renderer,
            selectors: &selectors,
            hash: selectors.hash(),
            event_sink,
            image_cache,
            cursor: cursor_icon,
        };

        let bc = BoxConstraints::new(Vec2::ZERO, window_size);
        let size = inner.view.layout(inner.view_state().as_mut(), &mut cx, bc);

        node_state.local_rect = Rect::min_size(node_state.local_rect.min, size);
        node_state.global_rect = Rect::min_size(node_state.global_rect.min, size);

        size
    }

    /// Layout the root node.
    pub fn layout_root(
        &self,
        style: &Stylesheet,
        renderer: &dyn Renderer,
        window_size: Vec2,
        event_sink: &EventSink,
        image_cache: &mut ImageCache,
        cursor_icon: &mut Cursor,
    ) -> Vec2 {
        Self::layout_root_inner(
            &self.inner,
            style,
            renderer,
            window_size,
            event_sink,
            image_cache,
            cursor_icon,
        )
    }

    fn draw_root_inner(
        inner: &NodeInner,
        style: &Stylesheet,
        frame: &mut Frame,
        renderer: &dyn Renderer,
        event_sink: &EventSink,
        image_cache: &mut ImageCache,
        cursor_icon: &mut Cursor,
    ) {
        let node_state = &mut inner.node_state();
        node_state.style = inner.view.style();

        let selector = node_state.selector();
        let selectors = StyleSelectors::new().with(selector);
        let mut cx = DrawContext {
            style,
            state: node_state,
            frame,
            renderer,
            selectors: &selectors,
            hash: selectors.hash(),
            event_sink,
            image_cache,
            cursor: cursor_icon,
        };

        inner.view.draw(inner.view_state().as_mut(), &mut cx);

        cx.state.draw();
    }

    /// Draw the root node.
    pub fn draw_root(
        &self,
        style: &Stylesheet,
        frame: &mut Frame,
        renderer: &dyn Renderer,
        event_sink: &EventSink,
        image_cache: &mut ImageCache,
        cursor_icon: &mut Cursor,
    ) {
        Self::draw_root_inner(
            &self.inner,
            style,
            frame,
            renderer,
            event_sink,
            image_cache,
            cursor_icon,
        );
    }
}
