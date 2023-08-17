use std::sync::atomic::{AtomicUsize, Ordering};

use glam::Vec2;

use crate::{Image, Pointer, PointerId, RawWindow, Size, WindowDescriptor};

/// A unique identifier for a window.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WindowId {
    index: usize,
}

impl WindowId {
    /// Create a new [`WindowId`].
    pub fn next() -> Self {
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
        let index = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        Self { index }
    }
}

/// A handle to a window.
#[derive(Debug)]
pub struct Window {
    id: WindowId,
    raw: Box<dyn RawWindow>,
    pointers: Vec<Pointer>,
}

impl Window {
    /// Create a new window with the given raw window implementation.
    pub fn new(raw: Box<dyn RawWindow>, desc: WindowDescriptor) -> Self {
        let mut window = Self {
            id: desc.id,
            raw,
            pointers: Vec::new(),
        };

        window.set_title(&desc.title);
        window.set_icon(desc.icon.as_ref());
        window.set_size(desc.width, desc.height);
        window.set_resizable(desc.resizable);
        window.set_decorated(desc.decorated);
        window.set_maximized(desc.maximized);
        window.set_visible(desc.visible);

        window
    }

    /// Get the [`WindowId`].
    pub fn id(&self) -> WindowId {
        self.id
    }

    /// Get the pointers.
    pub fn pointers(&self) -> &[Pointer] {
        &self.pointers
    }

    /// Get the pointer with `id`.
    pub fn pointer(&self, id: PointerId) -> Option<&Pointer> {
        self.pointers.iter().find(|p| p.id() == id)
    }

    pub(crate) fn pointer_mut(&mut self, id: PointerId) -> Option<&mut Pointer> {
        self.pointers.iter_mut().find(|p| p.id() == id)
    }

    pub(crate) fn pointer_moved(&mut self, id: PointerId, position: Vec2) {
        if let Some(pointer) = self.pointer_mut(id) {
            pointer.position = position;
        } else {
            self.pointers.push(Pointer::new(id, position));
        }
    }

    pub(crate) fn pointer_left(&mut self, id: PointerId) {
        if let Some(index) = self.pointers.iter().position(|p| p.id() == id) {
            self.pointers.swap_remove(index);
        }
    }

    /// Get the title of the window.
    pub fn title(&self) -> String {
        self.raw.title()
    }

    /// Set the title of the window.
    pub fn set_title(&mut self, title: &str) {
        self.raw.set_title(title);
    }

    /// Set the icon of the window.
    pub fn set_icon(&mut self, icon: Option<&Image>) {
        self.raw.set_icon(icon);
    }

    /// Get the size of the window.
    pub fn size(&self) -> Size {
        let (width, height) = self.raw.size();
        Size::new(width as f32, height as f32)
    }

    /// Get the width of the window.
    pub fn width(&self) -> u32 {
        self.raw.size().0
    }

    /// Get the height of the window.
    pub fn height(&self) -> u32 {
        self.raw.size().1
    }

    /// Set the size of the window.
    pub fn set_size(&mut self, width: u32, height: u32) {
        self.raw.set_size(width, height);
    }

    /// Get whether the window is resizable.
    pub fn resizable(&self) -> bool {
        self.raw.resizable()
    }

    /// Set whether the window is resizable.
    pub fn set_resizable(&mut self, resizable: bool) {
        self.raw.set_resizable(resizable);
    }

    /// Get whether the window is decorated.
    pub fn decorated(&self) -> bool {
        self.raw.decorated()
    }

    /// Set whether the window is decorated.
    pub fn set_decorated(&mut self, decorated: bool) {
        self.raw.set_decorated(decorated);
    }

    /// Get the scale factor of the window.
    pub fn scale_factor(&self) -> f32 {
        self.raw.scale_factor()
    }

    /// Get whether the window is minimized.
    pub fn minimized(&self) -> bool {
        self.raw.minimized()
    }

    /// Set whether the window is minimized.
    pub fn set_minimized(&mut self, minimized: bool) {
        self.raw.set_minimized(minimized);
    }

    /// Get whether the window is maximized.
    pub fn maximized(&self) -> bool {
        self.raw.maximized()
    }

    /// Set whether the window is maximized.
    pub fn set_maximized(&mut self, maximized: bool) {
        self.raw.set_maximized(maximized);
    }

    /// Get whether the window is visible.
    pub fn visible(&self) -> bool {
        self.raw.visible()
    }

    /// Set whether the window is visible.
    pub fn set_visible(&mut self, visible: bool) {
        self.raw.set_visible(visible);
    }

    /// Request a redraw of the window.
    pub fn request_draw(&mut self) {
        self.raw.request_draw();
    }
}
