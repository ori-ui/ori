use ori_core::{FontSource, Image, Theme, Ui, UiBuilder, View, WindowDescriptor};

use crate::{Error, Render};

/// An application.
pub struct App<T> {
    pub(crate) window: WindowDescriptor,
    pub(crate) builder: UiBuilder<T>,
    pub(crate) ui: Ui<T, Render>,
}

impl<T: 'static> App<T> {
    /// Creates a new application.
    pub fn new<V>(mut builder: impl FnMut(&mut T) -> V + 'static, data: T) -> Self
    where
        V: View<T> + 'static,
        V::State: 'static,
    {
        Self {
            window: WindowDescriptor::default(),
            builder: Box::new(move |data| Box::new(builder(data))),
            ui: Ui::new(data),
        }
    }

    /// Append the theme of the application.
    pub fn theme(mut self, theme: impl Into<Theme>) -> Self {
        self.ui.theme.extend(theme);
        self
    }

    /// Load a font.
    pub fn font(mut self, font: impl Into<FontSource>) -> Self {
        if let Err(err) = self.ui.fonts.load_font(font) {
            eprintln!("Failed to load font: {:?}", err);
        }

        self
    }

    /// Set the title of the window.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.window.title = title.into();
        self
    }

    /// Set the icon of the window.
    pub fn icon(mut self, icon: Option<Image>) -> Self {
        self.window.icon = icon;
        self
    }

    /// Set the size of the window.
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.window.width = width;
        self.window.height = height;
        self
    }

    /// Set whether the window is resizable.
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.window.resizable = resizable;
        self
    }

    /// Set whether the window is decorated.
    pub fn decorated(mut self, decorated: bool) -> Self {
        self.window.decorated = decorated;
        self
    }

    /// Set whether the window is transparent.
    pub fn transparent(mut self, transparent: bool) -> Self {
        self.window.transparent = transparent;
        self
    }

    /// Set whether the window is maximized.
    pub fn maximized(mut self, maximized: bool) -> Self {
        self.window.maximized = maximized;
        self
    }

    /// Set whether the window is visible.
    pub fn visible(mut self, visible: bool) -> Self {
        self.window.visible = visible;
        self
    }

    /// Try to run the application.
    pub fn try_run(self) -> Result<(), Error> {
        crate::run::run(self)
    }

    /// Run the application.
    pub fn run(self) {
        if let Err(err) = self.try_run() {
            panic!("{}", err);
        }
    }
}
