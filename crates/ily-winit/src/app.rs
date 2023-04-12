use std::{error::Error, sync::Arc};

use ily_core::{
    BoxConstraints, Callback, DrawContext, Event, EventContext, LayoutContext, Modifiers, Node,
    NodeState, PointerEvent, Scope, Style, Vec2, View, WeakCallback,
};
use ily_graphics::Frame;
use winit::{
    event::{Event as WinitEvent, KeyboardInput, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::convert::{convert_mouse_button, is_pressed};

pub struct App {
    style: Style,
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

        Self {
            style: Style::default(),
            builder: Some(builder),
        }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

struct AppState {
    window: Arc<Window>,
    style: Style,
    request_redraw: WeakCallback,
    mouse_position: Vec2,
    modifiers: Modifiers,
    root: Node,
    root_state: NodeState,
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
        let mut cx = EventContext {
            style: &self.style,
            state: &mut self.root_state,
            request_redraw: &self.request_redraw,
        };

        self.root.event(&mut cx, event);
    }

    fn layout(&mut self) {
        let size = self.window_size();
        let bc = BoxConstraints::new(Vec2::ZERO, size);

        let mut cx = LayoutContext {
            style: &self.style,
            text_layout: &mut self.renderer.text_layout(),
        };

        self.root.layout(&mut cx, bc);
    }

    fn draw(&mut self) {
        self.layout();

        self.frame.clear();

        let mut cx = DrawContext {
            style: &self.style,
            frame: &mut self.frame,
            state: &mut self.root_state,
            request_redraw: &self.request_redraw,
        };

        self.root.draw(&mut cx);

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
            style: self.style,
            request_redraw,
            mouse_position: Vec2::ZERO,
            modifiers: Modifiers::default(),
            root: builder(),
            root_state: NodeState::default(),
            frame: Frame::new(),
            #[cfg(feature = "wgpu")]
            renderer,
        };

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                WinitEvent::RedrawRequested(_) => {
                    state.layout();
                    state.draw();
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
