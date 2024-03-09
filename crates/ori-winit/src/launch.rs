use std::{collections::HashMap, mem};

use ori_core::{
    event::{Modifiers, PointerButton, PointerId},
    layout::{Point, Vector},
    shell::Windows,
    ui::{Ui, UiBuilder, UiRequest},
    window::{Window, WindowDescriptor},
};
use winit::{
    dpi::LogicalSize,
    event::{Event, KeyEvent, MouseScrollDelta, TouchPhase, WindowEvent},
    event_loop::{EventLoop, EventLoopWindowTarget},
    keyboard::{ModifiersState, PhysicalKey},
    window::WindowBuilder,
};

use crate::{
    clipboard::WinitClipboard,
    convert::{convert_key, convert_mouse_button, is_pressed},
    log::error_internal,
    window::WinitWindow,
    Error,
};

pub(crate) fn launch<T: 'static>(
    data: T,
    event_loop: EventLoop<()>,
    ui: Ui<T>,
    windows: Windows<T>,
) -> Result<(), Error> {
    /* initialize tracing if enabled */
    #[cfg(feature = "tracing")]
    if let Err(err) = crate::tracing::init_tracing() {
        eprintln!("Failed to initialize tracing: {}", err);
    }

    let mut state = AppState::new(data, ui, windows);
    state.ui.set_clipboard(WinitClipboard::new());

    event_loop.run(move |event, target| {
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
                state.idle();
            }

            // this event is sent by [`WinitWaker`] telling us that there are
            // commands that need to be processed
            Event::UserEvent(_) => {
                state.ui.handle_commands(&mut state.data);
            }
            Event::WindowEvent { window_id, event } => {
                state.window_event(window_id, event);
            }
            _ => {}
        }

        state.handle_requests(target);

        if state.ui.should_quit() && state.init {
            target.exit();
        }
    })?;

    Ok(())
}

struct AppState<T: 'static> {
    init: bool,
    windows: Windows<T>,
    data: T,
    ui: Ui<T>,
    window_ids: HashMap<winit::window::WindowId, ori_core::window::WindowId>,

    /* glow */
    #[cfg(feature = "glow")]
    renders: HashMap<ori_core::window::WindowId, (ori_glow::GlowRender, ori_glow::GlutinContext)>,

    /* wgpu */
    #[cfg(feature = "wgpu")]
    renders: HashMap<ori_core::window::WindowId, ori_wgpu::WgpuRender>,
    #[cfg(feature = "wgpu")]
    instance: Option<ori_wgpu::WgpuRenderInstance>,
}

impl<T> AppState<T> {
    fn new(data: T, ui: Ui<T>, windows: Windows<T>) -> Self {
        Self {
            init: false,
            windows,
            data,
            ui,
            window_ids: HashMap::new(),

            /* glow */
            #[cfg(feature = "glow")]
            renders: HashMap::new(),

            /* wgpu */
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
        self.ui.init(&mut self.data);

        for (desc, builder) in mem::take(&mut self.windows) {
            if let Err(err) = self.create_window(target, desc, builder) {
                error_internal!("Failed to create window: {}", err);
                return;
            }
        }
    }

    fn handle_requests(&mut self, target: &EventLoopWindowTarget<()>) {
        for request in self.ui.take_requests() {
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
            UiRequest::RedrawWindow(window) => self.redraw_window(window),
            UiRequest::CreateWindow(desc, builder) => self.create_window(target, desc, builder)?,
            UiRequest::RemoveWindow(window_id) => self.remove_window(window_id),
        }

        Ok(())
    }

    #[cfg(feature = "wgpu")]
    fn init_wgpu(
        &mut self,
        window: &winit::window::Window,
    ) -> Result<(ori_wgpu::WgpuRenderInstance, ori_wgpu::Surface), Error> {
        use ori_wgpu::WgpuContext;

        let (instance, surface) = unsafe { ori_wgpu::WgpuRenderInstance::new(window)? };

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
        desc: &WindowDescriptor,
    ) -> Result<(), Error> {
        use ori_wgpu::WgpuRender;

        let (instance, surface) = if let Some(ref instance) = self.instance {
            let surface = unsafe { instance.create_surface(window)? };
            (instance, surface)
        } else {
            let (instance, surface) = self.init_wgpu(window)?;
            (self.instance.insert(instance) as _, surface)
        };

        let samples = if desc.anti_aliasing { 4 } else { 1 };
        let size = window.inner_size();
        let render = WgpuRender::new(instance, surface, samples, size.width, size.height)?;

        self.renders.insert(desc.id, render);

        Ok(())
    }

    #[cfg(feature = "glow")]
    fn create_glow_render(
        &mut self,
        window: &winit::window::Window,
        desc: &WindowDescriptor,
    ) -> Result<(), Error> {
        use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

        let size = window.inner_size();
        let samples = if desc.anti_aliasing { 4 } else { 1 };
        let context = ori_glow::GlutinContext::new(
            window.raw_window_handle(),
            window.raw_display_handle(),
            size.width,
            size.height,
            samples,
        )?;

        context.make_current()?;

        let proc_addr = |s: &str| context.get_proc_address(s);
        let render = ori_glow::GlowRender::new(proc_addr, size.width, size.height)?;

        self.renders.insert(desc.id, (render, context));

        Ok(())
    }

    fn idle(&mut self) {
        self.ui.idle(&mut self.data);

        #[cfg(feature = "glow")]
        for (render, _) in self.renders.values_mut() {
            render.clean();
        }

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
            .with_inner_size(LogicalSize::new(desc.width, desc.height))
            .with_resizable(desc.resizable)
            .with_decorations(desc.decorated)
            .with_transparent(desc.transparent)
            .with_visible(false)
            .build(target)?;

        self.window_ids.insert(window.id(), desc.id);

        #[cfg(feature = "glow")]
        self.create_glow_render(&window, &desc)?;

        #[cfg(feature = "wgpu")]
        self.create_wgpu_render(&window, &desc)?;

        /* create the initial window */
        let raw_window = Box::new(WinitWindow::from(window));
        let mut window = Window::new(raw_window, desc.id);

        window.set_icon(desc.icon.as_ref());
        window.set_visible(desc.visible);
        window.set_maximized(desc.maximized);
        window.set_color(desc.color);

        /* add the window to the ui */
        self.ui.add_window(&mut self.data, builder, window);

        Ok(())
    }

    fn remove_window(&mut self, window_id: ori_core::window::WindowId) {
        self.window_ids.retain(|_, &mut id| id != window_id);

        self.ui.remove_window(window_id);

        #[cfg(feature = "wgpu")]
        self.renders.remove(&window_id);
    }

    fn redraw_window(&mut self, window_id: ori_core::window::WindowId) {
        self.ui.window_mut(window_id).window_mut().request_draw();
    }

    fn render(&mut self, window_id: ori_core::window::WindowId) -> Result<(), Error> {
        // sort the scene
        self.ui.render(&mut self.data, window_id);
        self.ui.window_mut(window_id).scene_mut().sort();

        let window = self.ui.window(window_id);

        let clear_color = window.color();

        let logical = window.window().size();
        let physical = window.window().physical_size();
        let scene = window.scene();

        /* glow */
        #[cfg(feature = "glow")]
        if let Some((render, context)) = self.renders.get_mut(&window_id) {
            context.make_current()?;
            render.render_scene(scene, clear_color, logical, physical)?;
            context.swap_buffers()?;
        }

        /* wgpu */
        #[cfg(feature = "wgpu")]
        if let Some(render) = self.renders.get_mut(&window_id) {
            let cx = self.ui.contexts.get::<ori_wgpu::WgpuContext>().unwrap();
            render.render_scene(cx, scene, clear_color, logical, physical);
        }

        Ok(())
    }

    fn window_event(&mut self, winit_id: winit::window::WindowId, event: WindowEvent) {
        // if the window id is not in the map, we ignore the event
        let Some(&id) = self.window_ids.get(&winit_id) else {
            return;
        };

        match event {
            WindowEvent::RedrawRequested => {
                if let Err(err) = self.render(id) {
                    error_internal!("Failed to render: {}", err);
                }
            }
            WindowEvent::CloseRequested => {
                self.ui.close_requested(&mut self.data, id);
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
                let scale_factor = self.ui.window(id).window().scale_factor();
                let position = Point::new(position.x as f32, position.y as f32) / scale_factor;
                self.ui.pointer_moved(
                    &mut self.data,
                    id,
                    PointerId::from_hash(&device_id),
                    position,
                );
            }
            WindowEvent::CursorLeft { device_id } => {
                (self.ui).pointer_left(&mut self.data, id, PointerId::from_hash(&device_id));
            }
            WindowEvent::MouseInput {
                device_id,
                state,
                button,
                ..
            } => {
                self.ui.pointer_button(
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
            } => self.ui.pointer_scroll(
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

                self.ui.keyboard_key(
                    &mut self.data,
                    id,
                    code,
                    text.map(Into::into),
                    is_pressed(state),
                );
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
    }

    fn touch_event(&mut self, window_id: ori_core::window::WindowId, event: winit::event::Touch) {
        let scale_factor = self.ui.window(window_id).window().scale_factor();
        let position = Point::new(event.location.x as f32, event.location.y as f32) / scale_factor;
        let pointer_id = PointerId::from_hash(&event.device_id);

        // we always send a pointer moved event first because the ui
        // needs to know where the pointer is. this will also ensure
        // that hot state is updated correctly
        (self.ui).pointer_moved(&mut self.data, window_id, pointer_id, position);

        match event.phase {
            TouchPhase::Started => {
                self.ui.pointer_button(
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
                self.ui.pointer_button(
                    &mut self.data,
                    window_id,
                    pointer_id,
                    // a touch event is always the primary button
                    PointerButton::Primary,
                    false,
                );

                // we also need to send a pointer left event because
                // the ui needs to know that the pointer left the window
                self.ui.pointer_left(&mut self.data, window_id, pointer_id);
            }
        }
    }
}
