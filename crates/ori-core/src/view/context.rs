use std::{
    any::{type_name, Any},
    future::Future,
    mem,
    ops::{Deref, DerefMut},
};

use cosmic_text::Buffer;
use instant::Instant;

use crate::{
    canvas::Mesh,
    clipboard::{Clipboard, ClipboardContext},
    command::{Command, CommandProxy},
    event::Quit,
    layout::{Affine, Point, Rect, Size},
    text::{Fonts, TextBuffer},
    ui::{UiBuilder, UiRequest, UiRequests},
    view::ViewState,
    window::{Window, WindowDescriptor, WindowId},
};

use super::{any, AnyView, ViewFlags, ViewId};

/// A context for a view.
#[derive(Debug, Default)]
pub struct Contexts {
    contexts: Vec<Box<dyn Any>>,
}

impl Contexts {
    /// Create a new context.
    pub fn new() -> Self {
        Self::default()
    }

    fn index_of<T: Any>(&self) -> Option<usize> {
        self.contexts
            .iter()
            .enumerate()
            .find(|(_, c)| c.as_ref().is::<T>())
            .map(|(i, _)| i)
    }

    /// Get the number of contexts.
    pub fn len(&self) -> usize {
        self.contexts.len()
    }

    /// Check if there are no contexts.
    pub fn is_empty(&self) -> bool {
        self.contexts.is_empty()
    }

    /// Check if the context is present.
    pub fn contains<T: Any>(&self) -> bool {
        self.index_of::<T>().is_some()
    }

    /// Push a context.
    pub fn insert<T: Any>(&mut self, mut context: T) -> Option<T> {
        if let Some(index) = self.get_mut::<T>() {
            mem::swap(index, &mut context);
            return Some(context);
        }

        self.contexts.push(Box::new(context));

        None
    }

    /// Pop a context.
    pub fn remove<T: Any>(&mut self) -> Option<T> {
        let index = self.index_of::<T>()?;

        let context = self.contexts.remove(index);
        Some(*context.downcast::<T>().expect("downcast failed"))
    }

    /// Get a context.
    pub fn get<T: Any>(&self) -> Option<&T> {
        let index = self.index_of::<T>()?;
        self.contexts[index].downcast_ref::<T>()
    }

    /// Get a mutable context.
    pub fn get_mut<T: Any>(&mut self) -> Option<&mut T> {
        let index = self.index_of::<T>()?;
        self.contexts[index].downcast_mut::<T>()
    }

    /// Get a context or insert a `default`.
    pub fn get_or_default<T: Any + Default>(&mut self) -> &mut T {
        if !self.contains::<T>() {
            self.insert(T::default());
        }

        self.get_mut::<T>().unwrap()
    }
}

/// A base context that is shared between all other contexts.
pub struct BaseCx<'a> {
    pub(crate) contexts: &'a mut Contexts,
    pub(crate) proxy: &'a mut CommandProxy,
    pub(crate) needs_rebuild: &'a mut bool,
}

impl<'a> BaseCx<'a> {
    /// Create a new base context.
    pub fn new(
        contexts: &'a mut Contexts,
        proxy: &'a mut CommandProxy,
        needs_rebuild: &'a mut bool,
    ) -> Self {
        Self {
            contexts,
            proxy,
            needs_rebuild,
        }
    }

    /// Get the [`Fonts`].
    pub fn fonts(&mut self) -> &mut Fonts {
        self.context_or_default()
    }

    /// Get the [`Clipboard`].
    pub fn clipboard(&mut self) -> &mut dyn Clipboard {
        self.context_or_default::<ClipboardContext>()
    }

    /// Get the [`CommandProxy`].
    pub fn proxy(&self) -> CommandProxy {
        self.proxy.clone()
    }

    /// Emit a command.
    pub fn cmd<T: Any + Send>(&mut self, command: T) {
        self.proxy.cmd_silent(Command::new(command));
    }

    /// Spawn a future.
    pub fn spawn_async(&mut self, future: impl Future<Output = ()> + Send + 'static) {
        self.proxy.spawn_async(future);
    }

    /// Spawn a future sending a command when it completes.
    pub fn cmd_async<T: Any + Send>(&self, future: impl Future<Output = T> + Send + 'static) {
        self.proxy.cmd_async(future);
    }

    /// Get a reference to the [`Contexts`].
    pub fn contexts(&self) -> &Contexts {
        self.contexts
    }

    /// Get a mutable reference to the [`Contexts`].
    pub fn contexts_mut(&mut self) -> &mut Contexts {
        self.contexts
    }

    /// Insert a context.
    pub fn insert_context<T: Any>(&mut self, context: T) -> Option<T> {
        self.contexts.insert(context)
    }

    /// Check if a context is contained.
    pub fn contains_context<T: Any>(&self) -> bool {
        self.contexts.contains::<T>()
    }

    /// Remove a context.
    pub fn remove_context<T: Any>(&mut self) -> Option<T> {
        self.contexts.remove::<T>()
    }

    /// Get a context.
    pub fn get_context<T: Any>(&self) -> Option<&T> {
        self.contexts.get::<T>()
    }

    /// Get a mutable context.
    pub fn get_context_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.contexts.get_mut::<T>()
    }

    /// Get a context.
    ///
    /// # Panics
    /// - If the context is not found.
    #[track_caller]
    pub fn context<T: Any>(&self) -> &T {
        match self.get_context::<T>() {
            Some(context) => context,
            None => panic!("context not found: {}", type_name::<T>()),
        }
    }

    /// Get a mutable context.
    ///
    /// # Panics
    /// - If the context is not found.
    #[track_caller]
    pub fn context_mut<T: Any>(&mut self) -> &mut T {
        match self.get_context_mut::<T>() {
            Some(context) => context,
            None => panic!("context not found: {}", type_name::<T>()),
        }
    }

    /// Get a context or insert a `default`.
    pub fn context_or_default<T: Any + Default>(&mut self) -> &mut T {
        self.contexts.get_or_default::<T>()
    }

    /// Request a rebuild of the view tree.
    pub fn request_rebuild(&mut self) {
        *self.needs_rebuild = true;
    }

    /// Quit the application.
    pub fn quit(&mut self) {
        self.cmd(Quit);
    }
}

/// A context for a [`Delegate`](crate::delegate::Delegate).
pub struct DelegateCx<'a, 'b, T> {
    pub(crate) base: &'a mut BaseCx<'b>,
    pub(crate) requests: &'a mut UiRequests<T>,
}

impl<'a, 'b, T> DelegateCx<'a, 'b, T> {
    pub(crate) fn new(base: &'a mut BaseCx<'b>, requests: &'a mut UiRequests<T>) -> Self {
        Self { base, requests }
    }

    /// Open a new window.
    pub fn open_window<V: AnyView<T> + 'static>(
        &mut self,
        desc: WindowDescriptor,
        mut ui: impl FnMut(&mut T) -> V + Send + 'static,
    ) {
        let builder: UiBuilder<T> = Box::new(move |data| any(ui(data)));
        self.requests.push(UiRequest::CreateWindow(desc, builder));
    }

    /// Close the window.
    pub fn close_window(&mut self, id: WindowId) {
        self.requests.push(UiRequest::RemoveWindow(id));
    }
}

impl<'a, 'b, T> Deref for DelegateCx<'a, 'b, T> {
    type Target = BaseCx<'b>;

    fn deref(&self) -> &Self::Target {
        self.base
    }
}

impl<'a, 'b, T> DerefMut for DelegateCx<'a, 'b, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.base
    }
}

/// A context for building the view tree.
pub struct BuildCx<'a, 'b> {
    pub(crate) base: &'a mut BaseCx<'b>,
    pub(crate) view_state: &'a mut ViewState,
    pub(crate) window: &'a mut Window,
    pub(crate) animation_frame: &'a mut Option<Instant>,
}

impl<'a, 'b> BuildCx<'a, 'b> {
    pub(crate) fn new(
        base: &'a mut BaseCx<'b>,
        view_state: &'a mut ViewState,
        window: &'a mut Window,
        animation_frame: &'a mut Option<Instant>,
    ) -> Self {
        Self {
            base,
            view_state,
            window,
            animation_frame,
        }
    }

    /// Create a child context.
    pub fn child(&mut self) -> BuildCx<'_, 'b> {
        BuildCx {
            base: self.base,
            view_state: self.view_state,
            window: self.window,
            animation_frame: self.animation_frame,
        }
    }
}

/// A context for rebuilding the view tree.
pub struct RebuildCx<'a, 'b> {
    pub(crate) base: &'a mut BaseCx<'b>,
    pub(crate) view_state: &'a mut ViewState,
    pub(crate) window: &'a mut Window,
    pub(crate) animation_frame: &'a mut Option<Instant>,
}

impl<'a, 'b> RebuildCx<'a, 'b> {
    pub(crate) fn new(
        base: &'a mut BaseCx<'b>,
        view_state: &'a mut ViewState,
        window: &'a mut Window,
        animation_frame: &'a mut Option<Instant>,
    ) -> Self {
        Self {
            base,
            view_state,
            window,
            animation_frame,
        }
    }

    /// Create a child context.
    pub fn child(&mut self) -> RebuildCx<'_, 'b> {
        RebuildCx {
            base: self.base,
            view_state: self.view_state,
            window: self.window,
            animation_frame: self.animation_frame,
        }
    }

    /// Get a build context.
    pub fn build_cx(&mut self) -> BuildCx<'_, 'b> {
        BuildCx::new(
            self.base,
            self.view_state,
            self.window,
            self.animation_frame,
        )
    }

    /// Get a layout context.
    pub fn layout_cx(&mut self) -> LayoutCx<'_, 'b> {
        LayoutCx::new(
            self.base,
            self.view_state,
            self.window,
            self.animation_frame,
        )
    }
}

/// A context for handling events.
pub struct EventCx<'a, 'b> {
    pub(crate) base: &'a mut BaseCx<'b>,
    pub(crate) view_state: &'a mut ViewState,
    pub(crate) window: &'a mut Window,
    pub(crate) animation_frame: &'a mut Option<Instant>,
    pub(crate) transform: Affine,
}

impl<'a, 'b> EventCx<'a, 'b> {
    pub(crate) fn new(
        base: &'a mut BaseCx<'b>,
        view_state: &'a mut ViewState,
        window: &'a mut Window,
        animation_frame: &'a mut Option<Instant>,
    ) -> Self {
        let transform = view_state.transform;

        Self {
            base,
            view_state,
            window,
            animation_frame,
            transform,
        }
    }

    /// Create a child context.
    pub fn child(&mut self) -> EventCx<'_, 'b> {
        EventCx {
            base: self.base,
            view_state: self.view_state,
            window: self.window,
            animation_frame: self.animation_frame,
            transform: self.transform,
        }
    }

    /// Get the transform of the view.
    pub fn transform(&self) -> Affine {
        self.transform
    }

    /// Transform a point from global space to local space.
    pub fn local(&self, point: Point) -> Point {
        self.transform.inverse() * point
    }

    /// Get a build context.
    pub fn build_cx(&mut self) -> BuildCx<'_, 'b> {
        BuildCx::new(
            self.base,
            self.view_state,
            self.window,
            self.animation_frame,
        )
    }

    /// Get a rebuild context.
    pub fn rebuild_cx(&mut self) -> RebuildCx<'_, 'b> {
        RebuildCx::new(
            self.base,
            self.view_state,
            self.window,
            self.animation_frame,
        )
    }

    /// Get whether the view was hot last call.
    pub fn was_hot(&self) -> bool {
        self.view_state.prev_flags.contains(ViewFlags::HOT)
    }

    /// Get whether the view was focused last call.
    pub fn was_focused(&self) -> bool {
        self.view_state.prev_flags.contains(ViewFlags::FOCUSED)
    }

    /// Get whether the view was active last call.
    pub fn was_active(&self) -> bool {
        self.view_state.prev_flags.contains(ViewFlags::ACTIVE)
    }

    /// Get whether a child view was hot last call.
    pub fn had_hot(&self) -> bool {
        self.view_state.prev_flags.contains(ViewFlags::HAS_HOT)
    }

    /// Get whether a child view was focused last call.
    pub fn had_focused(&self) -> bool {
        self.view_state.prev_flags.contains(ViewFlags::HAS_FOCUSED)
    }

    /// Get whether a child view was active last call.
    pub fn had_active(&self) -> bool {
        self.view_state.prev_flags.contains(ViewFlags::HAS_ACTIVE)
    }

    /// Get whether the view's hot state changed.
    pub fn hot_changed(&self) -> bool {
        self.was_hot() != self.is_hot()
    }

    /// Get whether the view's focused state changed.
    pub fn focused_changed(&self) -> bool {
        self.was_focused() != self.is_focused()
    }

    /// Get whether the view's active state changed.
    pub fn active_changed(&self) -> bool {
        self.was_active() != self.is_active()
    }

    /// Get whether a child view's hot state changed.
    pub fn has_hot_changed(&self) -> bool {
        self.had_hot() != self.has_hot()
    }

    /// Get whether a child view's focused state changed.
    pub fn has_focused_changed(&self) -> bool {
        self.had_focused() != self.has_focused()
    }

    /// Get whether a child view's active state changed.
    pub fn has_active_changed(&self) -> bool {
        self.had_active() != self.has_active()
    }
}

/// A context for laying out the view tree.
pub struct LayoutCx<'a, 'b> {
    pub(crate) base: &'a mut BaseCx<'b>,
    pub(crate) view_state: &'a mut ViewState,
    pub(crate) window: &'a mut Window,
    pub(crate) animation_frame: &'a mut Option<Instant>,
}

impl<'a, 'b> LayoutCx<'a, 'b> {
    pub(crate) fn new(
        base: &'a mut BaseCx<'b>,
        view_state: &'a mut ViewState,
        window: &'a mut Window,
        animation_frame: &'a mut Option<Instant>,
    ) -> Self {
        Self {
            base,
            view_state,
            window,
            animation_frame,
        }
    }

    /// Create a child context.
    pub fn child(&mut self) -> LayoutCx<'_, 'b> {
        LayoutCx {
            base: self.base,
            view_state: self.view_state,
            window: self.window,
            animation_frame: self.animation_frame,
        }
    }

    /// Get a rebuild context.
    pub fn build_cx(&mut self) -> BuildCx<'_, 'b> {
        BuildCx::new(
            self.base,
            self.view_state,
            self.window,
            self.animation_frame,
        )
    }

    /// Get a rebuild context.
    pub fn rebuild_cx(&mut self) -> RebuildCx<'_, 'b> {
        RebuildCx::new(
            self.base,
            self.view_state,
            self.window,
            self.animation_frame,
        )
    }
}

/// A context for drawing the view tree.
pub struct DrawCx<'a, 'b> {
    pub(crate) base: &'a mut BaseCx<'b>,
    pub(crate) view_state: &'a mut ViewState,
    pub(crate) window: &'a mut Window,
    pub(crate) animation_frame: &'a mut Option<Instant>,
}

impl<'a, 'b> DrawCx<'a, 'b> {
    pub(crate) fn new(
        base: &'a mut BaseCx<'b>,
        view_state: &'a mut ViewState,
        window: &'a mut Window,
        animation_frame: &'a mut Option<Instant>,
    ) -> Self {
        Self {
            base,
            view_state,
            window,
            animation_frame,
        }
    }

    /// Create a child context.
    pub fn layer(&mut self) -> DrawCx<'_, 'b> {
        DrawCx {
            base: self.base,
            view_state: self.view_state,
            window: self.window,
            animation_frame: self.animation_frame,
        }
    }

    /// Get a rebuild context.
    pub fn rebuild_cx(&mut self) -> RebuildCx<'_, 'b> {
        RebuildCx::new(
            self.base,
            self.view_state,
            self.window,
            self.animation_frame,
        )
    }
}

macro_rules! impl_deref {
    ($ident:ident) => {
        impl<'a, 'b> Deref for $ident<'a, 'b> {
            type Target = BaseCx<'b>;

            fn deref(&self) -> &Self::Target {
                self.base
            }
        }

        impl<'a, 'b> DerefMut for $ident<'a, 'b> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                self.base
            }
        }
    };
    ($($ident:ident),* $(,)?) => {
        $(impl_deref!($ident);)*
    };
}

impl_deref! {
    BuildCx,
    RebuildCx,
    EventCx,
    LayoutCx,
    DrawCx,
}

macro_rules! impl_context {
    ($ty:ty { $($impl:item)* }) => {
        impl $ty {
            $($impl)*
        }
    };
    ($ty:ty, $($mode:ty),* { $($impl:item)* }) => {
        impl_context!($ty { $($impl)* });
        impl_context!($($mode),* { $($impl)* });
    };
}

impl_context! {RebuildCx<'_, '_>, EventCx<'_, '_>, DrawCx<'_, '_> {
    /// Get the size of the view.
    pub fn size(&self) -> Size {
        self.view_state.size
    }

    /// Get the rect of the view in local space.
    pub fn rect(&self) -> Rect {
        Rect::min_size(Point::ZERO, self.size())
    }
}}

impl_context! {BuildCx<'_, '_>, RebuildCx<'_, '_>, EventCx<'_, '_>, LayoutCx<'_, '_>, DrawCx<'_, '_> {
    /// Request an animation frame.
    pub fn request_animation_frame(&mut self) {
        if self.animation_frame.is_none() {
            *self.animation_frame = Some(Instant::now());
        }
    }

    /// Get the window.
    pub fn window(&mut self) -> &mut Window {
        self.window
    }

    /// Prepare a text buffer for rasterization.
    pub fn prepare_text(&mut self, buffer: &TextBuffer) {
        self.fonts().prepare_text(buffer.raw());
    }

    /// Prepare a raw cosmic text buffer for rasterization.
    pub fn prepare_text_raw(&mut self, buffer: &Buffer) {
        self.fonts().prepare_text(buffer);
    }

    /// Create a mesh for the given text buffer.
    pub fn rasterize_text(&mut self, buffer: &TextBuffer) -> Mesh {
        self.fonts().rasterize_text(buffer.raw())
    }

    /// Create a mesh for the given raw cosmic text buffer.
    pub fn rasterize_text_raw(&mut self, buffer: &Buffer) -> Mesh {
        self.fonts().rasterize_text(buffer)
    }

    /// Get the id of the view.
    pub fn id(&self) -> ViewId {
        self.view_state.id()
    }

    /// Get whether the view is hot.
    pub fn is_hot(&self) -> bool {
        self.view_state.is_hot()
    }

    /// Set whether the view is hot.
    ///
    /// Returns `true` if the hot state changed.
    pub fn set_hot(&mut self, hot: bool) -> bool {
        let updated = self.is_hot() != hot;
        self.view_state.set_hot(hot);
        updated
    }

    /// Get whether the view is focused.
    pub fn is_focused(&self) -> bool {
        self.view_state.is_focused()
    }

    /// Set whether the view is focused.
    ///
    /// Returns `true` if the focused state changed.
    pub fn set_focused(&mut self, focused: bool) -> bool {
        let updated = self.is_focused() != focused;
        self.view_state.set_focused(focused);
        updated
    }

    /// Get whether the view is active.
    pub fn is_active(&self) -> bool {
        self.view_state.is_active()
    }

    /// Set whether the view is active.
    ///
    /// Returns `true` if the active state changed.
    pub fn set_active(&mut self, active: bool) -> bool {
        let updated = self.is_active() != active;
        self.view_state.set_active(active);
        updated
    }

    /// Get whether a child view is hot.
    pub fn has_hot(&self) -> bool {
        self.view_state.has_hot()
    }

    /// Get whether a child view is focused.
    pub fn has_focused(&self) -> bool {
        self.view_state.has_focused()
    }

    /// Get whether a child view is active.
    pub fn has_active(&self) -> bool {
        self.view_state.has_active()
    }

    /// Get the flex of the view.
    pub fn flex(&self) -> f32 {
        self.view_state.flex()
    }

    /// Set the flex of the view.
    pub fn set_flex(&mut self, flex: f32) {
        self.view_state.set_flex(flex);
    }

    /// Get whether the view is tight.
    pub fn is_tight(&self) -> bool {
        self.view_state.is_tight()
    }

    /// Set whether the view is tight.
    pub fn set_tight(&mut self, tight: bool) {
        self.view_state.set_tight(tight);
    }

    /// Request a rebuild of the view tree.
    pub fn request_rebuild(&mut self) {
        self.base.request_rebuild();
    }

    /// Request a layout of the view tree.
    pub fn request_layout(&mut self) {
        self.view_state.request_layout();
    }

    /// Request a draw of the view tree.
    pub fn request_draw(&mut self) {
        self.view_state.request_draw();
    }
}}
