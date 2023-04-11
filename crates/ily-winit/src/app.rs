use std::{error::Error, sync::Arc};

use ily_core::{
    BoxConstraints, Callback, Child, Event, Modifiers, PointerEvent, Scope, Vec2, View,
};
use ily_graphics::Frame;
use winit::{
    event::{Event as WinitEvent, KeyboardInput, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::convert::{convert_mouse_button, is_pressed};

pub struct App {
    builder: Option<Box<dyn FnOnce() -> Child>>,
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

            Child::new(view.unwrap())
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

        let request_redraw = Callback::new({
            let window = window.clone();
            move || {
                tracing::trace!("redraw requested");
                window.request_redraw()
            }
        });
        let request_redraw = request_redraw.downgrade();

        let mut mouse_position = Vec2::ZERO;
        let mut modifiers = Modifiers::default();

        let builder = self.builder.take().unwrap();
        let view = builder();

        let mut frame = Frame::new();

        #[cfg(feature = "wgpu")]
        let mut renderer = {
            let size = window.inner_size();
            unsafe { ily_wgpu::Renderer::new(window.as_ref(), size.width, size.height) }
        };

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                WinitEvent::RedrawRequested(_) => {
                    let size = window.inner_size();
                    let bc = BoxConstraints::window(size.width, size.height);
                    view.layout(&renderer.text_layout(), bc);

                    frame.clear();
                    view.draw(&mut frame, &request_redraw);

                    #[cfg(feature = "wgpu")]
                    renderer.render_frame(&frame);
                }
                WinitEvent::WindowEvent { event, .. } => match event {
                    WindowEvent::Resized(size)
                    | WindowEvent::ScaleFactorChanged {
                        new_inner_size: &mut size,
                        ..
                    } => {
                        #[cfg(feature = "wgpu")]
                        renderer.resize(size.width, size.height);
                    }
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::CursorMoved { position, .. } => {
                        mouse_position.x = position.x as f32;
                        mouse_position.y = position.y as f32;

                        let event = PointerEvent {
                            position: mouse_position,
                            modifiers,
                            ..Default::default()
                        };

                        let size = window.inner_size();
                        let bc = BoxConstraints::window(size.width, size.height);
                        view.layout(&renderer.text_layout(), bc);

                        view.event(&Event::new(event), &request_redraw);
                    }
                    WindowEvent::MouseInput { button, state, .. } => {
                        let event = PointerEvent {
                            position: mouse_position,
                            button: Some(convert_mouse_button(button)),
                            pressed: is_pressed(state),
                            modifiers,
                            ..Default::default()
                        };

                        let size = window.inner_size();
                        let bc = BoxConstraints::window(size.width, size.height);
                        view.layout(&renderer.text_layout(), bc);

                        view.event(&Event::new(event), &request_redraw);
                    }
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state,
                                virtual_keycode,
                                ..
                            },
                        ..
                    } => {}
                    WindowEvent::ModifiersChanged(new_modifiers) => {
                        modifiers = Modifiers {
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
