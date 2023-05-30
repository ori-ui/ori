use std::fmt::Debug;

use glam::UVec2;
use ori_graphics::ImageData;
use ori_reactive::EventSink;

use crate::{Cursor, Window, WindowId};

/// A trait that defines the interface for a windowing backend.
pub trait WindowBackend {
    type Target<'a>;
    /// The type of the surface. This is used to create the [`Renderer`](ori_graphics::Renderer).
    type Surface;
    type Error: Debug;

    fn create_window(
        &mut self,
        target: Self::Target<'_>,
        window: &Window,
    ) -> Result<(), Self::Error>;
    fn create_surface(&self, id: WindowId) -> Result<Self::Surface, Self::Error>;
    fn create_event_sink(&self, id: WindowId) -> Result<EventSink, Self::Error>;

    fn request_redraw(&mut self, id: WindowId);
    fn close_window(&mut self, id: WindowId);

    fn get_title(&self, id: WindowId) -> String;
    fn set_title(&mut self, id: WindowId, title: impl Into<String>);

    fn get_resizable(&self, id: WindowId) -> bool;
    fn set_resizable(&mut self, id: WindowId, resizable: bool);

    fn set_transparent(&mut self, id: WindowId, transparent: bool);

    fn set_icon(&mut self, id: WindowId, icon: Option<ImageData>);

    fn get_size(&self, id: WindowId) -> UVec2;
    fn set_size(&mut self, id: WindowId, size: UVec2);

    fn get_visible(&self, id: WindowId) -> bool;
    fn set_visible(&mut self, id: WindowId, visible: bool);

    fn set_cursor(&mut self, id: WindowId, cursor: Cursor);
}
