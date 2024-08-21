use std::{mem, num::NonZero, sync::Arc};

use ori_app::{App, AppBuilder, AppRequest, UiBuilder};
use ori_core::{
    command::CommandWaker,
    event::{PointerButton, PointerId},
    layout::Point,
    window::{Cursor, Window, WindowId, WindowUpdate},
};
use ori_glow::GlowRenderer;
use sctk_adwaita::{AdwaitaFrame, FrameConfig};
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_output, delegate_pointer, delegate_registry, delegate_seat,
    delegate_shm, delegate_subcompositor, delegate_xdg_shell, delegate_xdg_window,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{
        pointer::{
            CursorIcon, PointerEvent, PointerEventKind, PointerHandler, ThemeSpec, ThemedPointer,
        },
        Capability, SeatHandler, SeatState,
    },
    shell::{
        xdg::{
            window::{
                DecorationMode, Window as XdgWindow, WindowConfigure, WindowDecorations,
                WindowHandler,
            },
            XdgShell, XdgSurface,
        },
        WaylandSurface,
    },
    shm::{Shm, ShmHandler},
    subcompositor::SubcompositorState,
};
use tracing::warn;
use wayland_client::{
    backend::ObjectId,
    globals::registry_queue_init,
    protocol::{
        wl_output::{Transform, WlOutput},
        wl_pointer::WlPointer,
        wl_seat::WlSeat,
        wl_surface::WlSurface,
    },
    Connection, Proxy, QueueHandle,
};
use wayland_csd_frame::DecorationsFrame;
use wayland_egl::WlEglSurface;

use crate::platform::linux::{EglContext, EglNativeDisplay, EglSurface, LIB_GL};

use super::error::WaylandError;

/// Launch an Ori application on the Wayland platform.
pub fn launch<T>(app: AppBuilder<T>, mut data: T) -> Result<(), WaylandError> {
    let conn = Connection::connect_to_env()?;
    let (globals, mut event_queue) = registry_queue_init(&conn).unwrap();
    let qhandle = event_queue.handle();

    let display = EglNativeDisplay::Wayland(conn.backend().display_ptr() as _);
    let egl_context = EglContext::new(display)?;

    let compositor = CompositorState::bind(&globals, &qhandle).unwrap();
    let subcompositor = SubcompositorState::bind(
        // why do we need to clone the compositor here?
        compositor.wl_compositor().clone(),
        &globals,
        &qhandle,
    )
    .unwrap();
    let xdg_shell = XdgShell::bind(&globals, &qhandle).unwrap();
    let seat = SeatState::new(&globals, &qhandle);
    let shm = Shm::bind(&globals, &qhandle).unwrap();

    let output = OutputState::new(&globals, &qhandle);
    let registry = RegistryState::new(&globals);

    let waker = CommandWaker::new(|| {});
    let mut app = app.build(waker);

    let mut state = State {
        running: true,

        egl_context,

        conn,
        compositor: Arc::new(compositor),
        subcompositor: Arc::new(subcompositor),
        xdg_shell,
        seat,
        shm,

        output,
        registry,

        pointers: Vec::new(),

        events: Vec::new(),
        windows: Vec::new(),
    };

    while state.running {
        event_queue.blocking_dispatch(&mut state)?;
        handle_events(&mut app, &mut data, &mut state)?;
        handle_app_requests(&mut app, &mut data, &mut state, &qhandle)?;
        render_windows(&mut app, &mut data, &mut state)?;
        handle_app_requests(&mut app, &mut data, &mut state, &qhandle)?;
        set_cursor_icons(&mut state);
    }

    Ok(())
}

fn handle_app_requests<T>(
    app: &mut App<T>,
    data: &mut T,
    state: &mut State,
    qhandle: &QueueHandle<State>,
) -> Result<(), WaylandError> {
    for request in app.take_requests() {
        handle_app_request(app, data, state, qhandle, request)?;
    }

    Ok(())
}

fn handle_app_request<T>(
    app: &mut App<T>,
    data: &mut T,
    state: &mut State,
    qhandle: &QueueHandle<State>,
    request: AppRequest<T>,
) -> Result<(), WaylandError> {
    match request {
        AppRequest::OpenWindow(window, ui) => open_window(app, data, state, qhandle, window, ui)?,

        AppRequest::CloseWindow(_) => {}

        AppRequest::DragWindow(_) => {}

        AppRequest::RequestRedraw(id) => {
            if let Some(window) = window_by_id(&mut state.windows, id) {
                window.needs_redraw = true;
            }
        }

        AppRequest::UpdateWindow(id, update) => {
            let Some(window) = window_by_id(&mut state.windows, id) else {
                return Ok(());
            };

            match update {
                WindowUpdate::Title(title) => {
                    window.xdg_window.set_title(&title);
                    window.xdg_window.commit();
                }
                WindowUpdate::Icon(_) => {}
                WindowUpdate::Size(size) => {
                    let physical_width = (size.width * window.scale_factor) as u32;
                    let physical_height = (size.height * window.scale_factor) as u32;

                    window.physical_width = physical_width;
                    window.physical_height = physical_height;

                    window
                        .xdg_window
                        .set_window_geometry(0, 0, physical_width, physical_height);
                    window.wl_egl_surface.resize(
                        physical_width as i32,
                        physical_height as i32,
                        0,
                        0,
                    );
                    window.xdg_window.commit();
                    window.needs_redraw = true;
                }
                WindowUpdate::Scale(scale) => {
                    window.scale_factor = scale;
                    window.needs_redraw = true;
                }
                WindowUpdate::Resizable(_) => {}
                WindowUpdate::Decorated(_) => {}
                WindowUpdate::Maximized(_) => {}
                WindowUpdate::Visible(_) => {}
                WindowUpdate::Color(_) => {
                    window.needs_redraw = true;
                }
                WindowUpdate::Cursor(cursor) => {
                    window.cursor_icon = cursor_icon(cursor);
                    window.set_cursor_icon = true;
                }
            }
        }

        AppRequest::Quit => state.running = false,
    }

    Ok(())
}

fn cursor_icon(cursor: Cursor) -> CursorIcon {
    match cursor {
        Cursor::Default => CursorIcon::Default,
        Cursor::Crosshair => CursorIcon::Crosshair,
        Cursor::Pointer => CursorIcon::Pointer,
        Cursor::Arrow => CursorIcon::Default,
        Cursor::Move => CursorIcon::Move,
        Cursor::Text => CursorIcon::Text,
        Cursor::Wait => CursorIcon::Wait,
        Cursor::Help => CursorIcon::Help,
        Cursor::Progress => CursorIcon::Progress,
        Cursor::NotAllowed => CursorIcon::NotAllowed,
        Cursor::ContextMenu => CursorIcon::ContextMenu,
        Cursor::Cell => CursorIcon::Cell,
        Cursor::VerticalText => CursorIcon::VerticalText,
        Cursor::Alias => CursorIcon::Alias,
        Cursor::Copy => CursorIcon::Copy,
        Cursor::NoDrop => CursorIcon::NoDrop,
        Cursor::Grab => CursorIcon::Grab,
        Cursor::Grabbing => CursorIcon::Grabbing,
        Cursor::AllScroll => CursorIcon::AllScroll,
        Cursor::ZoomIn => CursorIcon::ZoomIn,
        Cursor::ZoomOut => CursorIcon::ZoomOut,
        Cursor::EResize => CursorIcon::EResize,
        Cursor::NResize => CursorIcon::NResize,
        Cursor::NeResize => CursorIcon::NeResize,
        Cursor::NwResize => CursorIcon::NwResize,
        Cursor::SResize => CursorIcon::SResize,
        Cursor::SeResize => CursorIcon::SeResize,
        Cursor::SwResize => CursorIcon::SwResize,
        Cursor::WResize => CursorIcon::WResize,
        Cursor::EwResize => CursorIcon::EwResize,
        Cursor::NsResize => CursorIcon::NsResize,
        Cursor::NeswResize => CursorIcon::NeswResize,
        Cursor::NwseResize => CursorIcon::NwseResize,
        Cursor::ColResize => CursorIcon::ColResize,
        Cursor::RowResize => CursorIcon::RowResize,
    }
}

fn open_window<T>(
    app: &mut App<T>,
    data: &mut T,
    state: &mut State,
    qhandle: &QueueHandle<State>,
    window: Window,
    ui: UiBuilder<T>,
) -> Result<(), WaylandError> {
    let physical_width = window.width();
    let physical_height = window.height();

    let surface = state.compositor.create_surface(qhandle);
    let xdg_window = state.xdg_shell.create_window(
        surface,
        // We prefer to use the server-side decorations.
        WindowDecorations::RequestServer,
        qhandle,
    );

    xdg_window.set_title(&window.title);
    xdg_window.set_min_size(None);
    xdg_window.set_max_size(None);
    xdg_window.commit();

    xdg_window.xdg_surface().set_window_geometry(
        0,
        0,
        physical_width as i32,
        physical_height as i32,
    );

    let wl_egl_surface = WlEglSurface::new(
        xdg_window.wl_surface().id(),
        physical_width as i32,
        physical_height as i32,
    )?;
    let egl_surface = EglSurface::new(&state.egl_context, wl_egl_surface.ptr() as _)?;

    egl_surface.make_current()?;
    egl_surface.swap_interval(1)?;

    let renderer = unsafe { GlowRenderer::new(|symbol| *LIB_GL.get(symbol.as_bytes()).unwrap()) };

    let window_state = WindowState {
        id: window.id(),

        needs_redraw: true,
        physical_width,
        physical_height,
        scale_factor: 1.0,
        cursor_icon: CursorIcon::Default,
        set_cursor_icon: false,
        pointers: Vec::new(),
        title: window.title.clone(),

        wl_egl_surface,
        egl_surface,
        renderer,

        frame: None,
        xdg_window,
    };

    state.windows.push(window_state);
    app.add_window(data, ui, window);

    Ok(())
}

fn render_windows<T>(
    app: &mut App<T>,
    data: &mut T,
    state: &mut State,
) -> Result<(), WaylandError> {
    for window in &mut state.windows {
        if let Some(ref mut frame) = window.frame {
            if frame.is_dirty() && !frame.is_hidden() {
                frame.draw();
            }
        }

        if !window.needs_redraw {
            continue;
        }

        window.needs_redraw = false;

        if let Some(draw_state) = app.draw_window(data, window.id) {
            window.egl_surface.make_current()?;

            unsafe {
                window.renderer.render(
                    draw_state.canvas,
                    draw_state.clear_color,
                    window.physical_width,
                    window.physical_height,
                    window.scale_factor,
                );
            }

            window.egl_surface.swap_buffers()?;
        }
    }

    Ok(())
}

fn set_cursor_icons(state: &mut State) {
    for window in &mut state.windows {
        if !window.set_cursor_icon {
            continue;
        }

        window.set_cursor_icon = false;

        for pointer in &state.pointers {
            if !window.pointers.contains(&pointer.pointer().id()) {
                continue;
            }

            if let Err(err) = pointer.set_cursor(&state.conn, window.cursor_icon) {
                warn!("Failed to set cursor icon: {}", err);
            }
        }
    }
}

fn handle_events<T>(app: &mut App<T>, data: &mut T, state: &mut State) -> Result<(), WaylandError> {
    for event in mem::take(&mut state.events) {
        handle_event(app, data, state, event)?;
    }

    Ok(())
}

fn handle_event<T>(
    app: &mut App<T>,
    data: &mut T,
    state: &mut State,
    event: Event,
) -> Result<(), WaylandError> {
    match event {
        Event::Resized { id, width, height } => {
            app.window_resized(data, id, width, height);
        }

        Event::CloseRequested { id } => {
            if let Some(index) = window_index_by_id(&state.windows, id) {
                if app.close_requested(data, id) {
                    state.windows.remove(index);
                }
            }
        }

        Event::PointerMoved {
            id,
            object_id,
            position,
        } => {
            if let Some(window) = window_by_id(&mut state.windows, id) {
                let position = position / window.scale_factor;
                let pointer_id = PointerId::from_hash(&object_id);

                app.pointer_moved(data, id, pointer_id, position);
            }
        }

        Event::PointerButton {
            id,
            object_id,
            button,
            pressed,
        } => {
            let pointer_id = PointerId::from_hash(&object_id);
            app.pointer_button(data, id, pointer_id, button, pressed);
        }
    }

    Ok(())
}

struct State {
    running: bool,

    egl_context: EglContext,

    conn: Connection,
    compositor: Arc<CompositorState>,
    subcompositor: Arc<SubcompositorState>,
    xdg_shell: XdgShell,
    seat: SeatState,
    shm: Shm,

    output: OutputState,
    registry: RegistryState,

    pointers: Vec<ThemedPointer>,

    events: Vec<Event>,
    windows: Vec<WindowState>,
}

enum Event {
    Resized {
        id: WindowId,
        width: u32,
        height: u32,
    },

    CloseRequested {
        id: WindowId,
    },

    PointerMoved {
        id: WindowId,
        object_id: ObjectId,
        position: Point,
    },

    PointerButton {
        id: WindowId,
        object_id: ObjectId,
        button: PointerButton,
        pressed: bool,
    },
}

#[allow(unused)]
struct WindowState {
    id: WindowId,

    needs_redraw: bool,
    physical_width: u32,
    physical_height: u32,
    scale_factor: f32,
    cursor_icon: CursorIcon,
    set_cursor_icon: bool,
    pointers: Vec<ObjectId>,
    title: String,

    wl_egl_surface: WlEglSurface,
    egl_surface: EglSurface,
    renderer: GlowRenderer,

    frame: Option<AdwaitaFrame<State>>,
    xdg_window: XdgWindow,
}

fn window_index_by_id(windows: &[WindowState], id: WindowId) -> Option<usize> {
    windows.iter().position(|w| w.id == id)
}

fn window_by_id(windows: &mut [WindowState], id: WindowId) -> Option<&mut WindowState> {
    (windows.iter_mut()).find(|w| w.id == id)
}

fn window_by_surface<'a>(
    windows: &'a mut [WindowState],
    surface: &WlSurface,
) -> Option<&'a mut WindowState> {
    (windows.iter_mut()).find(|w| w.xdg_window.wl_surface() == surface)
}

impl CompositorHandler for State {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &WlSurface,
        _new_factor: i32,
    ) {
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &WlSurface,
        _new_transform: Transform,
    ) {
    }

    fn frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &WlSurface,
        _time: u32,
    ) {
    }

    fn surface_enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &WlSurface,
        _output: &WlOutput,
    ) {
    }

    fn surface_leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &WlSurface,
        _output: &WlOutput,
    ) {
    }
}

impl OutputHandler for State {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output
    }

    fn new_output(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _output: WlOutput) {}

    fn update_output(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _output: WlOutput) {}

    fn output_destroyed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _output: WlOutput) {
    }
}

impl WindowHandler for State {
    fn request_close(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, window: &XdgWindow) {
        if let Some(window) = window_by_surface(&mut self.windows, window.wl_surface()) {
            self.events.push(Event::CloseRequested { id: window.id });
        }
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        window: &XdgWindow,
        configure: WindowConfigure,
        _serial: u32,
    ) {
        if let Some(window) = window_by_surface(&mut self.windows, window.wl_surface()) {
            let (width, height) = configure.new_size;

            match configure.decoration_mode {
                DecorationMode::Client => {
                    let frame = window.frame.get_or_insert_with(|| {
                        let mut frame = AdwaitaFrame::new(
                            &window.xdg_window,
                            &self.shm,
                            self.compositor.clone(),
                            self.subcompositor.clone(),
                            qh.clone(),
                            FrameConfig::auto(),
                        )
                        .unwrap();
                        frame.set_title(window.title.clone());
                        frame
                    });

                    frame.set_hidden(false);
                    frame.update_state(configure.state);
                    frame.update_wm_capabilities(configure.capabilities);

                    let (current_width, current_height) = frame.add_borders(
                        //
                        window.physical_width,
                        window.physical_height,
                    );

                    let one = NonZero::new(1).unwrap();
                    let (width, height) = frame.subtract_borders(
                        width.unwrap_or(NonZero::new(current_width).unwrap_or(one)),
                        height.unwrap_or(NonZero::new(current_height).unwrap_or(one)),
                    );

                    let width = width.unwrap_or(one);
                    let height = height.unwrap_or(one);

                    frame.resize(width, height);

                    let (x, y) = frame.location();
                    let (outer_width, outer_height) = frame.add_borders(width.get(), height.get());
                    window.xdg_window.xdg_surface().set_window_geometry(
                        x,
                        y,
                        outer_width as i32,
                        outer_height as i32,
                    );

                    window.physical_width = width.get();
                    window.physical_height = height.get();
                    window.needs_redraw = true;
                    (window.wl_egl_surface).resize(width.get() as i32, height.get() as i32, 0, 0);

                    self.events.push(Event::Resized {
                        id: window.id,
                        width: width.get(),
                        height: height.get(),
                    });
                }
                DecorationMode::Server => {
                    let width = width.map_or(window.physical_width, |w| w.get());
                    let height = height.map_or(window.physical_height, |h| h.get());

                    window.physical_width = width;
                    window.physical_height = height;
                    window.needs_redraw = true;
                    (window.wl_egl_surface).resize(width as i32, height as i32, 0, 0);

                    window.xdg_window.set_window_geometry(0, 0, width, height);

                    self.events.push(Event::Resized {
                        id: window.id,
                        width,
                        height,
                    });
                }
            }
        }
    }
}

impl ShmHandler for State {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl SeatHandler for State {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat
    }

    fn new_seat(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _seat: WlSeat) {}

    fn new_capability(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        seat: WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Pointer {
            let surface = self.compositor.create_surface(qh);
            let pointer = self.seat.get_pointer_with_theme(
                qh,
                &seat,
                self.shm.wl_shm(),
                surface,
                ThemeSpec::default(),
            );

            if let Ok(pointer) = pointer {
                self.pointers.push(pointer);
            }
        }
    }

    fn remove_capability(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _seat: WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Pointer {
            for pointer in self.pointers.drain(..) {
                pointer.pointer().release();
            }
        }
    }

    fn remove_seat(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _seat: WlSeat) {}
}

impl PointerHandler for State {
    fn pointer_frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        pointer: &WlPointer,
        events: &[PointerEvent],
    ) {
        for event in events {
            let Some(window) = window_by_surface(&mut self.windows, &event.surface) else {
                continue;
            };

            match event.kind {
                PointerEventKind::Enter { .. } => {
                    window.pointers.push(pointer.id());
                    window.set_cursor_icon = true;
                }

                PointerEventKind::Leave { .. } => {
                    window.pointers.retain(|id| *id != pointer.id());
                }

                PointerEventKind::Motion { .. } => {
                    let (x, y) = event.position;
                    let position = Point::new(x as f32, y as f32);

                    self.events.push(Event::PointerMoved {
                        id: window.id,
                        object_id: pointer.id(),
                        position,
                    });
                }

                PointerEventKind::Press { button, .. } => {
                    self.events.push(Event::PointerButton {
                        id: window.id,
                        object_id: pointer.id(),
                        button: pointer_button(button),
                        pressed: true,
                    });
                }

                PointerEventKind::Release { button, .. } => {
                    self.events.push(Event::PointerButton {
                        id: window.id,
                        object_id: pointer.id(),
                        button: pointer_button(button),
                        pressed: false,
                    });
                }
                PointerEventKind::Axis { .. } => {}
            }
        }
    }
}

fn pointer_button(button: u32) -> PointerButton {
    match button {
        0x110 => PointerButton::Primary,
        0x111 => PointerButton::Secondary,
        0x112 => PointerButton::Tertiary,
        other => PointerButton::Other(other as u16),
    }
}

impl ProvidesRegistryState for State {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry
    }

    registry_handlers!(OutputState);
}

delegate_compositor!(State);
delegate_subcompositor!(State);
delegate_output!(State);
delegate_shm!(State);

delegate_seat!(State);
delegate_pointer!(State);

delegate_xdg_shell!(State);
delegate_xdg_window!(State);

delegate_registry!(State);
