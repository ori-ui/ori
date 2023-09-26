use std::sync::Arc;

use ori_core::{
    command::CommandProxy,
    delegate::Delegate,
    image::Image,
    text::FontSource,
    theme::{Palette, Theme},
    ui::{Ui, UiBuilder},
    view::View,
    window::WindowDescriptor,
};
use winit::event_loop::{EventLoop, EventLoopBuilder};

use crate::{proxy::WinitWaker, Error};

/// An application.
pub struct App<T: 'static> {
    pub(crate) event_loop: EventLoop<()>,
    pub(crate) window: WindowDescriptor,
    pub(crate) builder: UiBuilder<T>,
    pub(crate) ui: Ui<T>,
    pub(crate) msaa: bool,
}

impl<T: 'static> App<T> {
    fn build_event_loop() -> EventLoop<()> {
        let mut builder = EventLoopBuilder::new();

        #[cfg(target_os = "android")]
        {
            use winit::platform::android::EventLoopBuilderExtAndroid;

            let app = crate::android::get_android_app();
            builder.with_android_app(app);
        }

        builder.build()
    }

    /// Creates a new application.
    pub fn new<V>(mut builder: impl FnMut(&mut T) -> V + 'static, data: T) -> Self
    where
        V: View<T> + 'static,
    {
        let event_loop = Self::build_event_loop();

        let waker = WinitWaker {
            proxy: event_loop.create_proxy().into(),
        };

        let mut app = Self {
            event_loop,
            window: WindowDescriptor::default(),
            builder: Box::new(move |data| Box::new(builder(data))),
            ui: Ui::new(data, Arc::new(waker)),
            #[cfg(feature = "wgpu")]
            msaa: true,
        };

        app.ui.fonts.load_system_fonts();

        app.theme(Palette::light).theme(Theme::builtin)
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

    /// Set whether the application uses multisample anti-aliasing.
    pub fn msaa(mut self, msaa: bool) -> Self {
        self.msaa = msaa;
        self
    }

    /// Set the delegate of the application.
    pub fn delegate(mut self, delegate: impl Delegate<T> + 'static) -> Self {
        self.ui.set_delegate(delegate);
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

    /// Set the title of the window.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.window.title = title.into();
        self
    }

    /// Set the icon of the window.
    pub fn icon(mut self, icon: impl Into<Option<Image>>) -> Self {
        self.window.icon = icon.into();
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
            panic!("Failed running the application: {}", err);
        }
    }
}
