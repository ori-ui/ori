use crate::{
    image::Image,
    view::{BoxedView, View},
    window::{WindowDescriptor, WindowId},
};

/// Event emitted when a window wants to close.
///
/// After this event is emitted, if it wasn't handled, the window will be closed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CloseRequested {
    /// The window that wants to close.
    pub window: WindowId,
}

impl CloseRequested {
    /// Create a new close requested event.
    pub fn new(window: WindowId) -> Self {
        Self { window }
    }
}

/// Command to open a new window.
pub struct OpenWindow<T: 'static> {
    /// The descriptor of the window to open.
    pub desc: WindowDescriptor,
    /// The builder of the UI of the window to open.
    #[allow(clippy::type_complexity)]
    pub builder: Box<dyn FnMut(&mut T) -> BoxedView<T> + Send>,
}

impl<T> OpenWindow<T> {
    /// Create a new open window command.
    pub fn new<V>(mut ui: impl FnMut(&mut T) -> V + Send + 'static) -> Self
    where
        V: View<T> + 'static,
    {
        Self {
            desc: WindowDescriptor::default(),
            builder: Box::new(move |data| Box::new(ui(data))),
        }
    }

    /// Set the title of the window.
    pub fn title(mut self, title: impl ToString) -> Self {
        self.desc.title = title.to_string();
        self
    }

    /// Set the icon of the window.
    pub fn icon(mut self, icon: impl Into<Option<Image>>) -> Self {
        self.desc.icon = icon.into();
        self
    }

    /// Set the size of the window.
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.desc.width = width;
        self.desc.height = height;
        self
    }

    /// Set width of the window.
    pub fn width(mut self, width: u32) -> Self {
        self.desc.width = width;
        self
    }

    /// Set height of the window.
    pub fn height(mut self, height: u32) -> Self {
        self.desc.height = height;
        self
    }

    /// Set whether the window is resizable.
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.desc.resizable = resizable;
        self
    }

    /// Set whether the window is decorated.
    pub fn decorated(mut self, decorated: bool) -> Self {
        self.desc.decorated = decorated;
        self
    }

    /// Set whether the window is transparent.
    pub fn transparent(mut self, transparent: bool) -> Self {
        self.desc.transparent = transparent;
        self
    }

    /// Set whether the window is maximized.
    pub fn maximized(mut self, maximized: bool) -> Self {
        self.desc.maximized = maximized;
        self
    }

    /// Set whether the window is visible.
    pub fn visible(mut self, visible: bool) -> Self {
        self.desc.visible = visible;
        self
    }
}

/// Command to close a window.
#[derive(Clone, Debug)]
pub struct CloseWindow {
    /// The window to close.
    pub window: WindowId,
}

impl CloseWindow {
    /// Create a new close window command.
    pub fn new(window: WindowId) -> Self {
        Self { window }
    }
}
