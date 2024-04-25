use std::{
    fmt::Display,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::{
    canvas::Color,
    event::{PointerButton, PointerId},
    image::Image,
    layout::{Point, Size, Vector},
    view::ViewId,
};

use super::{Cursor, Pointer};

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

/// The sizing of a window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowSizing {
    /// The window will have a fixed size equal to [`Window::size`].
    Fixed,

    /// The root [`View`](crate::view::View) will have [`Space::UNBOUNDED`](crate::layout::Space), and the window will
    /// resize to fit the content.
    Content,
}

impl Default for WindowSizing {
    fn default() -> Self {
        Self::Fixed
    }
}

/// A window.
#[derive(Debug)]
pub struct Window {
    id: WindowId,
    pointers: Vec<Pointer>,

    /// The title of the window.
    pub title: String,

    /// The icon of the window.
    pub icon: Option<Image>,

    /// The size of the window.
    pub size: Size,

    /// The sizing of the window.
    pub sizing: WindowSizing,

    /// The scale of the window.
    ///
    /// Modifying this is not recommended, and will probably not do what you expect,
    /// as a rule of thumb, don't do it.
    pub scale: f32,

    /// Whether the window is resizable.
    pub resizable: bool,

    /// Whether the window is decorated.
    pub decorated: bool,

    /// Whether the window is maximized.
    pub maximized: bool,

    /// Whether the window is visible.
    pub visible: bool,

    /// The color of the window.
    pub color: Option<Color>,
}

impl Default for Window {
    fn default() -> Self {
        Self::new()
    }
}

impl Window {
    /// Create a new [`Window`].
    pub fn new() -> Self {
        Self {
            id: WindowId::new(),
            pointers: Vec::new(),
            title: String::from("Ori window"),
            icon: None,
            size: Size::new(800.0, 600.0),
            sizing: WindowSizing::Fixed,
            scale: 1.0,
            resizable: true,
            decorated: true,
            maximized: false,
            visible: true,
            color: None,
        }
    }

    /// Get the unique identifier of the window.
    pub fn id(&self) -> WindowId {
        self.id
    }

    /// Set the title of the window.
    pub fn title(mut self, title: impl ToString) -> Self {
        self.title = title.to_string();
        self
    }

    /// Set the icon of the window.
    pub fn icon(mut self, icon: impl Into<Option<Image>>) -> Self {
        self.icon = icon.into();
        self
    }

    /// Set the size of the window.
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.size = Size::new(width as f32, height as f32);
        self
    }

    /// Set the sizing of the window.
    pub fn sizing(mut self, sizing: WindowSizing) -> Self {
        self.sizing = sizing;
        self
    }

    /// Set the sizing to [`WindowSizing::Content`].
    pub fn fit_content(mut self) -> Self {
        self.sizing = WindowSizing::Content;
        self
    }

    /// Set the scale of the window.
    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    /// Set whether the window is resizable.
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    /// Set whether the window is decorated.
    pub fn decorated(mut self, decorated: bool) -> Self {
        self.decorated = decorated;
        self
    }

    /// Set whether the window is maximized.
    pub fn maximized(mut self, maximized: bool) -> Self {
        self.maximized = maximized;
        self
    }

    /// Set whether the window is visible.
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Set the color of the window.
    pub fn color(mut self, color: impl Into<Option<Color>>) -> Self {
        self.color = color.into();
        self
    }

    /// Get the size of the window in physical pixels.
    ///
    /// This is a shorthand for `self.size * self.scale`.
    pub fn physical_size(&self) -> Size {
        self.size * self.scale
    }

    /// Get the width of the window.
    pub fn width(&self) -> u32 {
        self.size.width as u32
    }

    /// Get the height of the window.
    pub fn height(&self) -> u32 {
        self.size.height as u32
    }

    /// Get the pointers in the window.
    pub fn pointers(&self) -> &[Pointer] {
        &self.pointers
    }

    /// Get the pointers in the window mutably.
    pub fn pointers_mut(&mut self) -> &mut Vec<Pointer> {
        &mut self.pointers
    }

    /// Get whether a specific view is hovered.
    pub fn is_hovered(&self, view_id: ViewId) -> bool {
        (self.pointers.iter()).any(|pointer| pointer.hovering == Some(view_id))
    }

    /// Get whether `button` is held down on pointer with `pointer_id`.
    pub fn is_pointer_held(&self, pointer_id: PointerId, button: PointerButton) -> bool {
        match self.get_pointer(pointer_id) {
            Some(pointer) => pointer.is_pressed(button),
            None => false,
        }
    }

    /// Get the pointer with `pointer_id`.
    pub fn get_pointer(&self, pointer_id: PointerId) -> Option<&Pointer> {
        (self.pointers.iter()).find(|pointer| pointer.id() == pointer_id)
    }

    /// Get the pointer with `pointer_id` mutably.
    pub fn get_pointer_mut(&mut self, pointer_id: PointerId) -> Option<&mut Pointer> {
        (self.pointers.iter_mut()).find(|pointer| pointer.id() == pointer_id)
    }

    /// Press a button on a pointer.
    ///
    /// This is rarely what you want to do, do not use this unless you
    /// really know what you are doing.
    pub fn press_pointer(&mut self, pointer_id: PointerId, button: PointerButton) {
        if let Some(pointer) = self.get_pointer_mut(pointer_id) {
            pointer.press(button);
        }
    }

    /// Release a button on a pointer.
    ///
    /// This is rarely what you want to do, do not use this unless you
    /// really know what you are doing.
    pub fn release_pointer(&mut self, pointer_id: PointerId, button: PointerButton) -> bool {
        match self.get_pointer_mut(pointer_id) {
            Some(pointer) => pointer.release(button),
            None => false,
        }
    }

    /// Move a pointer, returning the movement.
    pub fn move_pointer(&mut self, pointer_id: PointerId, position: Point) -> Vector {
        match self.get_pointer_mut(pointer_id) {
            Some(pointer) => {
                let delta = position - pointer.position;
                pointer.position = position;

                delta
            }
            None => {
                self.pointers.push(Pointer::new(pointer_id, position));

                Vector::ZERO
            }
        }
    }

    /// Remove a pointer.
    ///
    /// This is rarely what you want to do, do not use this unless you
    /// really know what you are doing.
    pub fn remove_pointer(&mut self, pointer_id: PointerId) {
        self.pointers.retain(|pointer| pointer.id() != pointer_id);
    }

    /// Update the window.
    pub fn updates(&mut self) -> Vec<WindowUpdate> {
        vec![
            WindowUpdate::Title(self.title.clone()),
            WindowUpdate::Icon(self.icon.clone()),
            WindowUpdate::Size(self.size),
            WindowUpdate::Scale(self.scale),
            WindowUpdate::Resizable(self.resizable),
            WindowUpdate::Decorated(self.decorated),
            WindowUpdate::Maximized(self.maximized),
            WindowUpdate::Visible(self.visible),
            WindowUpdate::Color(self.color),
        ]
    }

    /// Get the [`WindowSnapshot`] of the window.
    pub fn snapshot(&self) -> WindowSnapshot {
        WindowSnapshot {
            title: self.title.clone(),
            icon: self.icon.clone(),
            size: self.size,
            scale: self.scale,
            resizable: self.resizable,
            decorated: self.decorated,
            maximized: self.maximized,
            visible: self.visible,
            color: self.color,
        }
    }
}

/// An update to a window.
#[derive(Clone, Debug)]
pub enum WindowUpdate {
    /// Set the title of the window.
    Title(String),

    /// Set the icon of the window.
    Icon(Option<Image>),

    /// Set the size of the window.
    Size(Size),

    /// Set the scale of the window.
    Scale(f32),

    /// Set whether the window is resizable.
    Resizable(bool),

    /// Set whether the window is decorated.
    Decorated(bool),

    /// Set whether the window is maximized.
    Maximized(bool),

    /// Set whether the window is visible.
    Visible(bool),

    /// Set the color of the window.
    Color(Option<Color>),

    /// Set the cursor of the window.
    Cursor(Cursor),
}

/// The state of a window.
#[derive(Clone, Debug)]
pub struct WindowSnapshot {
    /// The title of the window.
    pub title: String,

    /// The icon of the window.
    pub icon: Option<Image>,

    /// The size of the window.
    pub size: Size,

    /// The scale of the window.
    pub scale: f32,

    /// Whether the window is resizable.
    pub resizable: bool,

    /// Whether the window is decorated.
    pub decorated: bool,

    /// Whether the window is maximized.
    pub maximized: bool,

    /// Whether the window is visible.
    pub visible: bool,

    /// The color of the window.
    pub color: Option<Color>,
}

impl WindowSnapshot {
    /// Get the difference between a window and a previous state.
    pub fn difference(&self, window: &Window) -> Vec<WindowUpdate> {
        let mut updates = Vec::new();

        if self.title != window.title {
            updates.push(WindowUpdate::Title(window.title.clone()));
        }

        if self.icon != window.icon {
            updates.push(WindowUpdate::Icon(window.icon.clone()));
        }

        if self.size != window.size {
            updates.push(WindowUpdate::Size(window.size));
        }

        if self.scale != window.scale {
            updates.push(WindowUpdate::Scale(window.scale));
        }

        if self.resizable != window.resizable {
            updates.push(WindowUpdate::Resizable(window.resizable));
        }

        if self.decorated != window.decorated {
            updates.push(WindowUpdate::Decorated(window.decorated));
        }

        if self.maximized != window.maximized {
            updates.push(WindowUpdate::Maximized(window.maximized));
        }

        if self.visible != window.visible {
            updates.push(WindowUpdate::Visible(window.visible));
        }

        if self.color != window.color {
            updates.push(WindowUpdate::Color(window.color));
        }

        updates
    }
}
