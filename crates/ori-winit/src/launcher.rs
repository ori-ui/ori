use std::sync::Arc;

use ori_core::{
    command::CommandProxy,
    delegate::Delegate,
    text::FontSource,
    theme::{Palette, Theme},
    ui::{Ui, UiBuilder},
    view::{any, View},
    window::WindowDescriptor,
};
use winit::event_loop::{EventLoop, EventLoopBuilder};

use crate::Error;

/// A launcher for an application.
pub struct Launcher<T: 'static> {
    pub(crate) event_loop: EventLoop<()>,
    pub(crate) windows: Vec<(WindowDescriptor, UiBuilder<T>)>,
    pub(crate) ui: Ui<T>,
    pub(crate) msaa: bool,
}

impl<T: 'static> Launcher<T> {
    fn build_event_loop() -> EventLoop<()> {
        let mut builder = EventLoopBuilder::new();

        #[cfg(target_os = "android")]
        {
            use winit::platform::android::EventLoopBuilderExtAndroid;

            let app = crate::android::get_android_app();
            builder.with_android_app(app);
        }

        builder.build().unwrap()
    }

    /// Creates a new application.
    pub fn new(data: T) -> Self {
        let event_loop = Self::build_event_loop();

        let waker = Arc::new({
            let proxy = event_loop.create_proxy();

            move || {
                let _ = proxy.send_event(());
            }
        });

        let mut app = Self {
            event_loop,
            windows: Vec::new(),
            ui: Ui::new(data, waker),
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
        self.windows.push((descriptor, builder));
        self
    }

    /// Try to run the application.
    pub fn try_launch(self) -> Result<(), Error> {
        crate::launch::launch(self)
    }

    /// Run the application.
    pub fn launch(self) {
        if let Err(err) = self.try_launch() {
            panic!("Failed running the application: {}", err);
        }
    }
}
