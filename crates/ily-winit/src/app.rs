use std::{
    error::Error,
    fmt::Display,
    str::FromStr,
    sync::Arc,
    time::{Duration, Instant},
};

use ily_core::{
    Callback, Event, LoadedStyleKind, Modifiers, Node, PointerEvent, Scope, Style, StyleLoader,
    Vec2, View, WeakCallback,
};
use ily_graphics::Frame;
use winit::{
    event::{Event as WinitEvent, KeyboardInput, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::convert::{convert_mouse_button, is_pressed};

const BUILTIN_STYLES: &[&str] = &[
    include_str!("../../../style/default.css"),
    include_str!("../../../style/text.css"),
    include_str!("../../../style/button.css"),
    include_str!("../../../style/checkbox.css"),
];

pub struct App {
    style_loader: StyleLoader,
    builder: Option<Box<dyn FnOnce() -> Node>>,
}

fn init_tracing() -> Result<(), Box<dyn Error>> {
    let filter = tracing_subscriber::EnvFilter::from_default_env()
        .add_directive("wgpu=warn".parse()?)
        .add_directive("naga=warn".parse()?)
        .add_directive("winit=warn".parse()?)
        .add_directive("mio=warn".parse()?);

    tracing_subscriber::fmt().with_env_filter(filter).init();

    Ok(())
}

impl App {
    pub fn new<T: View>(content: impl FnOnce(Scope) -> T + 'static) -> Self {
        init_tracing().unwrap();

        let builder = Box::new(move || {
            let mut view = None;

            let _disposer = Scope::new(|cx| {
                view = Some(content(cx));
            });

            Node::new(view.unwrap())
        });

        let mut loader = StyleLoader::new();

        for builtin in BUILTIN_STYLES {
            let default_style = Style::from_str(builtin).unwrap();
            let _ = loader.add_style(default_style);
        }

        Self {
            style_loader: loader,
            builder: Some(builder),
        }
    }

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
}

struct AppState {
    window: Arc<Window>,
    style_loader: StyleLoader,
    request_redraw: WeakCallback,
    mouse_position: Vec2,
    modifiers: Modifiers,
    root: Node,
    frame: Frame,
    #[cfg(feature = "wgpu")]
    renderer: ily_wgpu::Renderer,
}

impl AppState {
    fn window_size(&self) -> Vec2 {
        let size = self.window.inner_size();
        Vec2::new(size.width as f32, size.height as f32)
    }

    fn event(&mut self, event: &Event) {
        (self.root).event_root(self.style_loader.style(), &self.request_redraw, event);
    }

    fn layout(&mut self) {
        let style = self.style_loader.style();
        let size = self.window_size();
        let text_layout = &mut self.renderer.text_layout();
        (self.root).layout_root(style, text_layout, size, &self.request_redraw);
    }

    fn draw(&mut self) {
        self.layout();

        self.frame.clear();
        let style = self.style_loader.style();
        (self.root).draw_root(style, &mut self.frame, &self.request_redraw);

        #[cfg(feature = "wgpu")]
        self.renderer.render_frame(&self.frame);
    }
}

impl App {
    pub fn run(mut self) {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title("Hello, world!")
            .build(&event_loop)
            .unwrap();
        let window = Arc::new(window);

        let request_redraw = Callback::new({
            let window = window.clone();
            move |_| {
                tracing::trace!("redraw requested");
                window.request_redraw()
            }
        });
        let request_redraw = request_redraw.downgrade();

        #[cfg(feature = "wgpu")]
        let renderer = {
            let size = window.inner_size();
            unsafe { ily_wgpu::Renderer::new(window.as_ref(), size.width, size.height) }
        };

        let builder = self.builder.take().unwrap();
        let mut state = AppState {
            window,
            style_loader: self.style_loader,
            request_redraw,
            mouse_position: Vec2::ZERO,
            modifiers: Modifiers::default(),
            root: builder(),
            frame: Frame::new(),
            #[cfg(feature = "wgpu")]
            renderer,
        };

        let update_time = Duration::from_secs_f32(1.0 / 2.0);

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::WaitUntil(Instant::now() + update_time);

            match event {
                WinitEvent::RedrawRequested(_) => {
                    state.draw();
                }
                WinitEvent::MainEventsCleared => match state.style_loader.reload() {
                    Ok(reload) if reload => {
                        tracing::info!("style reloaded");
                        state.draw();
                    }
                    Err(err) => tracing::error!("failed to reload style: {}", err),
                    _ => {}
                },
                WinitEvent::WindowEvent { event, .. } => match event {
                    WindowEvent::Resized(size)
                    | WindowEvent::ScaleFactorChanged {
                        new_inner_size: &mut size,
                        ..
                    } => {
                        #[cfg(feature = "wgpu")]
                        state.renderer.resize(size.width, size.height);
                    }
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::CursorMoved { position, .. } => {
                        state.mouse_position.x = position.x as f32;
                        state.mouse_position.y = position.y as f32;

                        let event = PointerEvent {
                            position: state.mouse_position,
                            modifiers: state.modifiers,
                            ..Default::default()
                        };

                        state.layout();
                        state.event(&Event::new(event));
                    }
                    WindowEvent::MouseInput {
                        button,
                        state: element_state,
                        ..
                    } => {
                        let event = PointerEvent {
                            position: state.mouse_position,
                            button: Some(convert_mouse_button(button)),
                            pressed: is_pressed(element_state),
                            modifiers: state.modifiers,
                            ..Default::default()
                        };

                        state.event(&Event::new(event));
                    }
                    WindowEvent::KeyboardInput {
                        input: KeyboardInput { .. },
                        ..
                    } => {}
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
