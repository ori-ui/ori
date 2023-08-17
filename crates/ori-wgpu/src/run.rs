use std::collections::HashMap;

use ori_core::{math::Vec2, Modifiers, PointerId, Ui, Window, WindowDescriptor};
use winit::{
    dpi::PhysicalSize,
    event::{Event, KeyboardInput, MouseScrollDelta, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    window::WindowBuilder,
};

use crate::{
    convert::{convert_key, convert_mouse_button, is_pressed},
    tracing::init_tracing,
    Error, Render, RenderInstance, WinitWindow,
};

use crate::App;

fn build_window(
    window_target: &EventLoopWindowTarget<()>,
    desc: &WindowDescriptor,
) -> Result<winit::window::Window, Error> {
    WindowBuilder::new()
        .with_title(&desc.title)
        .with_inner_size(PhysicalSize::new(
            desc.size.width as u32,
            desc.size.height as u32,
        ))
        .with_resizable(desc.resizable)
        .with_decorations(desc.decorated)
        .with_transparent(desc.transparent)
        .with_maximized(desc.maximized)
        .with_visible(desc.visible)
        .build(window_target)
        .map_err(Into::into)
}

pub(crate) fn run<T: 'static>(app: App<T>) -> Result<(), Error> {
    if let Err(err) = init_tracing() {
        eprintln!("Failed to initialize tracing: {}", err);
    }

    let event_loop = EventLoop::new();
    let window = build_window(&event_loop, &app.window)?;

    let runtime = tokio::runtime::Runtime::new().unwrap();

    // SAFETY: this function will never return and the window will therefore
    // be valid for the lifetime on the RenderInstance.
    let (instance, surface) = runtime.block_on(unsafe { RenderInstance::new(&window) })?;

    let _guard = runtime.enter();

    let mut ids = HashMap::new();
    let mut ui = Ui::<T, Render>::new(app.data);

    ids.insert(window.id(), app.window.id);

    let raw_window = Box::new(WinitWindow::from(window));
    let window = Window::new(app.window.id, raw_window);
    let render = Render::new(&instance, surface, window.width(), window.height())?;

    ui.add_window(app.builder, window, render);

    event_loop.run(move |event, _, control_flow| match event {
        Event::RedrawRequested(window_id) => {
            let id = ids[&window_id];
            ui.render(id);
        }
        Event::WindowEvent { window_id, event } => {
            let id = ids[&window_id];

            match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => {
                    ui.resized(id);
                }
                WindowEvent::CursorMoved {
                    device_id,
                    position,
                    ..
                } => {
                    ui.pointer_moved(
                        id,
                        PointerId::from_hash(&device_id),
                        Vec2::new(position.x as f32, position.y as f32),
                    );
                }
                WindowEvent::CursorLeft { device_id } => {
                    ui.pointer_left(id, PointerId::from_hash(&device_id));
                }
                WindowEvent::MouseInput {
                    device_id,
                    state,
                    button,
                    ..
                } => {
                    ui.pointer_button(
                        id,
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
                    ui.pointer_scroll(id, PointerId::from_hash(&device_id), Vec2::new(x, y));
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
                        ui.keyboard_key(id, key, is_pressed(state));
                    }
                }
                WindowEvent::ReceivedCharacter(c) => {
                    ui.keyboard_char(id, c);
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
    });
}
