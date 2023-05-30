use glam::UVec2;
use ori_graphics::{Color, ImageData};

use crate::{Cursor, WindowId};

#[derive(Clone, Debug, PartialEq)]
pub struct Window {
    id: WindowId,
    pub title: String,
    pub resizable: bool,
    pub clear_color: Color,
    pub icon: Option<ImageData>,
    pub scale: f32,
    pub size: UVec2,
    pub visible: bool,
    pub cursor: Cursor,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            id: WindowId::incremental(),
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
    pub fn new() -> Self {
        Self::default()
    }

    pub const fn id(&self) -> WindowId {
        self.id
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn esizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    pub fn clear_color(mut self, clear_color: Color) -> Self {
        self.clear_color = clear_color;
        self
    }

    pub fn icon(mut self, icon: Option<ImageData>) -> Self {
        self.icon = icon;
        self
    }

    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.size = UVec2::new(width, height);
        self
    }

    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    pub fn cursor(mut self, cursor: Cursor) -> Self {
        self.cursor = cursor;
        self
    }
}
