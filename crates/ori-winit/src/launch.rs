use std::{collections::HashMap, mem};

use ori_core::{
    event::{Modifiers, PointerButton, PointerId},
    layout::{Point, Vector},
    ui::{Ui, UiBuilder, UiRequest, UiRequests},
    window::{Window, WindowDescriptor},
};
use winit::{
    dpi::PhysicalSize,
    event::{Event, KeyEvent, MouseScrollDelta, TouchPhase, WindowEvent},
    event_loop::EventLoopWindowTarget,
    keyboard::{ModifiersState, PhysicalKey},
    window::WindowBuilder,
};

use crate::{
    convert::{convert_key, convert_mouse_button, is_pressed},
    log::error_internal,
    window::WinitWindow,
    Error, Launcher,
};

pub(crate) fn launch<T: 'static>(app: Launcher<T>) -> Result<(), Error> {
    /* initialize tracing if enabled */
    #[cfg(feature = "tracing")]
    if let Err(err) = crate::tracing::init_tracing() {
        eprintln!("Failed to initialize tracing: {}", err);
    }

    let mut state = AppState::new(app.windows, app.ui, app.msaa);

    app.event_loop.run(move |event, target| {
        match event {
            // we need to recreate the surfaces when the event loop is resumed
            //
            // this is necessary for android
            Event::Resumed => {
                state.resume(target);
            }
            Event::AboutToWait => {
                // after all events for a frame have been processed, we need to
                // run the idle function
                state.idle(target);
            }

            // this event is sent by [`WinitWaker`] telling us that there are
            // commands that need to be processed
            Event::UserEvent(_) => {
                let requests = state.ui.handle_commands();
                state.handle_requests(target, requests);
            }
            Event::WindowEvent { window_id, event } => {
                let requests = state.window_event(target, window_id, event);
                state.handle_requests(target, requests);
            }
            _ => {}
        }

        if state.ui.should_exit() && state.init {
            target.exit();
        }
    })?;

    Ok(())
}

struct AppState<T: 'static> {
    init: bool,
    windows: Vec<(WindowDescriptor, UiBuilder<T>)>,
    ui: Ui<T>,
    msaa: bool,
    window_ids: HashMap<winit::window::WindowId, ori_core::window::WindowId>,
    #[cfg(feature = "wgpu")]
    renders: HashMap<ori_core::window::WindowId, crate::wgpu::WgpuRender>,
    #[cfg(feature = "wgpu")]
    instance: Option<crate::wgpu::WgpuRenderInstance>,
}

impl<T> AppState<T> {
    fn new(windows: Vec<(WindowDescriptor, UiBuilder<T>)>, ui: Ui<T>, msaa: bool) -> Self {
        Self {
            init: false,
            windows,
            ui,
            msaa,
            window_ids: HashMap::new(),
            #[cfg(feature = "wgpu")]
            renders: HashMap::new(),
            #[cfg(feature = "wgpu")]
            instance: None,
        }
    }

    fn resume(&mut self, target: &EventLoopWindowTarget<()>) {
        if self.init {
            return;
        }

        self.init = true;

        for (desc, builder) in mem::take(&mut self.windows) {
            if let Err(err) = self.create_window(target, desc, builder) {
                error_internal!("Failed to create window: {}", err);
                return;
            }
        }

        let requests = self.ui.init();
        self.handle_requests(target, requests);
    }

    fn handle_requests(&mut self, target: &EventLoopWindowTarget<()>, requests: UiRequests<T>) {
        for request in requests {
            if let Err(err) = self.handle_request(target, request) {
                error_internal!("Failed to handle request: {}", err);
            }
        }
    }

    fn handle_request(
        &mut self,
        target: &EventLoopWindowTarget<()>,
        request: UiRequest<T>,
    ) -> Result<(), Error> {
        match request {
            UiRequest::Render(window) => self.render(window),
            UiRequest::CreateWindow(desc, builder) => self.create_window(target, desc, builder)?,
            UiRequest::RemoveWindow(window_id) => self.remove_window(window_id),
        }

        Ok(())
    }

    #[cfg(feature = "wgpu")]
    fn init_wgpu(
        &mut self,
        window: &winit::window::Window,
    ) -> Result<(crate::wgpu::WgpuRenderInstance, wgpu::Surface), Error> {
        use crate::wgpu::WgpuContext;

        let instance = unsafe { crate::wgpu::WgpuRenderInstance::new(window) };
        let (instance, surface) = pollster::block_on(instance)?;

        let context = WgpuContext {
            device: instance.device.clone(),
            queue: instance.queue.clone(),
            textures: Default::default(),
        };
        self.ui.contexts.insert(context);

        Ok((instance, surface))
    }

    #[cfg(feature = "wgpu")]
    fn create_wgpu_render(
        &mut self,
        window: &winit::window::Window,
        id: ori_core::window::WindowId,
    ) -> Result<(), Error> {
        use crate::wgpu::WgpuRender;

        let (instance, surface) = if let Some(ref instance) = self.instance {
            let surface = unsafe { instance.create_surface(window)? };
            (instance, surface)
        } else {
            let (instance, surface) = self.init_wgpu(window)?;
            (self.instance.insert(instance) as _, surface)
        };

        let samples = if self.msaa { 4 } else { 1 };
        let size = window.inner_size();
        let render = WgpuRender::new(instance, surface, samples, size.width, size.height)?;

        self.renders.insert(id, render);

        Ok(())
    }

    fn idle(&mut self, target: &EventLoopWindowTarget<()>) {
        let requests = self.ui.idle();
        self.handle_requests(target, requests);

        #[cfg(feature = "wgpu")]
        for render in self.renders.values_mut() {
            render.clean();
        }
    }

    fn create_window(
        &mut self,
        target: &EventLoopWindowTarget<()>,
        desc: WindowDescriptor,
        builder: UiBuilder<T>,
    ) -> Result<(), Error> {
        /* create the window */
        let window = WindowBuilder::new()
            .with_title(&desc.title)
            .with_inner_size(PhysicalSize::new(desc.width, desc.height))
            .with_resizable(desc.resizable)
            .with_decorations(desc.decorated)
            .with_transparent(desc.transparent)
            .with_maximized(desc.maximized)
            .with_visible(false)
            .build(target)?;

        self.window_ids.insert(window.id(), desc.id);

        #[cfg(feature = "wgpu")]
        self.create_wgpu_render(&window, desc.id)?;

        /* create the initial window */
        let raw_window = Box::new(WinitWindow::from(window));
        let mut window = Window::new(raw_window, desc.id);

        window.set_icon(desc.icon.as_ref());
        window.set_visible(desc.visible);
        window.set_color(desc.color);

        /* add the window to the ui */
        let requests = self.ui.add_window(builder, window);
        self.handle_requests(target, requests);

        Ok(())
    }

    fn remove_window(&mut self, window_id: ori_core::window::WindowId) {
        self.window_ids.retain(|_, &mut id| id != window_id);

        self.ui.remove_window(window_id);

        #[cfg(feature = "wgpu")]
        self.renders.remove(&window_id);
    }

    fn redraw(&mut self, target: &EventLoopWindowTarget<()>, window_id: winit::window::WindowId) {
        // if the window id is not in the map, we ignore the event
        if let Some(&window_id) = self.window_ids.get(&window_id) {
            // render the window
            let requests = self.ui.render(window_id);
            self.handle_requests(target, requests);
        }
    }

    fn render(&mut self, window: ori_core::window::WindowId) {
        #[cfg(feature = "wgpu")]
        {
            if let Some(render) = self.renders.get_mut(&window) {
                // sort the scene
                self.ui.window_mut(window).scene_mut().sort();

                let window = self.ui.window(window);

                let clear_color = window.color();

                let width = window.window().width();
                let height = window.window().height();
                let scene = window.scene();

                let context = self.ui.contexts.get::<crate::wgpu::WgpuContext>().unwrap();
                render.render_scene(context, scene, clear_color, width, height);
            }
        }
    }

    fn window_event(
        &mut self,
        target: &EventLoopWindowTarget<()>,
        winit_id: winit::window::WindowId,
        event: WindowEvent,
    ) -> UiRequests<T> {
        // if the window id is not in the map, we ignore the event
        let Some(&id) = self.window_ids.get(&winit_id) else {
            return UiRequests::new();
        };

        match event {
            WindowEvent::RedrawRequested => {
                self.redraw(target, winit_id);
            }
            WindowEvent::CloseRequested => {
                let requests = self.ui.close_requested(id);
                self.handle_requests(target, requests);
            }
            WindowEvent::Resized(_) => {
                self.ui.resized(id);
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                self.ui.scale_factor_changed(id);
            }
            WindowEvent::CursorMoved {
                device_id,
                position,
                ..
            } => {
                return self.ui.pointer_moved(
                    id,
                    PointerId::from_hash(&device_id),
                    Point::new(position.x as f32, position.y as f32),
                );
            }
            WindowEvent::CursorLeft { device_id } => {
                return self.ui.pointer_left(id, PointerId::from_hash(&device_id));
            }
            WindowEvent::MouseInput {
                device_id,
                state,
                button,
                ..
            } => {
                return self.ui.pointer_button(
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
                return self.ui.pointer_scroll(
                    id,
                    PointerId::from_hash(&device_id),
                    Vector::new(x, y),
                );
            }
            // since we're using a pointer model we need to handle touch
            // by emulating pointer events
            WindowEvent::Touch(event) => return self.touch_event(id, event),
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
                let mut requests = UiRequests::new();

                if let PhysicalKey::Code(code) = physical_key {
                    if let Some(key) = convert_key(code) {
                        requests.extend(self.ui.keyboard_key(id, key, is_pressed(state)));
                    }
                }

                if let Some(text) = text {
                    if is_pressed(state) {
                        requests.extend(self.ui.keyboard_text(id, text.into()));
                    }
                }

                return requests;
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.ui.modifiers_changed(Modifiers {
                    shift: modifiers.state().contains(ModifiersState::SHIFT),
                    ctrl: modifiers.state().contains(ModifiersState::CONTROL),
                    alt: modifiers.state().contains(ModifiersState::ALT),
                    meta: modifiers.state().contains(ModifiersState::SUPER),
                });
            }
            _ => {}
        }

        UiRequests::new()
    }

    fn touch_event(
        &mut self,
        window_id: ori_core::window::WindowId,
        event: winit::event::Touch,
    ) -> UiRequests<T> {
        let position = Point::new(event.location.x as f32, event.location.y as f32);
        let pointer_id = PointerId::from_hash(&event.device_id);

        // we always send a pointer moved event first because the ui
        // needs to know where the pointer is. this will also ensure
        // that hot state is updated correctly
        let mut requests = self.ui.pointer_moved(window_id, pointer_id, position);

        match event.phase {
            TouchPhase::Started => {
                let new_requests = self.ui.pointer_button(
                    window_id,
                    pointer_id,
                    // a touch event is always the primary button
                    PointerButton::Primary,
                    true,
                );

                requests.extend(new_requests);
            }
            TouchPhase::Moved => {}
            TouchPhase::Ended | TouchPhase::Cancelled => {
                let new_requests = self.ui.pointer_button(
                    window_id,
                    pointer_id,
                    // a touch event is always the primary button
                    PointerButton::Primary,
                    false,
                );

                requests.extend(new_requests);

                // we also need to send a pointer left event because
                // the ui needs to know that the pointer left the window
                let new_requests = self.ui.pointer_left(window_id, pointer_id);

                requests.extend(new_requests);
            }
        }

        requests
    }
}
