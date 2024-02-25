use ori_core::{
    command::CommandWaker,
    shell::{Shell, Windows},
    ui::Ui,
};
use winit::event_loop::{EventLoop, EventLoopBuilder};

use crate::Error;

/// The winit [`Shell`].
pub struct WinitShell {
    event_loop: EventLoop<()>,
}

impl WinitShell {
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
}

impl Shell for WinitShell {
    type Error = Error;

    fn init() -> (Self, CommandWaker) {
        let event_loop = Self::build_event_loop();

        let waker = CommandWaker::new({
            let proxy = event_loop.create_proxy();

            move || {
                let _ = proxy.send_event(());
            }
        });

        let shell = Self { event_loop };

        (shell, waker)
    }

    fn run<T>(self, data: T, ui: Ui<T>, windows: Windows<T>) -> Result<(), Self::Error> {
        crate::launch::launch(data, self.event_loop, ui, windows)
    }
}
