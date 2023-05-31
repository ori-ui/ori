use std::{
    error::Error,
    fmt::Display,
    time::{Duration, Instant},
};

use ori_core::{
    math::Vec2, LoadedStyleKind, Modifiers, Node, StyleLoader, Stylesheet, Window, Windows,
};
use ori_graphics::{prelude::UVec2, Color, ImageSource};
use ori_reactive::{Event, Scope};
use winit::{
    event::{Event as WinitEvent, KeyboardInput, MouseScrollDelta, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder},
    window::WindowId as WinitWindowId,
};

use crate::{
    backend::WinitBackend,
    convert::{convert_device_id, convert_key, convert_mouse_button, is_pressed},
};

fn init_tracing() -> Result<(), Box<dyn Error>> {
    use tracing_subscriber::layer::SubscriberExt;

    let filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive("wgpu=warn".parse()?)
        .with_default_directive("naga=warn".parse()?)
        .with_default_directive("winit=warn".parse()?)
        .with_default_directive("mio=warn".parse()?)
        .with_default_directive("debug".parse()?)
        .from_env()?;

    let subscriber = tracing_subscriber::registry().with(filter);
    let subscriber = subscriber.with(tracing_subscriber::fmt::Layer::default());

    #[cfg(feature = "tracy")]
    let subscriber = subscriber.with(tracing_tracy::TracyLayer::new());

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}

/// A app using [`winit`] as the windowing backend.
pub struct App {
    window: Window,
    style_loader: StyleLoader,
    event_loop: EventLoop<(WinitWindowId, Event)>,
    builder: Option<Box<dyn FnMut(Scope) -> Node + Send>>,
}

impl App {
    /// Create a new [`App`] with the given content.
    pub fn new(content: impl FnMut(Scope) -> Node + Send + 'static) -> Self {
        let event_loop = EventLoopBuilder::with_user_event().build();
        Self::new_with_event_loop(event_loop, content)
    }

    pub fn new_with_event_loop(
        event_loop: EventLoop<(WinitWindowId, Event)>,
        content: impl FnMut(Scope) -> Node + Send + 'static,
    ) -> Self {
        init_tracing().unwrap();

        let mut style_loader = StyleLoader::new();

        style_loader.add_style(Stylesheet::day_theme()).unwrap();

        Self {
            window: Window::default(),
            style_loader,
            event_loop,
            builder: Some(Box::new(content)),
        }
    }

    /// Set the default theme to night theme, this will clear all the styles
    /// that have been added before, and should therefore be called before
    /// [`App::style`].
    pub fn night_theme(mut self) -> Self {
        self.style_loader.clear();
        self.style_loader
            .add_style(Stylesheet::night_theme())
            .unwrap();
        self
    }

    /// Set the default theme to day theme, this will clear all the styles
    /// that have been added before, and should therefore be called before
    /// [`App::style`].
    pub fn day_theme(mut self) -> Self {
        self.style_loader.clear();
        self.style_loader
            .add_style(Stylesheet::day_theme())
            .unwrap();
        self
    }

    /// Set the title of the window.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.window.title = title.into();
        self
    }

    /// Add a style to the app, this can be called multiple times to add
    /// multiple styles.
    pub fn style<T>(mut self, style: T) -> Self
    where
        T: TryInto<LoadedStyleKind>,
        T::Error: Display,
    {
        #[allow(clippy::single_match)]
        match self.style_loader.add_style(style) {
            Err(err) => tracing::error!("failed to load style: {}", err),
            _ => {}
        };

        self
    }

    /// Set the size of the window.
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.window.size = UVec2::new(width, height);
        self
    }

    /// Set the width of the window.
    pub fn width(mut self, width: u32) -> Self {
        self.window.size.x = width;
        self
    }

    /// Set the height of the window.
    pub fn height(mut self, height: u32) -> Self {
        self.window.size.y = height;
        self
    }

    /// Set the window to be resizable or not.
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.window.resizable = resizable;
        self
    }

    /// Set the clear color of the window.
    pub fn clear_color(mut self, color: Color) -> Self {
        self.window.clear_color = color;
        self
    }

    /// Set the clear color of the window to transparent.
    pub fn transparent(self) -> Self {
        self.clear_color(Color::TRANSPARENT)
    }

    /// Set the icon of the window.
    pub fn icon(mut self, icon: impl Into<ImageSource>) -> Self {
        self.window.icon = Some(icon.into().load());
        self
    }
}

impl App {
    pub fn new_any_thread(content: impl FnMut(Scope) -> Node + Send + 'static) -> Self {
        let mut builder = EventLoopBuilder::with_user_event();

        #[cfg(target_os = "windows")]
        {
            use winit::platform::windows::EventLoopBuilderExtWindows;
            builder.with_any_thread(true);
        }

        #[cfg(all(target_os = "linux", feature = "x11"))]
        {
            use winit::platform::x11::EventLoopBuilderExtX11;
            builder.with_any_thread(true);
        }

        #[cfg(all(target_os = "linux", feature = "wayland"))]
        {
            use winit::platform::wayland::EventLoopBuilderExtWayland;
            builder.with_any_thread(true);
        }

        #[cfg(target_os = "macos")]
        {
            use winit::platform::macos::EventLoopBuilderExtMacOS;
            builder.with_any_thread(true);
        }

        Self::new_with_event_loop(builder.build(), content)
    }
}

impl App {
    /// Run the app.
    pub fn run(mut self) -> ! {
        let window_backend = WinitBackend::new(self.event_loop.create_proxy());
        #[cfg(feature = "wgpu")]
        let render_backend = ori_wgpu::WgpuBackend::new();

        let mut windows = Windows::new(window_backend, render_backend);
        windows.style_loader = self.style_loader;
        let ui = self.builder.take().unwrap();
        (windows.create_window(&self.event_loop, &self.window, ui)).unwrap();

        self.event_loop.run(move |event, target, control_flow| {
            *control_flow = ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(10));

            match event {
                WinitEvent::RedrawRequested(window) => {
                    if let Some(id) = windows.window_backend.id(window) {
                        windows.draw(id);
                    }
                }
                WinitEvent::MainEventsCleared
                | WinitEvent::NewEvents(StartCause::ResumeTimeReached { .. }) => {
                    windows.idle();
                }
                WinitEvent::UserEvent((window, event)) => {
                    let Some(id) = windows.window_backend.id(window) else {
                        return;
                    };

                    windows.event(target, id, &event);

                    if windows.is_empty() {
                        *control_flow = ControlFlow::Exit;
                    }
                }
                WinitEvent::WindowEvent {
                    event,
                    window_id: window,
                    ..
                } => {
                    let Some(window) = windows.window_backend.id(window) else {
                        return;
                    };

                    match event {
                        WindowEvent::Resized(size)
                        | WindowEvent::ScaleFactorChanged {
                            new_inner_size: &mut size,
                            ..
                        } => {
                            windows.resize_window(window, size.width, size.height);
                        }
                        WindowEvent::CloseRequested => {
                            windows.close_window(window);

                            if windows.is_empty() {
                                *control_flow = ControlFlow::Exit;
                            }
                        }
                        WindowEvent::CursorMoved {
                            position,
                            device_id,
                            ..
                        } => {
                            let device = convert_device_id(device_id);
                            let position = Vec2::new(position.x as f32, position.y as f32);
                            windows.pointer_moved(window, device, position);
                        }
                        WindowEvent::CursorLeft { device_id } => {
                            let device = convert_device_id(device_id);
                            windows.pointer_left(window, device);
                        }
                        WindowEvent::MouseInput {
                            button,
                            state: element_state,
                            device_id,
                            ..
                        } => {
                            let device = convert_device_id(device_id);
                            let button = convert_mouse_button(button);
                            let pressed = is_pressed(element_state);
                            windows.pointer_button(window, device, button, pressed);
                        }
                        WindowEvent::MouseWheel {
                            delta: MouseScrollDelta::LineDelta(x, y),
                            device_id,
                            ..
                        } => {
                            let device = convert_device_id(device_id);
                            let delta = Vec2::new(x, y);
                            windows.pointer_scroll(window, device, delta);
                        }
                        WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    virtual_keycode: Some(virtual_keycode),
                                    state: element_state,
                                    ..
                                },
                            ..
                        } => {
                            let key = convert_key(virtual_keycode);
                            let pressed = is_pressed(element_state);

                            if let Some(key) = key {
                                windows.key(window, key, pressed);
                            }
                        }
                        WindowEvent::ReceivedCharacter(c) => {
                            windows.text(window, String::from(c));
                        }
                        WindowEvent::ModifiersChanged(new_modifiers) => {
                            let modifiers = Modifiers {
                                shift: new_modifiers.shift(),
                                ctrl: new_modifiers.ctrl(),
                                alt: new_modifiers.alt(),
                                meta: new_modifiers.logo(),
                            };

                            windows.modifiers_changed(window, modifiers);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        });
    }
}
