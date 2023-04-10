use std::sync::{Arc, Mutex};

use ily_core::{AnyView, BoxConstraints, Event, Modifiers, PaintContext, PointerPress, Vec2, View};
use ily_graphics::{Frame, Rect};
use ily_reactive::{Callback, Scope};

use winit::{
    event::{Event as WinitEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::convert::{convert_mouse_button, is_pressed};

pub struct App {
    builder: Option<Box<dyn FnOnce() -> AnyView>>,
}

impl App {
    pub fn new<T: View>(mut content: impl FnMut(Scope) -> T + 'static) -> Self {
        tracing_subscriber::FmtSubscriber::builder()
            .with_max_level(tracing::Level::TRACE)
            .init();

        let builder = Box::new(move || {
            let mut view = None;

            let _disposer = Scope::new(|cx| {
                view = Some(AnyView::new_static(content(cx)));
            });

            view.unwrap()
        });

        Self {
            builder: Some(builder),
        }
    }

    pub fn run(mut self) {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title("Hello, world!")
            .build(&event_loop)
            .unwrap();
        let window = Arc::new(window);

        let request_redraw = Arc::new(Mutex::new({
            let window = window.clone();
            move || {
                tracing::trace!("redraw requested");
                window.request_redraw()
            }
        })) as Callback;

        let mut mouse_position = Vec2::ZERO;
        let mut modifiers = Modifiers::default();

        #[cfg(feature = "wgpu")]
        let mut renderer = {
            let size = window.inner_size();
            unsafe { ily_wgpu::Renderer::new(window.as_ref(), size.width, size.height) }
        };

        let builder = self.builder.take().unwrap();
        let view = builder();

        let window_size = |window: &Window| {
            Vec2::new(
                window.inner_size().width as f32,
                window.inner_size().height as f32,
            )
        };

        let window_constraints =
            move |window: &Window| BoxConstraints::new(Vec2::ZERO, window_size(window));

        let layout = move |view: &AnyView, window: &Window| -> Vec2 {
            let size = view.layout(window_constraints(window));
            view.set_rect(Rect::new(Vec2::ZERO, size));
            size
        };

        event_loop.run(move |event, _, control_flow| match event {
            WinitEvent::RedrawRequested(_) => {
                let size = layout(&view, &window);

                let mut frame = Frame::new();
                let rect = Rect::new(Vec2::ZERO, size);
                let mut cx = PaintContext {
                    frame: &mut frame,
                    request_redraw: Arc::downgrade(&request_redraw),
                    rect,
                };
                view.paint(&mut cx);

                renderer.render_frame(&frame);
            }
            WinitEvent::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(size)
                | WindowEvent::ScaleFactorChanged {
                    new_inner_size: &mut size,
                    ..
                } => {
                    renderer.resize(size.width, size.height);
                }
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::CursorMoved { position, .. } => {
                    mouse_position.x = position.x as f32;
                    mouse_position.y = position.y as f32;
                }
                WindowEvent::MouseInput { button, state, .. } => {
                    let event = PointerPress {
                        position: mouse_position,
                        button: convert_mouse_button(button),
                        pressed: is_pressed(state),
                        modifiers,
                    };

                    layout(&view, &window);
                    view.event(&Event::new(event));
                }
                _ => {}
            },
            _ => {}
        });
    }
}
