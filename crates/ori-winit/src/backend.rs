use std::collections::HashMap;

use ori_core::{Cursor, Window, WindowBackend, WindowId};
use ori_graphics::{prelude::UVec2, ImageData};
use ori_reactive::{Event, EventEmitter, EventSink};
use raw_window_handle::{
    HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle, RawWindowHandle,
};
use winit::{
    dpi::LogicalSize,
    event_loop::{EventLoopProxy, EventLoopWindowTarget},
    window::{Icon, Window as WinitWindow, WindowBuilder, WindowId as WinitWindowId},
};

use crate::convert::convert_cursor_icon;

struct EventLoopSender {
    window: WinitWindowId,
    proxy: EventLoopProxy<(WinitWindowId, Event)>,
}

impl EventLoopSender {
    fn new(window: WinitWindowId, proxy: EventLoopProxy<(WinitWindowId, Event)>) -> Self {
        Self { window, proxy }
    }
}

impl EventEmitter for EventLoopSender {
    fn send_event(&mut self, event: Event) {
        let _ = self.proxy.send_event((self.window, event));
    }
}

pub struct WinitBackend {
    pub(crate) proxy: EventLoopProxy<(winit::window::WindowId, Event)>,
    pub(crate) windows: HashMap<WindowId, WinitWindow>,
    pub(crate) ids: HashMap<WinitWindowId, WindowId>,
}

impl WinitBackend {
    pub fn new(proxy: EventLoopProxy<(winit::window::WindowId, Event)>) -> Self {
        Self {
            proxy,
            windows: HashMap::new(),
            ids: HashMap::new(),
        }
    }

    pub fn id(&self, id: WinitWindowId) -> Option<WindowId> {
        self.ids.get(&id).copied()
    }
}

impl WindowBackend for WinitBackend {
    type Target<'a> = &'a EventLoopWindowTarget<(winit::window::WindowId, Event)>;
    type Surface = (RawDisplayHandle, RawWindowHandle);
    type Error = winit::error::OsError;

    fn create_window(
        &mut self,
        target: Self::Target<'_>,
        window: &Window,
    ) -> Result<(), Self::Error> {
        let icon = match window.icon {
            Some(ref icon) => {
                Icon::from_rgba(icon.pixels().to_vec(), icon.width(), icon.height()).ok()
            }
            None => None,
        };

        let winit_window = WindowBuilder::new()
            .with_title(window.title.clone())
            .with_resizable(window.resizable)
            .with_window_icon(icon)
            .with_inner_size(LogicalSize::new(window.size.x as f64, window.size.y as f64))
            .with_visible(window.visible)
            .with_transparent(window.clear_color.is_translucent())
            .build(target)?;

        let id = winit_window.id();
        self.ids.insert(id, window.id());
        self.windows.insert(window.id(), winit_window);

        Ok(())
    }

    fn create_surface(&self, id: WindowId) -> Result<Self::Surface, Self::Error> {
        let display = self.windows[&id].raw_display_handle();
        let window = self.windows[&id].raw_window_handle();

        Ok((display, window))
    }

    fn create_event_sink(&self, id: WindowId) -> Result<EventSink, Self::Error> {
        let window = self.windows[&id].id();
        let sender = EventLoopSender::new(window, self.proxy.clone());
        Ok(EventSink::new(sender))
    }

    fn request_redraw(&mut self, id: WindowId) {
        self.windows[&id].request_redraw();
    }

    fn close_window(&mut self, id: WindowId) {
        self.windows.remove(&id);
    }

    fn get_title(&self, id: WindowId) -> String {
        self.windows[&id].title()
    }

    fn set_title(&mut self, id: WindowId, title: impl Into<String>) {
        self.windows[&id].set_title(&title.into());
    }

    fn get_resizable(&self, id: WindowId) -> bool {
        self.windows[&id].is_resizable()
    }

    fn set_resizable(&mut self, id: WindowId, resizable: bool) {
        self.windows[&id].set_resizable(resizable);
    }

    fn set_transparent(&mut self, id: WindowId, transparent: bool) {
        self.windows[&id].set_transparent(transparent);
    }

    fn set_icon(&mut self, id: WindowId, icon: Option<ImageData>) {
        let icon = match icon {
            Some(ref icon) => {
                Icon::from_rgba(icon.pixels().to_vec(), icon.width(), icon.height()).ok()
            }
            None => None,
        };

        self.windows[&id].set_window_icon(icon);
    }

    fn get_size(&self, id: WindowId) -> UVec2 {
        let size = self.windows[&id].inner_size();
        UVec2::new(size.width, size.height)
    }

    fn set_size(&mut self, id: WindowId, size: UVec2) {
        self.windows[&id].set_inner_size(LogicalSize::new(size.x as f64, size.y as f64));
    }

    fn get_visible(&self, id: WindowId) -> bool {
        self.windows[&id].is_visible().unwrap_or(true)
    }

    fn set_visible(&mut self, id: WindowId, visible: bool) {
        self.windows[&id].set_visible(visible);
    }

    fn set_cursor(&mut self, id: WindowId, cursor: Cursor) {
        self.windows[&id].set_cursor_icon(convert_cursor_icon(cursor));
    }
}
