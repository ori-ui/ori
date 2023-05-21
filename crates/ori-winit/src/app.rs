use std::{
    error::Error,
    fmt::Display,
    sync::Arc,
    time::{Duration, Instant},
};

use ori_core::{
    CallbackEmitter, Cursor, Event, EventEmitter, EventSink, ImageCache, KeyboardEvent,
    LoadedStyleKind, Modifiers, Node, PointerEvent, RequestRedrawEvent, Runtime, Scope,
    SetWindowIconEvent, SetWindowTitleEvent, StyleCache, StyleLoader, Stylesheet, Task, Vec2, View,
    WindowResizeEvent,
};
use ori_graphics::{Color, Frame};
use winit::{
    dpi::LogicalSize,
    error::OsError,
    event::{Event as WinitEvent, KeyboardInput, MouseScrollDelta, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder, EventLoopProxy},
    window::{Icon, Window, WindowBuilder},
};

use crate::convert::{
    convert_cursor_icon, convert_device_id, convert_key, convert_mouse_button, is_pressed,
};

struct EventLoopSender(EventLoopProxy<Event>);

impl EventEmitter for EventLoopSender {
    fn send_event(&mut self, event: Event) {
        let _ = self.0.send_event(event);
    }
}

fn initialize_log() -> Result<(), Box<dyn Error>> {
    use tracing_subscriber::layer::SubscriberExt;

    let filter = tracing_subscriber::EnvFilter::from_default_env()
        .add_directive("wgpu=warn".parse()?)
        .add_directive("naga=warn".parse()?)
        .add_directive("winit=warn".parse()?)
        .add_directive("mio=warn".parse()?);

    let subscriber = tracing_subscriber::registry().with(filter);
    let subscriber = subscriber.with(tracing_subscriber::fmt::Layer::default());

    #[cfg(feature = "tracy")]
    let subscriber = subscriber.with(tracing_tracy::TracyLayer::new());

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}

/// A app using [`winit`] as the windowing backend.
pub struct App {
    title: String,
    size: Vec2,
    reziseable: bool,
    clear_color: Color,
    event_loop: EventLoop<Event>,
    style_loader: StyleLoader,
    builder: Option<Box<dyn FnOnce(&EventSink, &CallbackEmitter<Event>) -> Node>>,
}

impl App {
    /// Create a new [`App`] with the given content.
    pub fn new<T: View>(content: impl FnOnce(Scope) -> T + 'static) -> Self {
        initialize_log().unwrap();

        let builder = Box::new(
            move |event_sink: &EventSink, event_callbacks: &CallbackEmitter<Event>| {
                let scope = Scope::new(event_sink.clone(), event_callbacks.clone());
                Node::new(content(scope))
            },
        );

        let mut style_loader = StyleLoader::new();

        style_loader.add_style(Stylesheet::day_theme()).unwrap();

        let event_loop = EventLoopBuilder::<Event>::with_user_event().build();

        Self {
            title: "Ori App".to_string(),
            size: Vec2::new(800.0, 600.0),
            reziseable: true,
            clear_color: Color::WHITE,
            event_loop,
            style_loader,
            builder: Some(builder),
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
        self.title = title.into();
        self
    }

    /// Add a style to the app, this can be called multiple times to add
    /// multiple styles.
    pub fn style<T>(mut self, style: T) -> Self
    where
        T: TryInto<LoadedStyleKind>,
        T::Error: Display,
    {
        match self.style_loader.add_style(style) {
            Err(err) => tracing::error!("failed to load style: {}", err),
            _ => {}
        };

        self
    }

    /// Set the size of the window.
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.size = Vec2::new(width, height);
        self
    }

    /// Set the width of the window.
    pub fn width(mut self, width: f32) -> Self {
        self.size.x = width;
        self
    }

    /// Set the height of the window.
    pub fn height(mut self, height: f32) -> Self {
        self.size.y = height;
        self
    }

    /// Set the window to be resizable or not.
    pub fn reziseable(mut self, reziseable: bool) -> Self {
        self.reziseable = reziseable;
        self
    }

    /// Set the clear color of the window.
    pub fn clear_color(mut self, color: Color) -> Self {
        self.clear_color = color;
        self
    }

    /// Set the clear color of the window to transparent.
    pub fn transparent(self) -> Self {
        self.clear_color(Color::TRANSPARENT)
    }

    /// Create an [`EventSink`] that can be used to send events to the app.
    pub fn event_sink(&self) -> EventSink {
        EventSink::new(EventLoopSender(self.event_loop.create_proxy()))
    }

    fn window(&self) -> Result<Window, OsError> {
        let size = LogicalSize::new(self.size.x, self.size.y);

        let builder = WindowBuilder::new()
            .with_title(&self.title)
            .with_inner_size(size)
            .with_resizable(self.reziseable)
            .with_transparent(self.clear_color.is_translucent());

        builder.build(&self.event_loop)
    }
}

struct AppState {
    window: Arc<Window>,
    style_loader: StyleLoader,
    mouse_position: Vec2,
    modifiers: Modifiers,
    root: Node,
    frame: Frame,
    clear_color: Color,
    event_sink: EventSink,
    event_callbacks: CallbackEmitter<Event>,
    image_cache: ImageCache,
    style_cache: StyleCache,
    cursor_icon: Cursor,
    #[cfg(feature = "wgpu")]
    renderer: ori_wgpu::WgpuRenderer,
}

impl AppState {
    fn clean(&mut self) {
        let icon = convert_cursor_icon(self.cursor_icon);
        self.window.set_cursor_icon(icon);
        self.image_cache.clean();
    }

    fn resize(&mut self, width: u32, heigth: u32) {
        #[cfg(feature = "wgpu")]
        self.renderer.resize(width, heigth);

        let size = Vec2::new(width as f32, heigth as f32);
        self.event(&Event::new(WindowResizeEvent::new(size)));
    }

    #[tracing::instrument(skip(self, event))]
    fn event(&mut self, event: &Event) {
        self.event_callbacks.emit(&event);

        self.cursor_icon = Cursor::Default;

        ori_core::delay_effects(|| {
            self.root.event_root(
                self.style_loader.style(),
                &mut self.style_cache,
                &self.renderer,
                &self.event_sink,
                event,
                &mut self.image_cache,
                &mut self.cursor_icon,
            );
        });

        self.clean();
    }

    #[tracing::instrument(skip(self))]
    fn layout(&mut self) {
        self.event(&Event::new(()));

        let style = self.style_loader.style();

        self.cursor_icon = Cursor::Default;
        ori_core::delay_effects(|| {
            self.root.layout_root(
                style,
                &mut self.style_cache,
                &self.renderer,
                &self.event_sink,
                &mut self.image_cache,
                &mut self.cursor_icon,
            );
        });

        self.clean();
    }

    #[tracing::instrument(skip(self))]
    fn draw(&mut self) {
        #[cfg(feature = "tracy")]
        tracing_tracy::client::frame_mark();

        self.layout();

        self.frame.clear();
        let style = self.style_loader.style();

        self.cursor_icon = Cursor::Default;

        ori_core::delay_effects(|| {
            self.root.draw_root(
                style,
                &mut self.style_cache,
                &mut self.frame,
                &self.renderer,
                &self.event_sink,
                &mut self.image_cache,
                &mut self.cursor_icon,
            );
        });

        self.clean();

        #[cfg(feature = "wgpu")]
        self.renderer.render_frame(&self.frame, self.clear_color);
    }
}

impl App {
    /// Run the app.
    pub fn run(mut self) -> ! {
        let window = Arc::new(self.window().unwrap());
        let event_sink = self.event_sink();

        #[cfg(feature = "wgpu")]
        let renderer = {
            let size = window.inner_size();
            unsafe { ori_wgpu::WgpuRenderer::new(window.as_ref(), size.width, size.height) }
        };

        let event_callbacks = CallbackEmitter::new();
        let builder = self.builder.take().unwrap();
        let mut state = AppState {
            window: window.clone(),
            style_loader: self.style_loader,
            mouse_position: Vec2::ZERO,
            modifiers: Modifiers::default(),
            root: builder(&event_sink, &event_callbacks),
            frame: Frame::new(),
            clear_color: self.clear_color,
            event_sink: event_sink.clone(),
            event_callbacks,
            image_cache: ImageCache::new(),
            style_cache: StyleCache::new(),
            cursor_icon: Cursor::Default,
            #[cfg(feature = "wgpu")]
            renderer,
        };

        self.event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(10));

            match event {
                WinitEvent::RedrawRequested(_) => {
                    state.draw();
                }
                WinitEvent::MainEventsCleared
                | WinitEvent::NewEvents(StartCause::ResumeTimeReached { .. }) => {
                    match state.style_loader.reload() {
                        Ok(reload) if reload => {
                            tracing::info!("style reloaded");
                            window.request_redraw();
                            state.style_cache.clear();
                        }
                        Err(err) => tracing::error!("failed to reload style: {}", err),
                        _ => {}
                    }
                }
                WinitEvent::UserEvent(event) => {
                    // poll awoken task
                    if let Some(task) = event.get::<Task>() {
                        unsafe { task.poll() };
                        return;
                    }

                    // set window title
                    if let Some(event) = event.get::<SetWindowTitleEvent>() {
                        window.set_title(&event.title);
                        return;
                    }

                    // set window icon
                    if let Some(event) = event.get::<SetWindowIconEvent>() {
                        // if icon is None, remove the icon
                        let Some(icon) = event.icon.as_ref() else {
                            window.set_window_icon(None);
                            return;
                        };

                        let pixels = icon.pixels().to_vec();
                        let icon = Icon::from_rgba(pixels, icon.width(), icon.height());

                        match icon {
                            Ok(icon) => window.set_window_icon(Some(icon)),
                            Err(err) => tracing::error!("failed to set window icon: {}", err),
                        }

                        return;
                    }

                    // request redraw
                    if event.is::<RequestRedrawEvent>() {
                        window.request_redraw();
                        return;
                    }

                    state.event(&event);
                }
                WinitEvent::WindowEvent { event, .. } => match event {
                    WindowEvent::Resized(size)
                    | WindowEvent::ScaleFactorChanged {
                        new_inner_size: &mut size,
                        ..
                    } => {
                        #[cfg(feature = "wgpu")]
                        state.resize(size.width, size.height);
                    }
                    WindowEvent::CloseRequested => {
                        state.root = Node::empty();
                        Runtime::stop_global();

                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::CursorMoved {
                        position,
                        device_id,
                        ..
                    } => {
                        state.mouse_position.x = position.x as f32;
                        state.mouse_position.y = position.y as f32;

                        let event = PointerEvent {
                            id: convert_device_id(device_id),
                            position: state.mouse_position,
                            modifiers: state.modifiers,
                            ..Default::default()
                        };

                        state.event(&Event::new(event));
                    }
                    WindowEvent::CursorLeft { device_id } => {
                        let event = PointerEvent {
                            id: convert_device_id(device_id),
                            position: state.mouse_position,
                            left: true,
                            modifiers: state.modifiers,
                            ..Default::default()
                        };

                        state.event(&Event::new(event));
                    }
                    WindowEvent::MouseInput {
                        button,
                        state: element_state,
                        device_id,
                        ..
                    } => {
                        let event = PointerEvent {
                            id: convert_device_id(device_id),
                            position: state.mouse_position,
                            button: Some(convert_mouse_button(button)),
                            pressed: is_pressed(element_state),
                            modifiers: state.modifiers,
                            ..Default::default()
                        };

                        state.event(&Event::new(event));
                    }
                    WindowEvent::MouseWheel {
                        delta: MouseScrollDelta::LineDelta(x, y),
                        device_id,
                        ..
                    } => {
                        let event = PointerEvent {
                            id: convert_device_id(device_id),
                            position: state.mouse_position,
                            scroll_delta: Vec2::new(x, y),
                            modifiers: state.modifiers,
                            ..Default::default()
                        };

                        state.event(&Event::new(event));
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
                        let event = KeyboardEvent {
                            key: convert_key(virtual_keycode),
                            pressed: is_pressed(element_state),
                            modifiers: state.modifiers,
                            ..Default::default()
                        };

                        state.event(&Event::new(event));
                    }
                    WindowEvent::ReceivedCharacter(c) => {
                        let event = KeyboardEvent {
                            text: Some(c),
                            modifiers: state.modifiers,
                            ..Default::default()
                        };

                        state.event(&Event::new(event));
                    }
                    WindowEvent::ModifiersChanged(new_modifiers) => {
                        state.modifiers = Modifiers {
                            shift: new_modifiers.shift(),
                            ctrl: new_modifiers.ctrl(),
                            alt: new_modifiers.alt(),
                            meta: new_modifiers.logo(),
                        };
                    }
                    _ => {}
                },
                _ => {}
            }
        });
    }
}
