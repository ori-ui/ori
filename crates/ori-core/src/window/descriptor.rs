use glam::UVec2;
use ori_graphics::{Color, ImageData};

use crate::{Cursor, WindowId};

/// A descriptor for a window.
#[derive(Clone, Debug, PartialEq)]
pub struct Window {
    id: WindowId,
    /// The title of the window.
    pub title: String,
    /// Whether the window is resizable.
    pub resizable: bool,
    /// The clear color of the window.
    pub clear_color: Color,
    /// The icon of the window.
    pub icon: Option<ImageData>,
    /// The scale of the window.
    pub scale: f32,
    /// The size of the window.
    pub size: UVec2,
    /// Whether the window is visible.
    pub visible: bool,
    /// The cursor of the window.
    pub cursor: Cursor,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            id: WindowId::new(),
            title: String::from("Ori App"),
            resizable: true,
            clear_color: Color::WHITE,
            icon: None,
            scale: 1.0,
            size: UVec2::new(800, 600),
            visible: true,
            cursor: Cursor::default(),
        }
    }
}

impl Window {
    /// Create a new window descriptor.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the [`WindowId`] of the window.
    pub const fn id(&self) -> WindowId {
        self.id
    }

    /// Set the `title` of the window.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Set whether the window is `resizable`.
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    /// Set the `clear_color` of the window.
    pub fn clear_color(mut self, clear_color: Color) -> Self {
        self.clear_color = clear_color;
        self
    }

    /// Set the `icon` of the window.
    pub fn icon(mut self, icon: Option<ImageData>) -> Self {
        self.icon = icon;
        self
    }

    /// Set the `scale` of the window.
    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    /// Set the `size` of the window.
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.size = UVec2::new(width, height);
        self
    }

    /// Set whether the window is `visible`.
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Set the `cursor` of the window.
    pub fn cursor(mut self, cursor: Cursor) -> Self {
        self.cursor = cursor;
        self
    }
}
