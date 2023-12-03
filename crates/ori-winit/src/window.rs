use ori_core::{
    canvas::Color,
    image::Image,
    window::{Cursor, RawWindow},
};
use winit::{dpi::LogicalSize, window::Icon};

use crate::convert::convert_cursor_icon;

pub struct WinitWindow {
    pub(crate) window: winit::window::Window,
    soft_input: bool,
    background_color: Option<Color>,
}

impl From<winit::window::Window> for WinitWindow {
    fn from(window: winit::window::Window) -> Self {
        Self {
            window,
            soft_input: false,
            background_color: None,
        }
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
        let physical = self.window.inner_size();
        (physical.to_logical::<f32>(self.window.scale_factor())).into()
    }

    fn set_size(&mut self, width: u32, height: u32) {
        (self.window).set_min_inner_size(Some(LogicalSize::new(width, height)));
        (self.window).set_max_inner_size(Some(LogicalSize::new(width, height)));
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

    fn color(&self) -> Option<Color> {
        self.background_color
    }

    fn set_color(&mut self, color: Option<Color>) {
        self.background_color = color;
    }

    fn set_soft_input(&mut self, visible: bool) {
        if self.soft_input == visible {
            return;
        }

        self.soft_input = visible;
    }

    fn drag_window(&mut self) {
        let _ = self.window.drag_window();
    }

    fn request_draw(&mut self) {
        self.window.request_redraw();
    }
}
