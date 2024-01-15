use std::{
    any::{type_name, Any},
    future::Future,
    mem,
    ops::{Deref, DerefMut},
    time::Instant,
};

use cosmic_text::Buffer;

use crate::{
    canvas::Mesh,
    clipboard::Clipboard,
    command::{Command, CommandProxy},
    event::{CloseWindow, OpenWindow, Quit},
    layout::{Affine, Point, Rect, Size},
    text::{Fonts, TextBuffer},
    view::ViewState,
    window::{Cursor, Window, WindowDescriptor, WindowId},
};

use super::{View, ViewId};

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
        Some(*context.downcast::<T>().ok()?)
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
    pub fn clipboard(&mut self) -> &mut Clipboard {
        self.context_or_default()
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

    /// Open a new window.
    pub fn open_window<T: 'static, V: View<T> + 'static>(
        &mut self,
        desc: WindowDescriptor,
        ui: impl FnMut(&mut T) -> V + Send + 'static,
    ) {
        let mut cmd = OpenWindow::new(ui);
        cmd.desc = desc;

        self.cmd(cmd);
    }

    /// Close the window.
    pub fn close_window(&mut self, id: WindowId) {
        self.cmd(CloseWindow::new(id));
    }
}

/// A context for building the view tree.
pub struct BuildCx<'a, 'b> {
    pub(crate) base: &'a mut BaseCx<'b>,
    pub(crate) window: &'a mut Window,
    pub(crate) animation_frame: &'a mut Option<Instant>,
}

impl<'a, 'b> BuildCx<'a, 'b> {
    pub(crate) fn new(
        base: &'a mut BaseCx<'b>,
        window: &'a mut Window,
        animation_frame: &'a mut Option<Instant>,
    ) -> Self {
        Self {
            base,
            window,
            animation_frame,
        }
    }

    /// Create a child context.
    pub fn child(&mut self) -> BuildCx<'_, 'b> {
        BuildCx {
            base: self.base,
            window: self.window,
            animation_frame: self.animation_frame,
        }
    }

    /// Request a rebuild of the view tree.
    pub fn request_rebuild(&mut self) {
        self.base.request_rebuild();
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
        BuildCx::new(self.base, self.window, self.animation_frame)
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
        BuildCx::new(self.base, self.window, self.animation_frame)
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

    /// Create a mesh for the given text buffer.
    pub fn rasterize_text(&mut self, buffer: &TextBuffer, rect: Rect) -> Mesh {
        self.fonts().rasterize_text(buffer.raw(), rect)
    }

    /// Create a mesh for the given raw cosmic text buffer.
    pub fn rasterize_text_raw(&mut self, buffer: &Buffer, rect: Rect) -> Mesh {
        self.fonts().rasterize_text(buffer, rect)
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
}

impl_deref!(BuildCx);
impl_deref!(RebuildCx);
impl_deref!(EventCx);
impl_deref!(LayoutCx);
impl_deref!(DrawCx);

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
}}

impl_context! {RebuildCx<'_, '_>, EventCx<'_, '_>, LayoutCx<'_, '_>, DrawCx<'_, '_> {
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

    /// Set the cursor of the view.
    pub fn set_cursor(&mut self, cursor: impl Into<Option<Cursor>>) {
        self.view_state.cursor = cursor.into();
    }

    /// Get whether the a child view has the cursor.
    pub fn has_cursor(&self) -> bool {
        self.view_state.has_cursor()
    }

    /// Get whether the view has soft input.
    pub fn has_soft_input(&self) -> bool {
        self.view_state.has_soft_input()
    }

    /// Set whether the view has soft input.
    pub fn set_soft_input(&mut self, soft_input: bool) {
        self.view_state.soft_input = soft_input;
    }

    pub(crate) fn update(&mut self) {
        match self.view_state.cursor {
            Some(cursor) if !self.view_state.has_cursor() => {
                self.window.set_cursor(cursor);
            }
            _ => {}
        }
    }

    /// Get the flex grow of the view.
    pub fn flex_grow(&self) -> f32 {
        self.view_state.flex_grow()
    }

    /// Get the flex shrink of the view.
    pub fn flex_shrink(&self) -> f32 {
        self.view_state.flex_shrink()
    }

    /// Set the flex grow of the view.
    pub fn set_flex_grow(&mut self, flex: f32) {
        self.view_state.set_flex_grow(flex);
    }

    /// Set the flex shrink of the view.
    pub fn set_flex_shrink(&mut self, flex: f32) {
        self.view_state.set_flex_shrink(flex);
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
