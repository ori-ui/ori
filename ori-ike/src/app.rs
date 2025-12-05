use std::{any::Any, sync::mpsc::Receiver, time::Instant};

use ike::Size;
use ori::AsyncContext;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

use crate::{Context, Effect, context::Event, key::convert_winit_key};

pub struct App {}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self {}
    }

    pub fn init_log() {
        let mut filter = EnvFilter::default();

        if cfg!(debug_assertions) {
            filter = filter.add_directive(tracing::Level::DEBUG.into());
        }

        if let Ok(env) = std::env::var("RUST_LOG")
            && let Ok(env) = env.parse()
        {
            filter = filter.add_directive(env);
        }

        let subscriber = tracing_subscriber::registry().with(filter);

        #[cfg(not(target_arch = "wasm32"))]
        let subscriber = subscriber.with(tracing_subscriber::fmt::layer());

        let _ = tracing::subscriber::set_global_default(subscriber);
    }

    pub fn run<T, V>(self, data: &mut T, mut ui: impl FnMut(&mut T) -> V + 'static)
    where
        V: Effect<T> + 'static,
        V::State: 'static,
    {
        Self::init_log();

        let event_loop = EventLoop::with_user_event().build().unwrap();

        #[cfg(feature = "vulkan")]
        let vulkan = unsafe { crate::vulkan::VulkanContext::new(&event_loop) };

        let rt;
        let _rt_guard = if tokio::runtime::Handle::try_current().is_err() {
            rt = Some(tokio::runtime::Runtime::new().unwrap());
            Some(rt.as_ref().unwrap().enter())
        } else {
            None
        };

        let runtime = tokio::runtime::Handle::current();

        let mut build: UiBuilder<T> = Box::new(move |data| Box::new(ui(data)));
        let view = build(data);

        #[cfg(feature = "vulkan")]
        let mut painter = crate::skia::SkiaPainter::new();
        #[cfg(feature = "vulkan")]
        painter.load_font(
            include_bytes!("InterVariable.ttf"),
            None,
        );

        let (sender, receiver) = std::sync::mpsc::channel();
        let context = Context {
            app: ike::App::new(),
            proxy: event_loop.create_proxy(),
            entries: Vec::new(),
            sender,

            use_type_names_unsafe: false,
        };

        let mut state = AppState {
            data,

            build,
            view,
            state: None,

            runtime,
            receiver,

            #[cfg(feature = "vulkan")]
            painter,
            context,
            windows: Vec::new(),

            #[cfg(feature = "vulkan")]
            vulkan,
        };

        event_loop.run_app(&mut state).unwrap();
    }
}

struct AppState<'a, T> {
    data: &'a mut T,

    build: UiBuilder<T>,
    view:  AnyEffect<T>,
    state: Option<Box<dyn Any>>,

    runtime:  tokio::runtime::Handle,
    receiver: Receiver<Event>,

    #[cfg(feature = "vulkan")]
    painter: crate::skia::SkiaPainter,
    context: Context,
    windows: Vec<WindowState>,

    #[cfg(feature = "vulkan")]
    vulkan: crate::vulkan::VulkanContext,
}

struct WindowState {
    animate: Option<Instant>,

    #[cfg(feature = "vulkan")]
    vulkan: crate::vulkan::VulkanWindow,

    id:       ike::WindowId,
    window:   Window,
    min_size: Size,
    max_size: Size,
}

impl<T> ApplicationHandler for AppState<'_, T> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }

        let (_, state) = ori::View::build(
            &mut self.view,
            &mut self.context,
            self.data,
        );

        self.state = Some(state);
        self.update_windows(event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = self.windows.iter_mut().find(|w| w.window.id() == window_id) else {
            return;
        };

        match event {
            WindowEvent::RedrawRequested => {
                if let Some(animate) = window.animate.take() {
                    let delta_time = animate.elapsed();
                    self.context.app.animate(window.id, delta_time);

                    if self.context.app.window_needs_animate(window.id) {
                        window.animate = Some(Instant::now());
                        window.window.request_redraw();
                    }
                }

                #[cfg(feature = "vulkan")]
                let new_window_size = unsafe {
                    let scale = window.window.scale_factor();

                    window.vulkan.draw(&window.window, |canvas| {
                        let ike = self.context.app.get_window(window.id).unwrap();

                        canvas.reset_matrix();
                        canvas.clear(skia_safe::Color::from_argb(
                            f32::round(ike.color.a * 255.0) as u8,
                            f32::round(ike.color.r * 255.0) as u8,
                            f32::round(ike.color.g * 255.0) as u8,
                            f32::round(ike.color.b * 255.0) as u8,
                        ));
                        canvas.scale((scale as f32, scale as f32));

                        let mut skia_canvas = crate::skia::SkiaCanvas {
                            painter: &mut self.painter,
                            canvas,
                        };

                        self.context.app.draw(window.id, &mut skia_canvas)
                    })
                };

                if let Some(size) = new_window_size.flatten() {
                    let size = LogicalSize::new(size.width, size.height);

                    window.window.set_min_inner_size(Some(size));
                    window.window.set_max_inner_size(Some(size));
                }
            }

            WindowEvent::Resized(..) | WindowEvent::ScaleFactorChanged { .. } => {
                #[cfg(feature = "vulkan")]
                window.resize();

                let scale = window.window.scale_factor();
                let size = window.window.inner_size().to_logical(scale);

                let size = ike::Size::new(size.width, size.height);

                match event {
                    WindowEvent::Resized(..) => {
                        self.context.app.window_resized(window.id, size);
                    }

                    WindowEvent::ScaleFactorChanged { .. } => {
                        (self.context.app).window_scaled(window.id, scale as f32, size);
                    }

                    _ => unreachable!(),
                }
            }

            WindowEvent::Focused(is_focused) => {
                self.context.app.window_focused(window.id, is_focused);
            }

            WindowEvent::CursorEntered { device_id } => {
                let id = ike::PointerId::from_hash(device_id);
                self.context.app.pointer_entered(window.id, id);
            }

            WindowEvent::CursorLeft { device_id } => {
                let id = ike::PointerId::from_hash(device_id);
                self.context.app.pointer_left(window.id, id);
            }

            WindowEvent::CursorMoved {
                device_id,
                position,
            } => {
                let position = position.to_logical(window.window.scale_factor());
                let position = ike::Point::new(position.x, position.y);

                self.context.app.pointer_moved(
                    window.id,
                    ike::PointerId::from_hash(device_id),
                    position,
                );
            }

            WindowEvent::MouseInput {
                device_id,
                state,
                button,
            } => {
                let button = match button {
                    MouseButton::Left => ike::PointerButton::Primary,
                    MouseButton::Right => ike::PointerButton::Secondary,
                    MouseButton::Middle => ike::PointerButton::Tertiary,
                    MouseButton::Back => ike::PointerButton::Backward,
                    MouseButton::Forward => ike::PointerButton::Forward,
                    MouseButton::Other(i) => ike::PointerButton::Other(i),
                };

                let pressed = matches!(state, ElementState::Pressed);

                self.context.app.pointer_button(
                    window.id,
                    ike::PointerId::from_hash(device_id),
                    button,
                    pressed,
                );
            }

            WindowEvent::KeyboardInput { event, .. } => {
                let pressed = matches!(event.state, ElementState::Pressed);

                self.context.app.key_press(
                    window.id,
                    convert_winit_key(event.logical_key),
                    event.repeat,
                    event.text.as_deref(),
                    pressed,
                );
            }

            WindowEvent::ModifiersChanged(mods) => {
                let mut modifiers = ike::Modifiers::empty();

                if mods.state().shift_key() {
                    modifiers |= ike::Modifiers::SHIFT;
                }

                if mods.state().control_key() {
                    modifiers |= ike::Modifiers::CONTROL;
                }

                if mods.state().alt_key() {
                    modifiers |= ike::Modifiers::ALT;
                }

                if mods.state().super_key() {
                    modifiers |= ike::Modifiers::META;
                }

                self.context.app.modifiers_changed(window.id, modifiers);
            }

            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            _ => {}
        }

        self.handle_events();
        self.update_windows(event_loop);
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, _event: ()) {
        self.handle_events();
        self.update_windows(event_loop);
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.painter.cleanup();
    }
}

type AnyEffect<T> = Box<dyn ori::AnyView<Context, T, ori::NoElement>>;
type UiBuilder<T> = Box<dyn FnMut(&mut T) -> AnyEffect<T>>;

impl<T> AppState<'_, T> {
    fn handle_events(&mut self) {
        for event in self.receiver.try_iter() {
            match event {
                Event::Rebuild => {
                    let mut view = (self.build)(self.data);
                    ori::View::rebuild(
                        &mut view,
                        &mut ori::NoElement,
                        self.state.as_mut().unwrap(),
                        &mut self.context,
                        self.data,
                        &mut self.view,
                    );

                    self.view = view;
                }

                Event::Event(mut event) => {
                    let action = ori::View::event(
                        &mut self.view,
                        &mut ori::NoElement,
                        self.state.as_mut().unwrap(),
                        &mut self.context,
                        self.data,
                        &mut event,
                    );

                    self.context.send_action(action);
                }

                Event::Spawn(future) => {
                    self.runtime.spawn(future);
                }
            }
        }
    }

    fn update_windows(&mut self, #[allow(unused_variables)] event_loop: &ActiveEventLoop) {
        for desc in self.context.app.windows() {
            match self.windows.iter_mut().find(|w| w.id == desc.id()) {
                Some(window) => window.update(desc),

                None => {
                    #[cfg(feature = "vulkan")]
                    self.windows.push(WindowState::new(
                        &mut self.vulkan,
                        event_loop,
                        desc,
                    ));
                }
            }
        }

        for window in &mut self.windows {
            if self.context.app.window_needs_animate(window.id) && window.animate.is_none() {
                window.animate = Some(Instant::now());
                window.window.request_redraw();
            }

            if self.context.app.window_needs_draw(window.id) {
                window.window.request_redraw();
            }
        }

        self.windows.retain(|state| {
            let Some(_window) = self.context.app.get_window(state.id) else {
                return false;
            };

            true
        });
    }
}

impl WindowState {
    fn new(
        #[cfg(feature = "vulkan")] vulkan: &mut crate::vulkan::VulkanContext,
        event_loop: &ActiveEventLoop,
        desc: &ike::Window,
    ) -> Self {
        use winit::dpi::LogicalSize;

        let size = match desc.sizing {
            ike::WindowSizing::FitContent => LogicalSize::new(
                desc.current_size().width,
                desc.current_size().height,
            ),
            ike::WindowSizing::Resizable { default_size, .. } => {
                LogicalSize::new(default_size.width, default_size.height)
            }
        };

        let min_size = match desc.sizing {
            ike::WindowSizing::FitContent => size,
            ike::WindowSizing::Resizable { min_size, .. } => {
                LogicalSize::new(min_size.width, min_size.height)
            }
        };

        let max_size = match desc.sizing {
            ike::WindowSizing::FitContent => size,
            ike::WindowSizing::Resizable { max_size, .. } => {
                LogicalSize::new(max_size.width, max_size.height)
            }
        };

        let attributes = Window::default_attributes()
            .with_title(&desc.title)
            .with_visible(desc.visible)
            .with_decorations(desc.decorated)
            .with_transparent(true)
            .with_min_inner_size(min_size)
            .with_max_inner_size(max_size)
            .with_inner_size(size)
            .with_resizable(matches!(
                desc.sizing,
                ike::WindowSizing::Resizable { .. }
            ));

        let window = event_loop.create_window(attributes).unwrap();

        #[cfg(feature = "vulkan")]
        let vulkan = unsafe { crate::vulkan::VulkanWindow::new(vulkan, &window) };

        let (min_size, max_size) = match desc.sizing {
            ike::WindowSizing::FitContent => (Size::default(), Size::default()),
            ike::WindowSizing::Resizable {
                min_size, max_size, ..
            } => (min_size, max_size),
        };

        Self {
            animate: None,
            id: desc.id(),
            vulkan,
            window,
            min_size,
            max_size,
        }
    }

    fn update(&mut self, desc: &ike::Window) {
        if self.window.title() != desc.title {
            self.window.set_title(&desc.title);
        }

        if let ike::WindowSizing::Resizable {
            min_size, max_size, ..
        } = desc.sizing
        {
            if self.min_size != min_size {
                self.window.set_min_inner_size(Some(LogicalSize::new(
                    min_size.width,
                    min_size.height,
                )));
                self.min_size = min_size;
            }

            if self.max_size != max_size {
                self.window.set_max_inner_size(Some(LogicalSize::new(
                    max_size.width,
                    max_size.height,
                )));
                self.max_size = max_size;
            }
        }

        self.window.set_decorations(desc.decorated);
        self.window.set_visible(desc.visible);
    }

    #[cfg(feature = "vulkan")]
    fn resize(&mut self) {
        let size = self.window.inner_size();

        unsafe {
            (self.vulkan).recreate_swapchain(size.width, size.height);
        }
    }
}
