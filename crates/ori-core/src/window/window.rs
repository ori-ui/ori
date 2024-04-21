use std::{
    fmt::Display,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::{
    canvas::Color,
    event::{PointerButton, PointerId},
    image::Image,
    layout::{Point, Rect, Size},
    view::ViewId,
};

use super::{Cursor, RawWindow};

/// A unique identifier for a window.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WindowId {
    index: usize,
}

impl Default for WindowId {
    fn default() -> Self {
        Self::new()
    }
}

impl WindowId {
    /// Create a new [`WindowId`].
    pub fn new() -> Self {
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
        let index = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        Self { index }
    }
}

impl Display for WindowId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:x}", self.index)
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
    pub fn new(raw: Box<dyn RawWindow>, id: WindowId) -> Self {
        Self {
            id,
            raw,
            pointers: Vec::new(),
        }
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

    /// Get the mutable pointer with `id`.
    pub fn pointer_mut(&mut self, id: PointerId) -> Option<&mut Pointer> {
        self.pointers.iter_mut().find(|p| p.id() == id)
    }

    /// Move the pointer with `id` to `position`.
    pub fn pointer_moved(&mut self, id: PointerId, position: Point) {
        if let Some(pointer) = self.pointer_mut(id) {
            pointer.position = position;
        } else {
            self.pointers.push(Pointer::new(id, position));
        }
    }

    /// Remove the pointer with `id`.
    pub fn pointer_left(&mut self, id: PointerId) {
        if let Some(index) = self.pointers.iter().position(|p| p.id() == id) {
            self.pointers.swap_remove(index);
        }
    }

    /// Set the hovered view of the pointer with `id`.
    pub fn pointer_hovered(&mut self, id: PointerId, hovered: Option<ViewId>) -> bool {
        if let Some(pointer) = self.pointer_mut(id) {
            let changed = pointer.hovered() != hovered;
            pointer.hovered = hovered;
            return changed;
        }

        false
    }

    /// Get the pointer that is currently hovered over the given view.
    pub fn is_hovered(&self, view: ViewId) -> bool {
        self.pointers.iter().any(|p| p.hovered() == Some(view))
    }

    /// Pointer pressed.
    pub fn pointer_pressed(&mut self, id: PointerId, button: PointerButton) {
        if let Some(pointer) = self.pointer_mut(id) {
            pointer.press(button);
        }
    }

    /// Pointer released.
    pub fn pointer_released(&mut self, id: PointerId, button: PointerButton) -> bool {
        (self.pointer_mut(id)).map_or(false, |pointer| pointer.release(button))
    }

    /// Try to downcast the window to a specific type.
    pub fn downcast_raw<T: RawWindow>(&self) -> Option<&T> {
        self.raw.downcast_ref()
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

    /// Get the rect of the window.
    pub fn rect(&self) -> Rect {
        Rect::min_size(Point::ZERO, self.size())
    }

    /// Get the physical size of the window.
    pub fn physical_size(&self) -> Size {
        self.size() * self.scale_factor()
    }

    /// Get the width of the window.
    pub fn width(&self) -> u32 {
        self.raw.size().0
    }

    /// Get the height of the window.
    pub fn height(&self) -> u32 {
        self.raw.size().1
    }

    /// Get the physical width of the window.
    pub fn physical_width(&self) -> u32 {
        let width = self.width() as f32 * self.scale_factor();
        width as u32
    }

    /// Get the physical height of the window.
    pub fn physical_height(&self) -> u32 {
        let height = self.height() as f32 * self.scale_factor();
        height as u32
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

    /// Set the cursor of the window.
    pub fn set_cursor(&mut self, cursor: Cursor) {
        self.raw.set_cursor(cursor);
    }

    /// Get the background of the window.
    pub fn color(&self) -> Option<Color> {
        self.raw.color()
    }

    /// Set the background of the window.
    pub fn set_color(&mut self, color: Option<Color>) {
        self.raw.set_color(color);
    }

    /// Get whether the soft input is enabled.
    pub fn set_soft_input(&mut self, enabled: bool) {
        self.raw.set_soft_input(enabled);
    }

    /// Drag the window.
    pub fn drag(&mut self) {
        self.raw.drag();
    }

    /// Request a redraw of the window.
    pub fn request_draw(&mut self) {
        self.raw.request_draw();
    }
}

/// The state of a pointer.
#[derive(Clone, Debug, PartialEq)]
pub struct Pointer {
    /// The unique id of the pointer.
    id: PointerId,

    /// The position of the pointer.
    position: Point,

    /// The buttons that are currently pressed by the pointer.
    pressed: Vec<(PointerButton, Point)>,

    /// The view that is currently hovered by the pointer.
    hovered: Option<ViewId>,
}

impl Pointer {
    /// Create a new pointer.
    pub fn new(id: PointerId, position: Point) -> Self {
        Self {
            id,
            position,
            pressed: Vec::new(),
            hovered: None,
        }
    }

    /// Get the unique id of the pointer.
    pub fn id(&self) -> PointerId {
        self.id
    }

    /// Get the position of the pointer.
    pub fn position(&self) -> Point {
        self.position
    }

    /// Get the view that is currently hovered by the pointer.
    pub fn hovered(&self) -> Option<ViewId> {
        self.hovered
    }

    /// Set the view that is currently hovered by the pointer.
    pub fn set_hovered(&mut self, hovered: Option<ViewId>) {
        self.hovered = hovered;
    }

    /// Check if a button is currently pressed by the pointer.
    pub fn is_pressed(&self, button: PointerButton) -> bool {
        self.pressed.iter().any(|(b, _)| *b == button)
    }

    /// Press a button.
    pub fn press(&mut self, button: PointerButton) {
        if !self.is_pressed(button) {
            self.pressed.push((button, self.position));
        }
    }

    /// Release a button.
    ///
    /// Returns `true` if the button was clicked.
    pub fn release(&mut self, button: PointerButton) -> bool {
        let Some(index) = self.pressed.iter().position(|(b, _)| *b == button) else {
            return false;
        };

        let (_, position) = self.pressed.swap_remove(index);
        self.position.distance(position) < 10.0
    }
}
