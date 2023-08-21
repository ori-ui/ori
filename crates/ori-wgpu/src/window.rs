use ori_core::{
    image::Image,
    window::{Cursor, RawWindow},
};
use winit::{dpi::PhysicalSize, window::Icon};

use crate::convert::convert_cursor_icon;

pub struct WinitWindow {
    window: winit::window::Window,
}

impl From<winit::window::Window> for WinitWindow {
    fn from(window: winit::window::Window) -> Self {
        Self { window }
    }
}

impl RawWindow for WinitWindow {
    fn title(&self) -> String {
        self.window.title()
    }

    fn set_title(&mut self, title: &str) {
        self.window.set_title(title);
    }

    fn set_icon(&mut self, icon: Option<&Image>) {
        let icon = match icon {
            Some(image) => {
                Icon::from_rgba(image.pixels().to_vec(), image.width(), image.height()).ok()
            }
            None => None,
        };

        self.window.set_window_icon(icon);
    }

    fn size(&self) -> (u32, u32) {
        self.window.inner_size().into()
    }

    fn set_size(&mut self, width: u32, height: u32) {
        self.window.set_inner_size(PhysicalSize::new(width, height));
    }

    fn resizable(&self) -> bool {
        self.window.is_resizable()
    }

    fn set_resizable(&mut self, resizable: bool) {
        self.window.set_resizable(resizable);
    }

    fn decorated(&self) -> bool {
        self.window.is_decorated()
    }

    fn set_decorated(&mut self, decorated: bool) {
        self.window.set_decorations(decorated);
    }

    fn scale_factor(&self) -> f32 {
        self.window.scale_factor() as f32
    }

    fn minimized(&self) -> bool {
        self.window.is_minimized().unwrap_or(false)
    }

    fn set_minimized(&mut self, minimized: bool) {
        self.window.set_minimized(minimized);
    }

    fn maximized(&self) -> bool {
        self.window.is_maximized()
    }

    fn set_maximized(&mut self, maximized: bool) {
        self.window.set_maximized(maximized);
    }

    fn visible(&self) -> bool {
        self.window.is_visible().unwrap_or(false)
    }

    fn set_visible(&mut self, visible: bool) {
        self.window.set_visible(visible);
    }

    fn set_cursor(&mut self, cursor: Cursor) {
        (self.window).set_cursor_icon(convert_cursor_icon(cursor));
    }

    fn request_draw(&mut self) {
        self.window.request_redraw();
    }
}
