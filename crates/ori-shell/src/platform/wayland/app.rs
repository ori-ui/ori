use ori_app::{App, AppBuilder, AppRequest, UiBuilder};
use ori_core::{
    command::CommandWaker,
    window::{Window, WindowId},
};
use ori_glow::GlowRenderer;
use wayland_client::{
    protocol::{wl_buffer, wl_compositor, wl_registry, wl_surface},
    Connection, Dispatch, Proxy, QueueHandle,
};
use wayland_egl::WlEglSurface;
use wayland_protocols::xdg::shell::client::{xdg_surface, xdg_toplevel, xdg_wm_base};

use crate::platform::linux::{EglContext, EglNativeDisplay, EglSurface, LIB_GL};

use super::error::WaylandError;

#[allow(unused)]
struct WaylandWindow {
    id: WindowId,
    physical_width: u32,
    physical_height: u32,
    renderer: GlowRenderer,
    egl_surface: EglSurface,
    wl_egl_surface: WlEglSurface,
    base_surface: wl_surface::WlSurface,
    xdg_toplevel: xdg_toplevel::XdgToplevel,
    xdg_surface: xdg_surface::XdgSurface,
    needs_redraw: bool,
}

/// Wayland platform application.
pub struct WaylandApp<T> {
    // keep here to ensure it's dropped first
    egl_context: EglContext,

    app: App<T>,
    data: T,
    conn: Connection,
    compositor: Option<wl_compositor::WlCompositor>,
    xdg_wm_base: Option<xdg_wm_base::XdgWmBase>,
    running: bool,

    windows: Vec<WaylandWindow>,
}

impl<T: 'static> WaylandApp<T> {
    /// Create a new Wayland platform application.
    pub fn new(app: AppBuilder<T>, data: T) -> Result<Self, WaylandError> {
        let conn = Connection::connect_to_env()?;

        let display_ptr = conn.backend().display_ptr();
        let egl_context = EglContext::new(EglNativeDisplay::Wayland(display_ptr as _))?;

        let waker = CommandWaker::new(|| todo!());

        let app = Self {
            app: app.build(waker),
            data,
            conn,
            compositor: None,
            xdg_wm_base: None,
            running: false,

            egl_context,
            windows: Vec::new(),
        };

        Ok(app)
    }

    /// Run the application.
    pub fn run(mut self) -> Result<(), WaylandError> {
        self.running = true;

        let mut event_queue = self.conn.new_event_queue();
        let qhandle = event_queue.handle();

        let display = self.conn.display();
        display.get_registry(&qhandle, ());

        while self.running {
            event_queue.blocking_dispatch(&mut self)?;
            self.handle_requests(&qhandle)?;
            self.render_windows()?;
        }

        Ok(())
    }
}

impl<T: 'static> WaylandApp<T> {
    fn handle_requests(&mut self, qhandle: &QueueHandle<Self>) -> Result<(), WaylandError> {
        for request in self.app.take_requests() {
            self.handle_request(qhandle, request)?;
        }

        Ok(())
    }

    fn handle_request(
        &mut self,
        qhandle: &QueueHandle<Self>,
        request: AppRequest<T>,
    ) -> Result<(), WaylandError> {
        match request {
            AppRequest::OpenWindow(window, ui) => self.open_window(qhandle, window, ui)?,
            AppRequest::CloseWindow(id) => {
                if let Some(i) = self.windows.iter().position(|w| w.id == id) {
                    self.windows.remove(i);
                }
            }
            AppRequest::DragWindow(_) => todo!(),
            AppRequest::RequestRedraw(id) => {
                if let Some(window) = self.windows.iter_mut().find(|w| w.id == id) {
                    window.needs_redraw = true;
                }
            }
            AppRequest::UpdateWindow(_, _) => todo!(),
            AppRequest::Quit => self.running = false,
        }

        Ok(())
    }

    fn open_window(
        &mut self,
        qhandle: &QueueHandle<Self>,
        window: Window,
        ui: UiBuilder<T>,
    ) -> Result<(), WaylandError> {
        let compositor = self.compositor.as_ref().unwrap();
        let xdg_wm_base = self.xdg_wm_base.as_ref().unwrap();

        let base_surface = compositor.create_surface(qhandle, ());
        let xdg_surface = xdg_wm_base.get_xdg_surface(&base_surface, qhandle, ());
        let xdg_toplevel = xdg_surface.get_toplevel(qhandle, ());
        xdg_toplevel.set_title(window.title.clone());

        base_surface.commit();

        let physical_width = window.width();
        let physical_height = window.height();

        let wl_egl_surface = WlEglSurface::new(
            base_surface.id(),
            physical_width as i32,
            physical_height as i32,
        )?;

        let surface_ptr = wl_egl_surface.ptr();
        let egl_surface = EglSurface::new(&self.egl_context, surface_ptr as _)?;

        egl_surface.make_current()?;
        egl_surface.swap_interval(0)?;

        let renderer = unsafe {
            GlowRenderer::new(|name| {
                let name = std::ffi::CString::new(name).unwrap();
                *LIB_GL.get(name.as_bytes_with_nul()).unwrap()
            })
        };

        let wayland_window = WaylandWindow {
            id: window.id(),
            physical_width,
            physical_height,
            base_surface,
            xdg_surface,
            xdg_toplevel,

            wl_egl_surface,
            egl_surface,
            renderer,
            needs_redraw: true,
        };

        self.windows.push(wayland_window);
        self.app.add_window(&mut self.data, ui, window);

        Ok(())
    }

    fn render_windows(&mut self) -> Result<(), WaylandError> {
        for window in &mut self.windows {
            if !window.needs_redraw {
                continue;
            }

            window.needs_redraw = false;

            if let Some(state) = self.app.draw_window(&mut self.data, window.id) {
                unsafe {
                    window.egl_surface.make_current()?;
                    window.renderer.render(
                        state.canvas,
                        state.clear_color,
                        window.physical_width,
                        window.physical_height,
                        1.0,
                    );
                    window.egl_surface.swap_buffers()?;
                }
            }
        }

        Ok(())
    }
}

impl<T: 'static> Dispatch<wl_registry::WlRegistry, ()> for WaylandApp<T> {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: <wl_registry::WlRegistry as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name, interface, ..
        } = event
        {
            match interface.as_str() {
                "wl_compositor" => {
                    let compositor = registry.bind(name, 1, qhandle, ());
                    state.compositor = Some(compositor);
                }
                "xdg_wm_base" => {
                    let xdg_wm_base = registry.bind(name, 1, qhandle, ());
                    state.xdg_wm_base = Some(xdg_wm_base);
                }
                _ => {}
            }
        }
    }
}

impl<T> Dispatch<wl_compositor::WlCompositor, ()> for WaylandApp<T> {
    fn event(
        _state: &mut Self,
        _compositor: &wl_compositor::WlCompositor,
        _event: <wl_compositor::WlCompositor as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}

impl<T> Dispatch<wl_buffer::WlBuffer, ()> for WaylandApp<T> {
    fn event(
        _state: &mut Self,
        _buffer: &wl_buffer::WlBuffer,
        _event: <wl_buffer::WlBuffer as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}

impl<T> Dispatch<wl_surface::WlSurface, ()> for WaylandApp<T> {
    fn event(
        _state: &mut Self,
        _surface: &wl_surface::WlSurface,
        _event: <wl_surface::WlSurface as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}

impl<T> Dispatch<xdg_wm_base::XdgWmBase, ()> for WaylandApp<T> {
    fn event(
        _state: &mut Self,
        xdg_wm_base: &xdg_wm_base::XdgWmBase,
        event: <xdg_wm_base::XdgWmBase as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        if let xdg_wm_base::Event::Ping { serial } = event {
            xdg_wm_base.pong(serial);
        }
    }
}

impl<T> Dispatch<xdg_surface::XdgSurface, ()> for WaylandApp<T> {
    fn event(
        _state: &mut Self,
        xdg_surface: &xdg_surface::XdgSurface,
        event: <xdg_surface::XdgSurface as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        if let xdg_surface::Event::Configure { serial } = event {
            xdg_surface.ack_configure(serial);
        }
    }
}

impl<T> Dispatch<xdg_toplevel::XdgToplevel, ()> for WaylandApp<T> {
    fn event(
        state: &mut Self,
        xdg_toplevel: &xdg_toplevel::XdgToplevel,
        event: <xdg_toplevel::XdgToplevel as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        let window = state
            .windows
            .iter_mut()
            .find(|w| w.xdg_toplevel.id() == xdg_toplevel.id())
            .unwrap();

        match event {
            xdg_toplevel::Event::Configure { width, height, .. } => {
                window.physical_width = width as u32;
                window.physical_height = height as u32;

                window.needs_redraw = true;

                window.wl_egl_surface.resize(width, height, 0, 0);

                state.app.window_resized(
                    &mut state.data,
                    window.id,
                    window.physical_width,
                    window.physical_height,
                );
            }
            xdg_toplevel::Event::Close => {
                state.app.close_requested(&mut state.data, window.id);
            }
            _ => {}
        }
    }
}
