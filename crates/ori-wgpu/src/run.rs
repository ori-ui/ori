use std::collections::HashMap;

use ori_core::{
    event::{Modifiers, PointerId},
    math::Vec2,
    window::Window,
};
use winit::{
    event::{Event, KeyboardInput, MouseScrollDelta, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::{
    convert::{convert_key, convert_mouse_button, is_pressed},
    render::{Render, RenderInstance},
    window::WinitWindow,
    Error,
};

use crate::App;

pub(crate) fn run<T: 'static>(mut app: App<T>) -> Result<(), Error> {
    #[cfg(feature = "tracing")]
    if let Err(err) = crate::tracing::init_tracing() {
        eprintln!("Failed to initialize tracing: {}", err);
    }

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_visible(false)
        .with_transparent(app.window.transparent)
        .build(&event_loop)?;

    let runtime = tokio::runtime::Runtime::new().unwrap();

    // SAFETY: this function will never return and the window will therefore
    // be valid for the lifetime on the RenderInstance.
    let (instance, surface) = runtime.block_on(unsafe { RenderInstance::new(&window) })?;

    let _guard = runtime.enter();

    let mut ids = HashMap::new();
    ids.insert(window.id(), app.window.id);

    let raw_window = Box::new(WinitWindow::from(window));
    let window = Window::new(raw_window, app.window);
    let render = Render::new(&instance, surface, window.width(), window.height())?;

    app.ui.add_window(app.builder, window, render);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::RedrawEventsCleared => {
                app.ui.idle();
            }
            Event::RedrawRequested(window_id) => {
                let id = ids[&window_id];
                app.ui.render(id);
            }
            Event::WindowEvent { window_id, event } => {
                let window_id = ids[&window_id];

                match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => {
                        app.ui.resized(window_id);
                    }
                    WindowEvent::CursorMoved {
                        device_id,
                        position,
                        ..
                    } => {
                        app.ui.pointer_moved(
                            window_id,
                            PointerId::from_hash(&device_id),
                            Vec2::new(position.x as f32, position.y as f32),
                        );
                    }
                    WindowEvent::CursorLeft { device_id } => {
                        (app.ui).pointer_left(window_id, PointerId::from_hash(&device_id));
                    }
                    WindowEvent::MouseInput {
                        device_id,
                        state,
                        button,
                        ..
                    } => {
                        app.ui.pointer_button(
                            window_id,
                            PointerId::from_hash(&device_id),
                            convert_mouse_button(button),
                            is_pressed(state),
                        );
                    }
                    WindowEvent::MouseWheel {
                        delta: MouseScrollDelta::LineDelta(x, y),
                        device_id,
                        ..
                    } => {
                        app.ui.pointer_scroll(
                            window_id,
                            PointerId::from_hash(&device_id),
                            Vec2::new(x, y),
                        );
                    }
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(keycode),
                                state,
                                ..
                            },
                        ..
                    } => {
                        if let Some(key) = convert_key(keycode) {
                            app.ui.keyboard_key(window_id, key, is_pressed(state));
                        }
                    }
                    WindowEvent::ReceivedCharacter(c) => {
                        app.ui.keyboard_char(window_id, c);
                    }
                    WindowEvent::ModifiersChanged(modifiers) => {
                        app.ui.modifiers_changed(Modifiers {
                            shift: modifiers.shift(),
                            ctrl: modifiers.ctrl(),
                            alt: modifiers.alt(),
                            meta: modifiers.logo(),
                        });
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    });
}
