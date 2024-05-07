use std::{collections::HashMap, mem, num::NonZeroU32, rc::Rc, slice};

use ori_app::{App, AppBuilder, AppRequest, UiBuilder};
use ori_core::{
    canvas::Color,
    command::CommandWaker,
    event::{Modifiers, PointerButton, PointerId},
    layout::{Point, Size, Vector},
    window::{Window, WindowUpdate},
};
use ori_tiny_skia::TinySkiaRenderer;
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{KeyEvent, MouseScrollDelta, TouchPhase, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{ModifiersState, PhysicalKey},
};

use crate::{
    clipboard::WinitClipboard,
    convert::{convert_cursor_icon, convert_key, convert_mouse_button, is_pressed},
    Error,
};

/// Launch an application.
pub fn launch<T>(app: AppBuilder<T>, data: T) -> Result<(), Error> {
    /* initialize tracing if enabled */
    if let Err(err) = crate::tracing::init_tracing() {
        eprintln!("Failed to initialize tracing: {}", err);
    }

    let event_loop = EventLoop::new()?;

    let waker = CommandWaker::new({
        let proxy = event_loop.create_proxy();

        move || {
            let _ = proxy.send_event(());
        }
    });

    let mut app = app.build(waker);
    app.set_clipboard(WinitClipboard::new());

    let mut winit_app = WinitApp::new(data, app);
    Ok(event_loop.run_app(&mut winit_app)?)
}

type RcWindow = Rc<winit::window::Window>;

struct WindowState {
    window: RcWindow,
    #[allow(unused)]
    context: softbuffer::Context<RcWindow>,
    surface: softbuffer::Surface<RcWindow, RcWindow>,
    renderer: TinySkiaRenderer,
}

struct WinitApp<T> {
    init: bool,
    app: App<T>,
    data: T,
    window_ids: HashMap<winit::window::WindowId, ori_core::window::WindowId>,
    windows: HashMap<ori_core::window::WindowId, WindowState>,
}

impl<T> ApplicationHandler<()> for WinitApp<T> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.init {
            return;
        }

        self.init = true;
        self.app.init(&mut self.data);

        self.handle_requests(event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        winit_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        // if the window id is not in the map, we ignore the event
        let Some(&id) = self.window_ids.get(&winit_id) else {
            return;
        };

        match event {
            WindowEvent::RedrawRequested => {
                if let Err(err) = self.render(id) {
                    tracing::error!("Failed to render: {}", err);
                }
            }
            WindowEvent::CloseRequested => {
                self.app.close_requested(&mut self.data, id);
            }
            WindowEvent::Resized(inner_size) => {
                (self.app).window_resized(&mut self.data, id, inner_size.width, inner_size.height);

                if let Some(state) = self.windows.get_mut(&id) {
                    state
                        .surface
                        .resize(
                            NonZeroU32::new(inner_size.width).unwrap(),
                            NonZeroU32::new(inner_size.height).unwrap(),
                        )
                        .unwrap();

                    state.renderer.resize(inner_size.width, inner_size.height);
                }
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                (self.app).window_scaled(&mut self.data, id, scale_factor as f32);
            }
            WindowEvent::CursorMoved {
                device_id,
                position,
                ..
            } => {
                let scale_factor = self.scale_factor(id);
                let position = Point::new(position.x as f32, position.y as f32) / scale_factor;
                self.app.pointer_moved(
                    &mut self.data,
                    id,
                    PointerId::from_hash(&device_id),
                    position,
                );
            }
            WindowEvent::CursorLeft { device_id } => {
                (self.app).pointer_left(&mut self.data, id, PointerId::from_hash(&device_id));
            }
            WindowEvent::MouseInput {
                device_id,
                state,
                button,
                ..
            } => {
                self.app.pointer_button(
                    &mut self.data,
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
            } => self.app.pointer_scrolled(
                &mut self.data,
                id,
                PointerId::from_hash(&device_id),
                Vector::new(x, y),
            ),
            // since we're using a pointer model we need to handle touch
            // by emulating pointer events
            WindowEvent::Touch(event) => self.touch_event(id, event),
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key,
                        text,
                        state,
                        ..
                    },
                ..
            } => {
                let code = match physical_key {
                    PhysicalKey::Code(code) => convert_key(code),
                    _ => None,
                };

                self.app.keyboard_key(
                    &mut self.data,
                    id,
                    code,
                    text.map(Into::into),
                    is_pressed(state),
                );
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.app.modifiers_changed(Modifiers {
                    shift: modifiers.state().contains(ModifiersState::SHIFT),
                    ctrl: modifiers.state().contains(ModifiersState::CONTROL),
                    alt: modifiers.state().contains(ModifiersState::ALT),
                    meta: modifiers.state().contains(ModifiersState::SUPER),
                });
            }
            _ => {}
        }

        self.handle_requests(event_loop);
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, _event: ()) {
        self.app.handle_commands(&mut self.data);

        self.handle_requests(event_loop);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.app.idle(&mut self.data);

        self.handle_requests(event_loop);
    }
}

impl<T> WinitApp<T> {
    fn new(data: T, app: App<T>) -> Self {
        Self {
            init: false,
            app,
            data,
            window_ids: HashMap::new(),
            windows: HashMap::new(),
        }
    }

    fn handle_requests(&mut self, target: &ActiveEventLoop) {
        for request in self.app.take_requests() {
            if let Err(err) = self.handle_request(target, request) {
                tracing::error!("Failed to handle request: {}", err);
            }
        }
    }

    fn handle_request(
        &mut self,
        target: &ActiveEventLoop,
        request: AppRequest<T>,
    ) -> Result<(), Error> {
        match request {
            AppRequest::OpenWindow(desc, builder) => {
                self.create_window(target, desc, builder)?;
            }
            AppRequest::CloseWindow(id) => {
                self.windows.remove(&id);
            }
            AppRequest::DragWindow(id) => {
                if let Some(state) = self.windows.get_mut(&id) {
                    if let Err(err) = state.window.drag_window() {
                        tracing::warn!("Failed to drag window: {}", err);
                    }
                }
            }
            AppRequest::RequestRedraw(id) => {
                if let Some(state) = self.windows.get_mut(&id) {
                    state.window.request_redraw();
                }
            }
            AppRequest::UpdateWindow(id, update) => {
                if let Some(state) = self.windows.get_mut(&id) {
                    match update {
                        WindowUpdate::Title(title) => state.window.set_title(&title),
                        WindowUpdate::Icon(icon) => match icon {
                            Some(icon) => {
                                let icon = winit::window::Icon::from_rgba(
                                    icon.data().to_vec(),
                                    icon.width(),
                                    icon.height(),
                                )
                                .expect("Failed to create icon");

                                state.window.set_window_icon(Some(icon));
                            }
                            None => {
                                state.window.set_window_icon(None);
                            }
                        },
                        WindowUpdate::Size(size) => {
                            let size = size.max(Size::all(10.0));
                            let inner = LogicalSize::new(size.width, size.height);

                            state.window.set_min_inner_size(Some(inner));
                            state.window.set_max_inner_size(Some(inner));
                        }
                        WindowUpdate::Scale(_) => {}
                        WindowUpdate::Resizable(resizable) => {
                            state.window.set_resizable(resizable);
                        }
                        WindowUpdate::Decorated(decorated) => {
                            state.window.set_decorations(decorated);
                        }
                        WindowUpdate::Maximized(maximized) => {
                            state.window.set_maximized(maximized);
                        }
                        WindowUpdate::Visible(visible) => {
                            state.window.set_visible(visible);
                        }
                        WindowUpdate::Color(color) => {
                            let transparent = color.map_or(false, Color::is_translucent);
                            state.window.set_transparent(transparent);
                        }
                        WindowUpdate::Cursor(cursor) => {
                            state.window.set_cursor(convert_cursor_icon(cursor));
                        }
                    }
                }
            }
            AppRequest::Quit => target.exit(),
        }

        Ok(())
    }

    fn create_window(
        &mut self,
        target: &ActiveEventLoop,
        ori: Window,
        builder: UiBuilder<T>,
    ) -> Result<(), Error> {
        let attributes = winit::window::WindowAttributes::default()
            .with_title(&ori.title)
            .with_inner_size(LogicalSize::new(ori.width(), ori.height()))
            .with_resizable(ori.resizable)
            .with_decorations(ori.decorated)
            .with_transparent(ori.color.map_or(false, Color::is_translucent))
            .with_visible(false);

        /* create the window */
        let window = Rc::new(target.create_window(attributes)?);
        self.window_ids.insert(window.id(), ori.id());

        let icon = match ori.icon {
            Some(ref icon) => {
                let icon = winit::window::Icon::from_rgba(
                    icon.data().to_vec(),
                    icon.width(),
                    icon.height(),
                )
                .expect("Failed to create icon");

                Some(icon)
            }
            None => None,
        };

        window.set_window_icon(icon);
        window.set_visible(ori.visible);
        window.set_maximized(ori.maximized);

        let context = softbuffer::Context::new(window.clone()).unwrap();
        let mut surface = softbuffer::Surface::new(&context, window.clone()).unwrap();

        surface.resize(
            NonZeroU32::new(ori.width()).unwrap(),
            NonZeroU32::new(ori.height()).unwrap(),
        )?;

        self.windows.insert(
            ori.id(),
            WindowState {
                window,
                context,
                surface,
                renderer: TinySkiaRenderer::new(ori.width(), ori.height()),
            },
        );

        /* add the window to the ui */
        self.app.add_window(&mut self.data, builder, ori);

        Ok(())
    }

    fn render(&mut self, window_id: ori_core::window::WindowId) -> Result<(), Error> {
        if let Some(state) = self.windows.get_mut(&window_id) {
            let mut buffer = state.surface.buffer_mut()?;

            if let Some(draw) = self.app.draw_window(&mut self.data, window_id) {
                let data = unsafe {
                    slice::from_raw_parts_mut(
                        buffer.as_mut_ptr().cast::<u8>(),
                        buffer.len() * mem::size_of::<u32>(),
                    )
                };

                let mut buffer = ori_tiny_skia::Buffer::Argb8(data);
                (state.renderer).render(&mut buffer, draw.canvas, draw.clear_color);
            }

            buffer.present()?;
        }

        Ok(())
    }

    fn scale_factor(&self, window_id: ori_core::window::WindowId) -> f32 {
        match self.windows.get(&window_id) {
            Some(state) => state.window.scale_factor() as f32,
            None => 1.0,
        }
    }

    fn touch_event(&mut self, window_id: ori_core::window::WindowId, event: winit::event::Touch) {
        let scale_factor = self.scale_factor(window_id);
        let position = Point::new(event.location.x as f32, event.location.y as f32) / scale_factor;
        let pointer_id = PointerId::from_hash(&event.device_id);

        // we always send a pointer moved event first because the ui
        // needs to know where the pointer is. this will also ensure
        // that hot state is updated correctly
        (self.app).pointer_moved(&mut self.data, window_id, pointer_id, position);

        match event.phase {
            TouchPhase::Started => {
                self.app.pointer_button(
                    &mut self.data,
                    window_id,
                    pointer_id,
                    // a touch event is always the primary button
                    PointerButton::Primary,
                    true,
                );
            }
            TouchPhase::Moved => {}
            TouchPhase::Ended | TouchPhase::Cancelled => {
                self.app.pointer_button(
                    &mut self.data,
                    window_id,
                    pointer_id,
                    // a touch event is always the primary button
                    PointerButton::Primary,
                    false,
                );

                // we also need to send a pointer left event because
                // the ui needs to know that the pointer left the window
                self.app.pointer_left(&mut self.data, window_id, pointer_id);
            }
        }
    }
}
