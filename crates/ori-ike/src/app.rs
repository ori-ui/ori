use std::{any::Any, ffi, mem, num::NonZeroU32, time::Instant};

use glutin::{
    config::{Config as GlConfig, ConfigTemplateBuilder, GlConfig as _},
    context::{ContextAttributes, PossiblyCurrentContext as GlContext},
    display::GetGlDisplay,
    prelude::{GlDisplay, NotCurrentGlContext, PossiblyCurrentGlContext},
    surface::{GlSurface, Surface, SurfaceAttributesBuilder, WindowSurface},
};
use glutin_winit::DisplayBuilder;
use ori::AsyncContext;
use skia_safe::gpu::gl::FramebufferInfo;
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    raw_window_handle::HasWindowHandle,
    window::{Window, WindowId},
};

use crate::{
    Context, Effect,
    context::Event,
    key::convert_winit_key,
    skia::{SkiaCanvas, SkiaFonts},
};

pub struct App {}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run<T, V>(self, data: &mut T, mut ui: impl FnMut(&mut T) -> V + 'static)
    where
        V: Effect<T> + 'static,
        V::State: 'static,
    {
        let event_loop = EventLoop::with_user_event().build().unwrap();

        let rt;
        let _rt_guard = if tokio::runtime::Handle::try_current().is_err() {
            rt = Some(tokio::runtime::Runtime::new().unwrap());
            Some(rt.as_ref().unwrap().enter())
        } else {
            None
        };

        let runtime = tokio::runtime::Handle::current();

        let mut build: UiBuilder<T> = Box::new(move |data| Box::new(ui(data)));
        let view = build(data);

        let fonts = SkiaFonts::new();
        let context = Context {
            app:      ike::App::new(),
            proxy:    event_loop.create_proxy(),
            contexts: Vec::new(),
        };

        let mut state = AppState {
            data,

            build,
            view,
            state: None,

            runtime,

            fonts,
            context,
            windows: Vec::new(),
            gl_state: None,
        };

        event_loop.run_app(&mut state).unwrap();
    }
}

struct AppState<'a, T> {
    data: &'a mut T,

    build: UiBuilder<T>,
    view:  AnyEffect<T>,
    state: Option<Box<dyn Any>>,

    runtime: tokio::runtime::Handle,

    fonts:    SkiaFonts,
    context:  Context,
    windows:  Vec<WindowState>,
    gl_state: Option<GlState>,
}

struct WindowState {
    fb_info:      FramebufferInfo,
    skia_surface: skia_safe::Surface,
    skia_context: skia_safe::gpu::DirectContext,
    gl_surface:   Surface<WindowSurface>,

    animate: Option<Instant>,

    id:     ike::WindowId,
    window: Window,
}

struct GlState {
    config:  GlConfig,
    context: GlContext,
}

impl<T> ApplicationHandler<Event> for AppState<'_, T> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }

        self.create_gl_context(event_loop);

        let (_, state) = ori::View::build(
            &mut self.view,
            &mut self.context,
            self.data,
        );

        self.state = Some(state);
        self.update_windows(event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = self.windows.iter_mut().find(|w| w.window.id() == window_id) else {
            return;
        };

        match event {
            WindowEvent::RedrawRequested => {
                if let Some(animate) = window.animate.take() {
                    let delta_time = animate.elapsed();
                    self.context.app.animate(window.id, delta_time);
                }

                if self.context.app.window_needs_animate(window.id) {
                    window.window.request_redraw();
                    window.animate = Some(Instant::now());
                }

                let gl = self.gl_state.as_ref().unwrap();
                gl.context.make_current(&window.gl_surface).unwrap();

                let ike = self.context.app.get_window(window.id).unwrap();

                let scale = window.window.scale_factor();
                let canvas = window.skia_surface.canvas();
                canvas.clear(skia_safe::Color::from_argb(
                    f32::round(ike.color.a * 255.0) as u8,
                    f32::round(ike.color.r * 255.0) as u8,
                    f32::round(ike.color.g * 255.0) as u8,
                    f32::round(ike.color.b * 255.0) as u8,
                ));
                canvas.save();
                canvas.scale((scale as f32, scale as f32));

                let mut skia_canvas = SkiaCanvas {
                    fonts: &mut self.fonts,
                    canvas,
                };

                self.context.app.draw(window.id, &mut skia_canvas);

                canvas.restore();

                window.skia_context.flush_and_submit();
                window.gl_surface.swap_buffers(&gl.context).unwrap();
            }

            WindowEvent::Resized(..) | WindowEvent::ScaleFactorChanged { .. } => {
                let gl = self.gl_state.as_ref().unwrap();
                window.resize(gl);

                let scale = window.window.scale_factor();
                let size = window.window.inner_size().to_logical(scale);

                let size = ike::Size::new(size.width, size.height);

                match event {
                    WindowEvent::Resized(..) => {
                        self.context.app.window_resized(window.id, size);
                    }

                    WindowEvent::ScaleFactorChanged { .. } => {
                        (self.context.app).window_scaled(window.id, scale as f32, size);
                    }

                    _ => unreachable!(),
                }
            }

            WindowEvent::CursorEntered { device_id } => {
                let id = ike::PointerId::from_hash(device_id);
                self.context.app.pointer_entered(window.id, id);
            }

            WindowEvent::CursorLeft { device_id } => {
                let id = ike::PointerId::from_hash(device_id);
                self.context.app.pointer_left(window.id, id);
            }

            WindowEvent::CursorMoved {
                device_id,
                position,
            } => {
                let position = position.to_logical(window.window.scale_factor());
                let position = ike::Point::new(position.x, position.y);

                self.context.app.pointer_moved(
                    window.id,
                    ike::PointerId::from_hash(device_id),
                    position,
                );
            }

            WindowEvent::MouseInput {
                device_id,
                state,
                button,
            } => {
                let button = match button {
                    MouseButton::Left => ike::PointerButton::Primary,
                    MouseButton::Right => ike::PointerButton::Secondary,
                    MouseButton::Middle => ike::PointerButton::Tertiary,
                    MouseButton::Back => ike::PointerButton::Backward,
                    MouseButton::Forward => ike::PointerButton::Forward,
                    MouseButton::Other(i) => ike::PointerButton::Other(i),
                };

                let pressed = matches!(state, ElementState::Pressed);

                self.context.app.pointer_button(
                    window.id,
                    ike::PointerId::from_hash(device_id),
                    button,
                    pressed,
                );
            }

            WindowEvent::KeyboardInput { event, .. } => {
                let pressed = matches!(event.state, ElementState::Pressed);

                self.context.app.key_press(
                    window.id,
                    convert_winit_key(event.logical_key),
                    event.repeat,
                    event.text.as_deref(),
                    pressed,
                );
            }

            WindowEvent::ModifiersChanged(mods) => {
                let mut modifiers = ike::Modifiers::empty();

                if mods.state().shift_key() {
                    modifiers |= ike::Modifiers::SHIFT;
                }

                if mods.state().control_key() {
                    modifiers |= ike::Modifiers::CONTROL;
                }

                if mods.state().alt_key() {
                    modifiers |= ike::Modifiers::ALT;
                }

                if mods.state().super_key() {
                    modifiers |= ike::Modifiers::META;
                }

                self.context.app.modifiers_changed(window.id, modifiers);
            }

            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            _ => {}
        }

        for window in &mut self.windows {
            if self.context.app.window_needs_animate(window.id) && window.animate.is_none() {
                window.window.request_redraw();
                window.animate = Some(Instant::now());
            }

            if self.context.app.window_needs_draw(window.id) {
                window.window.request_redraw();
            }
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: Event) {
        match event {
            Event::Rebuild => {
                let mut view = (self.build)(self.data);
                ori::View::rebuild(
                    &mut view,
                    &mut ori::NoElement,
                    self.state.as_mut().unwrap(),
                    &mut self.context,
                    self.data,
                    &mut self.view,
                );

                self.view = view;
            }

            Event::Event(mut event) => {
                let action = ori::View::event(
                    &mut self.view,
                    &mut ori::NoElement,
                    self.state.as_mut().unwrap(),
                    &mut self.context,
                    self.data,
                    &mut event,
                );

                self.context.send_action(action);
            }

            Event::Spawn(future) => {
                self.runtime.spawn(future);
            }
        }

        for window in &self.windows {
            if self.context.app.window_needs_draw(window.id) {
                window.window.request_redraw();
            }
        }
    }
}

type AnyEffect<T> = Box<dyn ori::AnyView<Context, T, ori::NoElement>>;
type UiBuilder<T> = Box<dyn FnMut(&mut T) -> AnyEffect<T>>;

impl<T> AppState<'_, T> {
    fn create_gl_context(&mut self, event_loop: &ActiveEventLoop) {
        let (_, gl_config) = DisplayBuilder::new()
            .build(
                event_loop,
                ConfigTemplateBuilder::new(),
                |configs| {
                    configs
                        .reduce(
                            |acc, cfg| match cfg.num_samples() > acc.num_samples() {
                                true => cfg,
                                false => acc,
                            },
                        )
                        .unwrap()
                },
            )
            .unwrap();

        let attrs = ContextAttributes::default();

        let gl_context = unsafe {
            gl_config
                .display()
                .create_context(&gl_config, &attrs)
                .unwrap()
                .treat_as_possibly_current()
        };

        self.gl_state = Some(GlState {
            config:  gl_config,
            context: gl_context,
        });
    }

    fn update_windows(&mut self, event_loop: &ActiveEventLoop) {
        for window in self.context.app.windows() {
            if !self.windows.iter().any(|w| w.id == window.id()) {
                let gl = self.gl_state.as_ref().unwrap();
                self.windows.push(WindowState::new(gl, event_loop, window));
            }
        }

        self.windows.retain(|state| {
            let Some(_window) = self.context.app.get_window(state.id) else {
                return false;
            };

            true
        });
    }
}

impl WindowState {
    fn new(gl: &GlState, event_loop: &ActiveEventLoop, window: &ike::Window) -> Self {
        let attrs = Window::default_attributes().with_inner_size(LogicalSize::new(
            window.size.width,
            window.size.height,
        ));

        let win = event_loop.create_window(attrs).unwrap();

        let size = win.inner_size();

        let handle = win.window_handle().unwrap().as_raw();
        let attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
            handle,
            NonZeroU32::new(size.width).unwrap_or(NonZeroU32::MIN),
            NonZeroU32::new(size.height).unwrap_or(NonZeroU32::MIN),
        );

        let gl_surface = unsafe {
            gl.config
                .display()
                .create_window_surface(&gl.config, &attrs)
                .unwrap()
        };

        gl.context.make_current(&gl_surface).unwrap();

        let interface = skia_safe::gpu::gl::Interface::new_load_with_cstr(|name| {
            if name == c"eglGetCurrentDisplay" {
                return std::ptr::null();
            }

            gl.config.display().get_proc_address(name)
        })
        .unwrap();

        let mut skia_context = skia_safe::gpu::direct_contexts::make_gl(interface, None).unwrap();

        let fb_info = unsafe {
            let mut fboid: ffi::c_int = 0;

            let get_intergerv = gl.config.display().get_proc_address(c"glGetIntegerv");
            assert!(!get_intergerv.is_null());

            let get_intergerv: unsafe extern "C" fn(ffi::c_uint, *mut ffi::c_int) =
                mem::transmute(get_intergerv);

            get_intergerv(0x8ca6, &mut fboid);

            FramebufferInfo {
                fboid: fboid as u32,
                format: skia_safe::gpu::gl::Format::RGBA8.into(),
                ..Default::default()
            }
        };

        let render_target = skia_safe::gpu::backend_render_targets::make_gl(
            (size.width as i32, size.height as i32),
            gl.config.num_samples() as usize,
            gl.config.stencil_size() as usize,
            fb_info,
        );

        let skia_surface = skia_safe::gpu::surfaces::wrap_backend_render_target(
            &mut skia_context,
            &render_target,
            skia_safe::gpu::SurfaceOrigin::BottomLeft,
            skia_safe::ColorType::RGBA8888,
            None,
            None,
        )
        .unwrap();

        gl_surface.swap_buffers(&gl.context).unwrap();

        WindowState {
            fb_info,
            skia_surface,
            skia_context,
            gl_surface,

            animate: None,

            id: window.id(),
            window: win,
        }
    }

    fn resize(&mut self, gl: &GlState) {
        let size = self.window.inner_size();

        gl.context.make_current(&self.gl_surface).unwrap();

        let render_target = skia_safe::gpu::backend_render_targets::make_gl(
            (size.width as i32, size.height as i32),
            gl.config.num_samples() as usize,
            gl.config.stencil_size() as usize,
            self.fb_info,
        );

        self.skia_surface = skia_safe::gpu::surfaces::wrap_backend_render_target(
            &mut self.skia_context,
            &render_target,
            skia_safe::gpu::SurfaceOrigin::BottomLeft,
            skia_safe::ColorType::RGBA8888,
            None,
            None,
        )
        .unwrap();

        self.gl_surface.resize(
            &gl.context,
            NonZeroU32::new(size.width).unwrap_or(NonZeroU32::MIN),
            NonZeroU32::new(size.height).unwrap_or(NonZeroU32::MIN),
        );
    }
}
