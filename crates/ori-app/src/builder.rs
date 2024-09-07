use ori_core::{
    command::{CommandProxy, CommandWaker},
    context::Contexts,
    style::{Styles, Theme},
    text::{FontSource, Fonts},
    window::Window,
};

use crate::{App, AppRequest, Delegate, IntoUiBuilder};

/// A builder for an [`App`].
pub struct AppBuilder<T> {
    delegates: Vec<Box<dyn Delegate<T>>>,
    requests: Vec<AppRequest<T>>,
    styles: Styles,
    fonts: Fonts,
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
            fonts: Fonts::new(),
        }
    }

    /// Add a delegate to the application.
    pub fn delegate(mut self, delegate: impl Delegate<T> + 'static) -> Self {
        self.delegates.push(Box::new(delegate));
        self
    }

    /// Add a style to the application.
    pub fn style(mut self, styles: Styles) -> Self {
        self.styles.extend(styles);
        self
    }

    /// Add a theme to the application.
    pub fn theme(mut self, theme: Theme) -> Self {
        self.styles.extend(Styles::from(theme));
        self
    }

    /// Add a font to the application.
    pub fn font<'a>(mut self, font: impl Into<FontSource<'a>>) -> Self {
        if let Err(err) = self.fonts.load_font(font) {
            eprintln!("Failed to load font: {}", err);
        }

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
    pub fn build(self, waker: CommandWaker) -> App<T> {
        let (proxy, receiver) = CommandProxy::new(waker);

        let mut contexts = Contexts::new();
        contexts.insert(self.styles);
        contexts.insert(self.fonts);

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
