use std::{error::Error, fmt::Display, str::FromStr, sync::Arc};

use ily_core::{
    Event, EventSender, EventSink, KeyboardEvent, LoadedStyleKind, Modifiers, Node, PointerEvent,
    RequestRedrawEvent, Scope, StyleLoader, Stylesheet, Vec2, View,
};
use ily_graphics::{Color, Frame};
use winit::{
    dpi::LogicalSize,
    error::OsError,
    event::{Event as WinitEvent, KeyboardInput, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder, EventLoopProxy},
    window::{Window, WindowBuilder},
};

use crate::convert::{convert_key, convert_mouse_button, is_pressed};

const BUILTIN_STYLES: &[&str] = &[
    include_str!("../../../style/default.css"),
    include_str!("../../../style/text.css"),
    include_str!("../../../style/text-input.css"),
    include_str!("../../../style/button.css"),
    include_str!("../../../style/checkbox.css"),
    include_str!("../../../style/knob.css"),
];

struct EventLoopSender(EventLoopProxy<Event>);

impl EventSender for EventLoopSender {
    fn send_event(&self, event: Event) {
        let _ = self.0.send_event(event);
    }
}

fn initialize_log() -> Result<(), Box<dyn Error>> {
    let filter = tracing_subscriber::EnvFilter::from_default_env()
        .add_directive("wgpu=warn".parse()?)
        .add_directive("naga=warn".parse()?)
        .add_directive("winit=warn".parse()?)
        .add_directive("mio=warn".parse()?);

    tracing_subscriber::fmt().with_env_filter(filter).init();

    Ok(())
}

pub struct App {
    title: String,
    size: Vec2,
    reziseable: bool,
    clear_color: Color,
    event_loop: EventLoop<Event>,
    style_loader: StyleLoader,
    builder: Option<Box<dyn FnOnce() -> Node>>,
}

impl App {
    pub fn new<T: View>(content: impl FnOnce(Scope) -> T + 'static) -> Self {
        initialize_log().unwrap();

        let builder = Box::new(move || {
            let mut view = None;

            let _disposer = Scope::new(|cx| {
                view = Some(content(cx));
            });

            Node::new(view.unwrap())
        });

        let mut style_loader = StyleLoader::new();

        for builtin in BUILTIN_STYLES {
            let default_style = Stylesheet::from_str(builtin).unwrap();
            let _ = style_loader.add_style(default_style);
        }

        let event_loop = EventLoopBuilder::<Event>::with_user_event().build();

        Self {
            title: "Ily App".to_string(),
            size: Vec2::new(800.0, 600.0),
            reziseable: true,
            clear_color: Color::WHITE,
            event_loop,
            style_loader,
            builder: Some(builder),
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
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

    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.size = Vec2::new(width, height);
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.size.x = width;
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.size.y = height;
        self
    }

    pub fn reziseable(mut self, reziseable: bool) -> Self {
        self.reziseable = reziseable;
        self
    }

    pub fn clear_color(mut self, color: Color) -> Self {
        self.clear_color = color;
        self
    }

    pub fn transparent(self) -> Self {
        self.clear_color(Color::TRANSPARENT)
    }

    pub fn event_sink(&self) -> EventSink {
        EventSink::new(EventLoopSender(self.event_loop.create_proxy()))
    }

    fn window(&self) -> Result<Window, OsError> {
        let size = LogicalSize::new(self.size.x, self.size.y);

        WindowBuilder::new()
            .with_title(&self.title)
            .with_inner_size(size)
            .with_resizable(self.reziseable)
            .with_transparent(self.clear_color.is_translucent())
            .build(&self.event_loop)
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
    #[cfg(feature = "wgpu")]
    renderer: ily_wgpu::WgpuRenderer,
}

impl AppState {
    fn window_size(&self) -> Vec2 {
        let size = self.window.inner_size();
        Vec2::new(size.width as f32, size.height as f32)
    }

    fn event(&mut self, event: &Event) {
        self.root.event_root(
            self.style_loader.style(),
            &self.renderer,
            &self.event_sink,
            event,
        );
    }

    fn layout(&mut self) {
        let style = self.style_loader.style();
        let size = self.window_size();
        (self.root).layout_root(style, &self.renderer, size, &self.event_sink);
    }

    fn draw(&mut self) {
        self.layout();

        self.frame.clear();
        let style = self.style_loader.style();
        (self.root).draw_root(style, &mut self.frame, &self.renderer, &self.event_sink);

        #[cfg(feature = "wgpu")]
        self.renderer.render_frame(&self.frame, self.clear_color);
    }
}

impl App {
    pub fn run(mut self) {
        let window = Arc::new(self.window().unwrap());
        let event_sink = self.event_sink();

        #[cfg(feature = "wgpu")]
        let renderer = {
            let size = window.inner_size();
            unsafe { ily_wgpu::WgpuRenderer::new(window.as_ref(), size.width, size.height) }
        };

        let builder = self.builder.take().unwrap();
        let mut state = AppState {
            window: window.clone(),
            style_loader: self.style_loader,
            mouse_position: Vec2::ZERO,
            modifiers: Modifiers::default(),
            root: builder(),
            frame: Frame::new(),
            clear_color: self.clear_color,
            event_sink: event_sink.clone(),
            #[cfg(feature = "wgpu")]
            renderer,
        };

        self.event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                WinitEvent::RedrawRequested(_) => {
                    state.draw();
                }
                WinitEvent::MainEventsCleared => match state.style_loader.reload() {
                    Ok(reload) if reload => {
                        tracing::info!("style reloaded");
                        window.request_redraw();
                    }
                    Err(err) => tracing::error!("failed to reload style: {}", err),
                    _ => {}
                },
                WinitEvent::UserEvent(event) => {
                    if event.is::<RequestRedrawEvent>() {
                        window.request_redraw();
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
