//! Launcher for an application.

use crate::{
    command::CommandProxy,
    delegate::Delegate,
    shell::{Shell, Windows},
    text::FontSource,
    theme::{Palette, Theme},
    ui::{Ui, UiBuilder},
    view::{any, View},
    window::WindowDescriptor,
};

/// A launcher for an application.
pub struct Launcher<T: 'static, S> {
    pub(crate) shell: S,
    pub(crate) ui: Ui<T>,
    pub(crate) windows: Windows<T>,
}

impl<T, S: Shell> Launcher<T, S> {
    /// Crate a new application.
    pub fn new(data: T) -> Self {
        let (shell, waker) = S::init();

        let mut ui = Ui::new(data, waker);

        ui.push_theme(|| Palette::light().into());
        ui.push_theme(Theme::builtin);

        Self {
            shell,
            ui,
            windows: Windows::new(),
        }
    }

    /// Append the theme of the application.
    pub fn theme<I: Into<Theme>>(mut self, mut theme: impl FnMut() -> I + 'static) -> Self {
        self.ui.push_theme(move || theme().into());
        self
    }

    /// Load a font from a [`FontSource`].
    pub fn font(mut self, font: impl Into<FontSource>) -> Self {
        if let Err(err) = self.ui.fonts.load_font(font) {
            eprintln!("Failed to load font: {:?}", err);
        }

        self
    }

    /// Get the command proxy of the application.
    pub fn proxy(&self) -> CommandProxy {
        self.ui.proxy()
    }

    /// Set the proxy of the application.
    ///
    /// This is useful when starting background tasks.
    pub fn with_proxy(self, f: impl FnOnce(CommandProxy)) -> Self {
        f(self.proxy());
        self
    }

    /// Push a delegate to the application.
    ///
    /// Delegates are called in order of insertion.
    pub fn delegate(mut self, delegate: impl Delegate<T> + 'static) -> Self {
        self.ui.push_delegate(Box::new(delegate));
        self
    }

    /// Set the delegate of the application with a proxy.
    ///
    /// This is useful when starting background tasks.
    pub fn delegate_with_proxy<D: Delegate<T> + 'static>(
        self,
        delegate: impl FnOnce(CommandProxy) -> D,
    ) -> Self {
        let delegate = delegate(self.proxy());
        self.delegate(delegate)
    }

    /// Push a window to the application.
    pub fn window<V: View<T> + 'static>(
        mut self,
        descriptor: WindowDescriptor,
        mut ui: impl FnMut(&mut T) -> V + 'static,
    ) -> Self {
        let builder: UiBuilder<T> = Box::new(move |data| any(ui(data)));
        self.windows.push(descriptor, builder);
        self
    }

    /// Try to launch the application.
    pub fn try_launch(self) -> Result<(), S::Error> {
        self.shell.run(self.ui, self.windows)
    }

    /// Launch the application.
    pub fn launch(self) {
        self.try_launch().unwrap();
    }
}
