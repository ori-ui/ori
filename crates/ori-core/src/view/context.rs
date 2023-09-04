use std::{any::Any, time::Instant};

use glam::Vec2;

use crate::{
    canvas::Mesh,
    command::{Command, CommandProxy},
    layout::{Affine, Rect, Size},
    text::{Fonts, Glyphs, TextSection},
    view::ViewState,
    window::{Cursor, Window},
};

/// A base context that is shared between all other contexts.
pub struct BaseCx<'a> {
    pub(crate) fonts: &'a mut Fonts,
    pub(crate) proxy: &'a mut CommandProxy,
    pub(crate) needs_rebuild: &'a mut bool,
}

impl<'a> BaseCx<'a> {
    /// Create a new base context.
    pub fn new(
        fonts: &'a mut Fonts,
        proxy: &'a mut CommandProxy,
        needs_rebuild: &'a mut bool,
    ) -> Self {
        Self {
            fonts,
            proxy,
            needs_rebuild,
        }
    }

    /// Get the [`Fonts`].
    pub fn fonts(&mut self) -> &mut Fonts {
        self.fonts
    }

    /// Get the [`CommandProxy`].
    pub fn proxy(&self) -> CommandProxy {
        self.proxy.clone()
    }

    /// Emit a command.
    pub fn cmd<T: Any + Send>(&mut self, command: T) {
        self.proxy.send_internal(Command::new(command));
    }

    /// Request a rebuild of the view tree.
    pub fn request_rebuild(&mut self) {
        *self.needs_rebuild = true;
    }
}

/// A context for building the view tree.
pub struct BuildCx<'a, 'b> {
    pub(crate) base: &'a mut BaseCx<'b>,
    pub(crate) window: &'a mut Window,
}

impl<'a, 'b> BuildCx<'a, 'b> {
    pub(crate) fn new(base: &'a mut BaseCx<'b>, window: &'a mut Window) -> Self {
        Self { base, window }
    }

    /// Create a child context.
    pub fn child(&mut self) -> BuildCx<'_, 'b> {
        BuildCx {
            base: self.base,
            window: self.window,
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
        BuildCx::new(self.base, self.window)
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
    pub fn local(&self, point: Vec2) -> Vec2 {
        self.transform.inverse() * point
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

    /// Create a mesh for the given glyphs.
    pub fn text_mesh(&mut self, glyphs: &Glyphs, rect: Rect) -> Option<Mesh> {
        self.base.fonts.text_mesh(glyphs, rect)
    }
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

impl_context! {EventCx<'_, '_>, DrawCx<'_, '_> {
    /// Get the size of the view.
    pub fn size(&self) -> Size {
        self.view_state.size
    }

    /// Get the rect of the view in local space.
    pub fn rect(&self) -> Rect {
        Rect::min_size(Vec2::ZERO, self.size())
    }
}}

impl_context! {BuildCx<'_, '_>, RebuildCx<'_, '_>, EventCx<'_, '_>, LayoutCx<'_, '_>, DrawCx<'_, '_> {
    /// Get the fonts.
    pub fn fonts(&mut self) -> &mut Fonts {
        self.base.fonts()
    }

    /// Get the window.
    pub fn window(&mut self) -> &mut Window {
        self.window
    }

    /// Get a proxy for sending commands.
    pub fn proxy(&self) -> CommandProxy {
        self.base.proxy()
    }

    /// Emit a command.
    pub fn cmd<T: Any + Send>(&mut self, command: T) {
        self.base.cmd(command);
    }
}}

impl_context! {RebuildCx<'_, '_>, EventCx<'_, '_>, LayoutCx<'_, '_>, DrawCx<'_, '_> {
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

    pub(crate) fn update(&mut self) {
        match self.view_state.cursor {
            Some(cursor) if !self.view_state.has_cursor() => {
                self.window.set_cursor(cursor);
            }
            _ => {}
        }
    }

    /// Get the flex of the view.
    pub fn flex(&self) -> f32 {
        self.view_state.flex
    }

    /// Set the flex of the view.
    pub fn set_flex(&mut self, flex: f32) {
        self.view_state.set_flex(flex);
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

    /// Request an animation frame.
    pub fn request_animation_frame(&mut self) {
        if self.animation_frame.is_none() {
            *self.animation_frame = Some(Instant::now());
        }
    }

    /// Layout the given [`TextSection`].
    pub fn layout_text(&mut self, text: &TextSection<'_>) -> Option<Glyphs> {
        self.base.fonts.layout_text(text)
    }
}}
