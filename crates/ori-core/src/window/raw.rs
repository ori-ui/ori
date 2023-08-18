use std::fmt::Debug;

use crate::Image;

/// A wrapper around a raw window.
pub trait RawWindow {
    /// Get the title of the window.
    fn title(&self) -> String;
    /// Set the title of the window.
    fn set_title(&mut self, title: &str);

    /// Set the icon of the window.
    fn set_icon(&mut self, icon: Option<&Image>);

    /// Get the size of the window.
    fn size(&self) -> (u32, u32);
    /// Set the size of the window.
    fn set_size(&mut self, width: u32, height: u32);

    /// Get whether the window is resizable.
    fn resizable(&self) -> bool;
    /// Set whether the window is resizable.
    fn set_resizable(&mut self, resizable: bool);

    /// Get whether the window is decorated.
    fn decorated(&self) -> bool;
    /// Set whether the window is decorated.
    fn set_decorated(&mut self, decorated: bool);

    /// Get the scale factor of the window.
    fn scale_factor(&self) -> f32;

    /// Get whether the window is minimized.
    fn minimized(&self) -> bool;
    /// Set whether the window is minimized.
    fn set_minimized(&mut self, minimized: bool);

    /// Get whether the window is maximized.
    fn maximized(&self) -> bool;
    /// Set whether the window is maximized.
    fn set_maximized(&mut self, maximized: bool);

    /// Get whether the window is visible.
    fn visible(&self) -> bool;
    /// Set whether the window is visible.
    fn set_visible(&mut self, visible: bool);

    /// Get whether the window is focused.
    fn request_draw(&mut self);
}

impl Debug for dyn RawWindow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RawWindow")
            .field("title", &self.title())
            .field("size", &self.size())
            .finish()
    }
}