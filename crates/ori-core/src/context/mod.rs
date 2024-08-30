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

use crate::{view::ViewId, window::Window};

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
        self.context_mut()
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

    /// Check if the view has the property `T`.
    pub fn contains_property<T: 'static>(&self) -> bool {
        self.view_state.contains_property::<T>()
    }

    /// Insert a property into the view.
    pub fn insert_property<T: 'static>(&mut self, item: T) {
        self.view_state.insert_property(item);
    }

    /// Remove a property from the view.
    pub fn remove_property<T: 'static>(&mut self) -> Option<T> {
        self.view_state.remove_property()
    }

    /// Get the property `T` of the view.
    pub fn get_property<T: 'static>(&self) -> Option<&T> {
        self.view_state.get_property()
    }

    /// Get the property `T` of the view mutably.
    pub fn get_property_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.view_state.get_property_mut()
    }

    /// Get the property `T` of the view or insert it with a value.
    pub fn property_or_insert_with<T: 'static, F: FnOnce() -> T>(&mut self, f: F) -> &mut T {
        self.view_state.property_or_insert_with(f)
    }

    /// Get the property `T` of the view or insert it with a value.
    pub fn property_or<T: 'static>(&mut self, item: T) -> &mut T {
        self.view_state.property_or(item)
    }

    /// Get the property `T` of the view or insert it with a default value.
    pub fn property_or_default<T: 'static + Default>(&mut self) -> &mut T {
        self.view_state.property_or_default()
    }
}}
