use std::collections::HashMap;

use futures_lite::future;
use ori_core::{
    event::{Modifiers, PointerId},
    math::Vec2,
    ui::Ui,
    window::Window,
};
use winit::{
    event::{Event, KeyboardInput, MouseScrollDelta, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::{
    convert::{convert_key, convert_mouse_button, is_pressed},
    render::{WgpuRender, WgpuRenderInstance},
    window::WinitWindow,
    App, Error,
};

pub(crate) fn run<T: 'static>(mut app: App<T>) -> Result<(), Error> {
    #[cfg(feature = "tracing")]
    if let Err(err) = crate::tracing::init_tracing() {
        eprintln!("Failed to initialize tracing: {}", err);
    }

    /* create the window */
    let window = WindowBuilder::new()
        .with_visible(false)
        .with_transparent(app.window.transparent)
        .build(&app.event_loop)?;

    // SAFETY: this function will never return and the window will therefore
    // be valid for the lifetime on the RenderInstance.
    let (instance, surface) = future::block_on(unsafe { WgpuRenderInstance::new(&window) })?;

    /* create the window map */
    let mut ids = HashMap::new();
    ids.insert(window.id(), app.window.id);

    /* create the initial window */
    let raw_window = Box::new(WinitWindow::from(window));
    let window = Window::new(raw_window, app.window.clone());
    let render = WgpuRender::new(&instance, surface, window.width(), window.height())?;

    app.ui.add_window(app.builder, window, render);
    app.builder = Box::new(|_| unreachable!());

    /* initialize the ui */
    app.ui.init();

    /* enter the event loop */
    enter_event_loop(app.event_loop, app.ui, ids);
}

fn enter_event_loop<T: 'static>(
    event_loop: EventLoop<()>,
    mut ui: Ui<T, WgpuRender>,
    ids: HashMap<winit::window::WindowId, ori_core::window::WindowId>,
) -> ! {
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::RedrawEventsCleared => {
                ui.idle();
            }
            Event::RedrawRequested(window_id) => {
                let id = ids[&window_id];
                ui.render(id);
            }
            Event::UserEvent(_) => {
                ui.handle_commands();
            }
            Event::WindowEvent { window_id, event } => {
                let window_id = ids[&window_id];

                match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::Resized(_) => {
                        ui.resized(window_id);
                    }
                    WindowEvent::ScaleFactorChanged { .. } => {
                        ui.rebuild_theme(window_id);
                        ui.resized(window_id);
                    }
                    WindowEvent::CursorMoved {
                        device_id,
                        position,
                        ..
                    } => {
                        ui.pointer_moved(
                            window_id,
                            PointerId::from_hash(&device_id),
                            Vec2::new(position.x as f32, position.y as f32),
                        );
                    }
                    WindowEvent::CursorLeft { device_id } => {
                        ui.pointer_left(window_id, PointerId::from_hash(&device_id));
                    }
                    WindowEvent::MouseInput {
                        device_id,
                        state,
                        button,
                        ..
                    } => {
                        ui.pointer_button(
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
                        ui.pointer_scroll(
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
                            ui.keyboard_key(window_id, key, is_pressed(state));
                        }
                    }
                    WindowEvent::ReceivedCharacter(c) => {
                        ui.keyboard_char(window_id, c);
                    }
                    WindowEvent::ModifiersChanged(modifiers) => {
                        ui.modifiers_changed(Modifiers {
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
