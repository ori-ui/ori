use ori_core::{
    command::{CommandProxy, CommandWaker},
    context::Contexts,
    style::{Styles, Theme},
    text::{include_font, FontSource, Fonts},
    window::Window,
};

use crate::{App, AppDelegate, AppRequest, IntoUiBuilder};

/// A builder for an [`App`].
pub struct AppBuilder<T> {
    delegates: Vec<Box<dyn AppDelegate<T>>>,
    requests: Vec<AppRequest<T>>,
    styles: Styles,
    fonts: Vec<FontSource<'static>>,
}

impl<T> Default for AppBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> AppBuilder<T> {
    /// Create a new application builder.
    pub fn new() -> Self {
        Self {
            delegates: Vec::new(),
            requests: Vec::new(),
            styles: Styles::from(Theme::dark()),
            fonts: vec![include_font!("font")],
        }
    }

    /// Add a delegate to the application.
    pub fn delegate(mut self, delegate: impl AppDelegate<T> + 'static) -> Self {
        self.delegates.push(Box::new(delegate));
        self
    }

    /// Add a style to the application.
    pub fn style(mut self, styles: impl Into<Styles>) -> Self {
        self.styles.extend(styles);
        self
    }

    /// Add a theme to the application.
    pub fn theme(mut self, theme: Theme) -> Self {
        self.styles.extend(Styles::from(theme));
        self
    }

    /// Add a font to the application.
    pub fn font(mut self, font: impl Into<FontSource<'static>>) -> Self {
        self.fonts.push(font.into());
        self
    }

    /// Add a window to the application.
    pub fn window<V, P>(
        mut self,
        window: Window,
        builder: impl IntoUiBuilder<V, P, Data = T>,
    ) -> Self {
        let builder = builder.into_ui_builder();
        (self.requests).push(AppRequest::OpenWindow(window, builder));
        self
    }

    /// Build the application.
    pub fn build(self, waker: CommandWaker, mut fonts: Box<dyn Fonts>) -> App<T> {
        for font in self.fonts {
            fonts.load(font);
        }

        let (proxy, receiver) = CommandProxy::new(waker);

        let mut contexts = Contexts::new();
        contexts.insert(self.styles);
        contexts.insert(fonts);

        App {
            windows: Default::default(),
            modifiers: Default::default(),
            delegates: self.delegates,
            proxy,
            receiver,
            requests: self.requests,
            contexts,
        }
    }
}
