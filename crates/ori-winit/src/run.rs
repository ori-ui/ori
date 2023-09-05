use std::{collections::HashMap, mem};

use futures_lite::future;
use ori_core::{
    event::{Modifiers, PointerButton, PointerId},
    math::Vec2,
    ui::Ui,
    window::{UiBuilder, Window, WindowDescriptor},
};
use winit::{
    event::{Event, KeyboardInput, MouseScrollDelta, TouchPhase, WindowEvent},
    event_loop::{ControlFlow, EventLoopWindowTarget},
    window::WindowBuilder,
};

use crate::{
    convert::{convert_key, convert_mouse_button, is_pressed},
    render::{WgpuRender, WgpuRenderInstance},
    window::WinitWindow,
    App, Error,
};

unsafe fn init<T>(
    ids: &mut HashMap<winit::window::WindowId, ori_core::window::WindowId>,
    window_desc: WindowDescriptor,
    target: &EventLoopWindowTarget<()>,
    builder: &mut UiBuilder<T>,
    ui: &mut Ui<T, WgpuRender>,
    instance: &mut Option<WgpuRenderInstance>,
) {
    /* create the window */
    let window = WindowBuilder::new()
        .with_visible(false)
        .with_transparent(window_desc.transparent)
        .build(target)
        .unwrap();

    // SAFETY: this function will never return and the window will therefore
    // be valid for the lifetime on the RenderInstance.

    let surface = unsafe {
        let (new_instance, surface) = future::block_on(WgpuRenderInstance::new(&window)).unwrap();
        *instance = Some(new_instance);

        surface
    };

    ids.insert(window.id(), window_desc.id);

    /* create the initial window */
    let raw_window = Box::new(WinitWindow::from(window));
    let window = Window::new(raw_window, window_desc);

    /* create the render */
    let render = WgpuRender::new(
        instance.as_ref().unwrap(),
        surface,
        window.width(),
        window.height(),
    )
    .unwrap();

    /* add the window to the ui */
    let builder = mem::replace(builder, Box::new(|_| unreachable!()));
    ui.add_window(builder, window, render);

    /* initialize the ui */
    ui.init();
}

unsafe fn recreate_surfaces<T>(ui: &mut Ui<T, WgpuRender>, instance: &WgpuRenderInstance) {
    for window_id in ui.window_ids() {
        let window = ui.window_mut(window_id);

        let width = window.window().width();
        let height = window.window().height();

        let surface = unsafe {
            let window = window.window().downcast_raw::<WinitWindow>().unwrap();
            instance.create_surface(&window.window).unwrap()
        };

        let render = WgpuRender::new(instance, surface, width, height).unwrap();
        window.set_render(render);
    }
}

pub(crate) fn run<T: 'static>(mut app: App<T>) -> Result<(), Error> {
    /* initialize tracing if enabled */
    #[cfg(feature = "tracing")]
    if let Err(err) = crate::tracing::init_tracing() {
        eprintln!("Failed to initialize tracing: {}", err);
    }

    /* create the window map
     *
     * this is used to map the winit window id to the ori window id */
    let mut ids = HashMap::new();
    let mut instance = None;

    app.event_loop.run(move |event, target, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            // we need to recreate the surfaces when the event loop is resumed
            //
            // this is necessary for android
            Event::Resumed => {
                if let Some(ref instance) = instance {
                    unsafe { recreate_surfaces(&mut app.ui, instance) };
                } else {
                    // if the instance is not initialized yet, we need to
                    // initialize the ui
                    unsafe {
                        init(
                            &mut ids,
                            app.window.clone(),
                            target,
                            &mut app.builder,
                            &mut app.ui,
                            &mut instance,
                        );
                    }
                }
            }
            Event::RedrawEventsCleared => {
                app.ui.idle();
            }
            Event::RedrawRequested(window_id) => {
                let id = ids[&window_id];
                app.ui.render(id);
            }
            Event::UserEvent(_) => {
                app.ui.handle_commands();
            }
            Event::WindowEvent { window_id, event } => {
                let window_id = ids[&window_id];

                match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::Resized(_) => {
                        app.ui.resized(window_id);
                    }
                    WindowEvent::ScaleFactorChanged { .. } => {
                        app.ui.scale_factor_changed(window_id);
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
                    WindowEvent::Touch(event) => {
                        let position = Vec2::new(event.location.x as f32, event.location.y as f32);
                        let pointer_id = PointerId::from_hash(&event.device_id);

                        app.ui.pointer_moved(window_id, pointer_id, position);

                        match event.phase {
                            TouchPhase::Started => {
                                app.ui.pointer_button(
                                    window_id,
                                    pointer_id,
                                    PointerButton::Primary,
                                    true,
                                );
                            }
                            TouchPhase::Moved => {}
                            TouchPhase::Ended | TouchPhase::Cancelled => {
                                app.ui.pointer_button(
                                    window_id,
                                    pointer_id,
                                    PointerButton::Primary,
                                    false,
                                );

                                app.ui.pointer_left(window_id, pointer_id);
                            }
                        }
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
