//! Contexts for views.

mod base;
mod build;
mod contexts;
mod draw;
mod event;
mod layout;
mod rebuild;

pub use base::*;
pub use build::*;
pub use contexts::*;
pub use draw::*;
pub use event::*;
pub use layout::*;
pub use rebuild::*;

use cosmic_text::Buffer;

use crate::{canvas::Mesh, text::TextBuffer, view::ViewId, window::Window};

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

impl_context! {BuildCx<'_, '_>, RebuildCx<'_, '_>, EventCx<'_, '_>, LayoutCx<'_, '_>, DrawCx<'_, '_> {
    /// Get the window.
    pub fn window(&mut self) -> &mut Window {
        self.window
    }

    /// Prepare a text buffer for rasterization.
    pub fn prepare_text(&mut self, buffer: &TextBuffer) {
        let scale = self.window().scale;
        self.fonts().prepare_text(buffer.raw(), scale);
    }

    /// Prepare a raw cosmic text buffer for rasterization.
    pub fn prepare_text_raw(&mut self, buffer: &Buffer) {
        let scale = self.window().scale;
        self.fonts().prepare_text(buffer, scale);
    }

    /// Create a mesh for the given text buffer.
    pub fn rasterize_text(&mut self, buffer: &TextBuffer) -> Mesh {
        let scale = self.window().scale;
        self.fonts().rasterize_text(buffer.raw(), scale)
    }

    /// Create a mesh for the given raw cosmic text buffer.
    pub fn rasterize_text_raw(&mut self, buffer: &Buffer) -> Mesh {
        let scale = self.window().scale;
        self.fonts().rasterize_text(buffer, scale)
    }

    /// Get the id of the view.
    pub fn id(&self) -> ViewId {
        self.view_state.id()
    }

    /// Get whether the view is hot.
    pub fn is_hot(&self) -> bool {
        self.view_state.is_hot()
    }

    /// Get whether the view is focused.
    pub fn is_focused(&self) -> bool {
        self.view_state.is_focused()
    }

    /// Get whether the view is active.
    pub fn is_active(&self) -> bool {
        self.view_state.is_active()
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

    /// Get whether the view is tight.
    pub fn is_tight(&self) -> bool {
        self.view_state.is_tight()
    }
}}
