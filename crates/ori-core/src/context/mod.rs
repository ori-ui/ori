//! Contexts for views.

mod base;
mod build;
mod contexts;
mod draw;
mod event;
mod layout;
mod rebuild;

use std::any::Any;

pub use base::*;
pub use build::*;
pub use contexts::*;
pub use draw::*;
pub use event::*;
pub use layout::*;
pub use rebuild::*;

use crate::{
    event::{Ime, RequestFocus, RequestFocusNext, RequestFocusPrev},
    style::{Style, Styles, Theme},
    view::{ViewId, ViewState},
    window::{Cursor, Window},
};

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
    pub fn window(&self) -> &Window {
        self.context()
    }

    /// Get the window mutably.
    pub fn window_mut(&mut self) -> &mut Window {
        self.context_mut()
    }

    /// Get the styles.
    pub fn styles(&mut self) -> &mut Styles {
        self.context_mut()
    }

    /// Get the style `T`.
    pub fn style<T: Style + Any>(&mut self) -> &T {
        self.styles().style::<T>()
    }

    /// Get the [`Theme`] of the context.
    pub fn theme(&mut self) -> &Theme {
        self.style()
    }

    /// Get the id of the view.
    pub fn id(&self) -> ViewId {
        self.view_state.id()
    }

    /// Get whether the view is hovered.
    pub fn is_hovered(&self) -> bool {
        self.view_state.is_hovered()
    }

    /// Get whether the view is focused.
    pub fn is_focused(&self) -> bool {
        self.view_state.is_focused()
    }

    /// Get whether the view is active.
    pub fn is_active(&self) -> bool {
        self.view_state.is_active()
    }


    /// Get whether the view is focusable.
    pub fn is_focusable(&self) -> bool {
        self.view_state.is_focusable()
    }

    /// Get whether a child view is hovered.
    pub fn has_hovered(&self) -> bool {
        self.view_state.has_hovered()
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

impl_context! {BuildCx<'_, '_>, RebuildCx<'_, '_>, EventCx<'_, '_> {
    /// Propagate the view state of a child view.
    pub fn propagate(&mut self, child: &mut ViewState) {
        self.view_state.propagate(child);
    }

    /// Request a layout of the view tree.
    pub fn layout(&mut self) {
        self.view_state.request_layout();
    }

    /// Request a draw of the view tree.
    pub fn draw(&mut self) {
        self.view_state.request_draw();
    }

    /// Request an animation frame.
    pub fn animate(&mut self) {
        self.view_state.request_animate();
    }

    /// Request focus for the view.
    pub fn focus(&mut self) {
        if !self.is_focused() {
            let cmd = RequestFocus(self.window().id(), self.id());
            self.cmd(cmd);
        }
    }

    /// Request the next focusable view to be focused.
    pub fn focus_next(&mut self) {
        let cmd = RequestFocusNext(self.window().id());
        self.cmd(cmd);
    }

    /// Request the previous focusable view to be focused.
    pub fn focus_prev(&mut self) {
        let cmd = RequestFocusPrev(self.window().id());
        self.cmd(cmd);
    }

    /// Set the cursor of the view.
    pub fn set_cursor(&mut self, cursor: Option<Cursor>) {
        self.view_state.set_cursor(cursor);
    }

    /// Set the ime of the view.
    pub fn set_ime(&mut self, ime: Option<Ime>) {
        self.view_state.set_ime(ime);
    }

    /// Set whether the view is hovered.
    ///
    /// Returns `true` if the hovered state changed.
    pub fn set_hovered(&mut self, hovered: bool) -> bool {
        let updated = self.is_hovered() != hovered;
        self.view_state.set_hovered(hovered);
        updated
    }

    /// Set whether the view is focused.
    ///
    /// Returns `true` if the focused state changed.
    pub fn set_focused(&mut self, focused: bool) -> bool {
        let updated = self.is_focused() != focused;
        self.view_state.set_focused(focused);
        updated
    }

    /// Set whether the view is active.
    ///
    /// Returns `true` if the active state changed.
    pub fn set_active(&mut self, active: bool) -> bool {
        let updated = self.is_active() != active;
        self.view_state.set_active(active);
        updated
    }

    /// Set whether the view is focusable.
    ///
    /// Returns `true` if the focusable state changed.
    pub fn set_focusable(&mut self, focusable: bool) -> bool {
        let updated = self.is_focusable() != focusable;
        self.view_state.set_focusable(focusable);
        updated
    }
}}
