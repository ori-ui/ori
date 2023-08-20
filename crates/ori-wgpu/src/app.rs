use ori_core::{
    delegate::Delegate,
    image::Image,
    style::{Palette, Theme},
    text::FontSource,
    ui::Ui,
    view::View,
    window::{UiBuilder, WindowDescriptor},
};

use crate::{render::Render, Error};

/// An application.
pub struct App<T> {
    pub(crate) window: WindowDescriptor,
    pub(crate) builder: UiBuilder<T>,
    pub(crate) theme: Vec<Box<dyn Fn() -> Theme>>,
    pub(crate) ui: Ui<T, Render>,
    pub(crate) text_size: f32,
}

impl<T: 'static> App<T> {
    /// Creates a new application.
    pub fn new<V>(mut builder: impl FnMut(&mut T) -> V + 'static, data: T) -> Self
    where
        V: View<T> + 'static,
        V::State: 'static,
    {
        let mut app = Self {
            window: WindowDescriptor::default(),
            builder: Box::new(move |data| Box::new(builder(data))),
            theme: Vec::new(),
            ui: Ui::new(data),
            text_size: 16.0,
        };

        app.ui.fonts.load_system_fonts();

        app.theme(Palette::light).theme(Theme::builtin)
    }

    /// Append the theme of the application.
    pub fn theme<I: Into<Theme>>(mut self, theme: impl Fn() -> I + 'static) -> Self {
        self.theme.push(Box::new(move || theme().into()));
        self
    }

    /// Load a font.
    pub fn font(mut self, font: impl Into<FontSource>) -> Self {
        if let Err(err) = self.ui.fonts.load_font(font) {
            eprintln!("Failed to load font: {:?}", err);
        }

        self
    }

    /// Set the text size of the application.
    pub fn text_size(mut self, size: f32) -> Self {
        self.text_size = size;
        self
    }

    /// Set the delegate of the application.
    pub fn delegate(mut self, delegate: impl Delegate<T> + 'static) -> Self {
        self.ui.set_delegate(delegate);
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
