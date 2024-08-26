use std::{mem, num::NonZero, sync::Arc, time::Duration};

use ori_app::{App, AppBuilder, AppRequest, UiBuilder};
use ori_core::{
    clipboard::{Clipboard, ClipboardBackend},
    command::CommandWaker,
    event::{Code, Key, PointerButton, PointerId},
    layout::{Point, Vector},
    window::{Cursor, Window, WindowId, WindowUpdate},
};
use ori_glow::GlowRenderer;
use sctk_adwaita::{AdwaitaFrame, FrameConfig};
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState, SurfaceData},
    delegate_compositor, delegate_output, delegate_pointer, delegate_registry, delegate_seat,
    delegate_shm, delegate_subcompositor, delegate_xdg_shell, delegate_xdg_window,
    output::{OutputHandler, OutputState},
    reexports::{
        calloop::{
            timer::{TimeoutAction, Timer},
            EventLoop, LoopHandle, RegistrationToken,
        },
        calloop_wayland_source::WaylandSource,
        protocols::xdg::shell::client::xdg_toplevel::ResizeEdge as XdgResizeEdge,
    },
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{
        pointer::{
            CursorIcon, PointerData, PointerEvent, PointerEventKind, PointerHandler, ThemeSpec,
            ThemedPointer,
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
use tracing::{debug, warn};
use wayland_client::{
    backend::ObjectId,
    globals::registry_queue_init,
    protocol::{
        wl_keyboard::{Event as KeyboardEvent, KeyState, KeymapFormat, WlKeyboard},
        wl_output::{Transform, WlOutput},
        wl_pointer::WlPointer,
        wl_seat::WlSeat,
        wl_surface::WlSurface,
    },
    Connection, Dispatch, Proxy, QueueHandle, WEnum,
};
use wayland_csd_frame::{
    DecorationsFrame, FrameAction, FrameClick, ResizeEdge, WindowState as CsdWindowState,
};
use wayland_egl::WlEglSurface;
use xkeysym::Keysym;

use crate::platform::linux::{
    egl::{EglContext, EglNativeDisplay, EglSurface},
    xkb::{XkbContext, XkbKeyboard},
    LIB_GL,
};

use super::error::WaylandError;

/// Launch an Ori application on the Wayland platform.
pub fn launch<T>(app: AppBuilder<T>, data: &mut T) -> Result<(), WaylandError> {
    let conn = Connection::connect_to_env()?;
    let (globals, event_queue) = registry_queue_init(&conn)?;
    let qhandle = event_queue.handle();

    let mut event_loop = EventLoop::try_new().unwrap();
    let loop_handle = event_loop.handle();
    WaylandSource::new(conn.clone(), event_queue)
        .insert(loop_handle.clone())
        .unwrap();

    let display_ptr = conn.backend().display_ptr() as _;
    let display = EglNativeDisplay::Wayland(display_ptr);
    let egl_context = EglContext::new(display)?;

    let xkb_context = XkbContext::new().unwrap();

    let clipboard = unsafe { smithay_clipboard::Clipboard::new(display_ptr) };
    let clipboard = WaylandClipboard { clipboard };

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

    let waker = CommandWaker::new({
        let loop_signal = event_loop.get_signal();
        move || loop_signal.wakeup()
    });

    let mut app = app.build(waker);
    app.add_context(Clipboard::new(Box::new(clipboard)));
    app.init(data);

    let mut state = State {
        running: true,

        egl_context,
        xkb_context,

        conn,
        loop_handle,

        compositor: Arc::new(compositor),
        subcompositor: Arc::new(subcompositor),
        xdg_shell,
        seat,
        shm,

        output,
        registry,

        pointers: Vec::new(),
        keyboards: Vec::new(),

        events: Vec::new(),
        windows: Vec::new(),
    };

    while state.running {
        let timeout = match state.needs_redraw() {
            true => Some(Duration::from_millis(2)),
            false => None,
        };

        event_loop.dispatch(timeout, &mut state).unwrap();
        app.handle_commands(data);

        handle_events(&mut app, data, &mut state)?;
        handle_app_requests(&mut app, data, &mut state, &qhandle)?;

        render_windows(&mut app, data, &mut state)?;
        handle_app_requests(&mut app, data, &mut state, &qhandle)?;

        app.idle(data);
        handle_app_requests(&mut app, data, &mut state, &qhandle)?;

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

        AppRequest::CloseWindow(id) => {
            if let Some(index) = window_index_by_id(&state.windows, id) {
                state.windows.remove(index);
            }
        }

        AppRequest::DragWindow(id) => {
            if let Some(window) = window_by_id(&mut state.windows, id) {
                if let Some(pointer_id) = window.pointers.last() {
                    let pointer = pointer_by_id(&mut state.pointers, pointer_id.clone()).unwrap();
                    (window.xdg_window).move_(&pointer.seat, pointer.last_button_serial);
                }
            }
        }

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
                    if let Some(ref mut frame) = window.frame {
                        frame.set_title(&title);
                    }

                    window.xdg_window.set_title(&title);
                    window.xdg_window.commit();
                }
                WindowUpdate::Icon(_) => {
                    warn!(
                        "Setting window icons is not supported on Wayland, set it a .desktop file"
                    );
                }
                WindowUpdate::Size(size) => {
                    let physical_width = (size.width * window.scale_factor) as u32;
                    let physical_height = (size.height * window.scale_factor) as u32;

                    if let Some(ref mut configure) = window.last_configure {
                        let one = NonZero::new(1).unwrap();
                        let width = NonZero::new(physical_width).unwrap_or(one);
                        let height = NonZero::new(physical_height).unwrap_or(one);

                        configure.new_size = (Some(width), Some(height));
                    }

                    if let Some(event) = window.resize() {
                        state.events.push(event);
                    }
                }
                WindowUpdate::Scale(scale) => {
                    window.scale_factor = scale;
                    window.needs_redraw = true;
                }
                WindowUpdate::Resizable(resizable) => {
                    set_resizable(window, resizable);
                    window.resizable = resizable;
                }
                WindowUpdate::Decorated(decorated) => {
                    window.decorated = decorated;

                    let mode = match decorated {
                        true => DecorationMode::Server,
                        false => DecorationMode::Client,
                    };

                    window.xdg_window.request_decoration_mode(Some(mode));

                    if let Some(ref mut frame) = window.frame {
                        frame.set_hidden(!decorated);

                        if let Some(event) = window.resize() {
                            state.events.push(event);
                        }
                    }
                }
                WindowUpdate::Maximized(maximized) => {
                    match maximized {
                        true => window.xdg_window.set_maximized(),
                        false => window.xdg_window.unset_maximized(),
                    }

                    window.xdg_window.commit();
                }
                WindowUpdate::Visible(_) => {
                    warn!("Setting window visibility is not supported on Wayland");
                }
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

    if window.icon.is_some() {
        debug!("Window icons are not supported on Wayland, set it a .desktop file");
    }

    let window_state = WindowState {
        id: window.id(),

        needs_redraw: true,
        physical_width,
        physical_height,
        scale_factor: 1.0,
        cursor_icon: CursorIcon::Default,
        frame_cursor_icon: None,
        set_cursor_icon: false,
        title: window.title.clone(),
        resizable: window.resizable,
        decorated: window.decorated,
        last_configure: None,

        pointers: Vec::new(),
        keyboards: Vec::new(),

        wl_egl_surface,
        egl_surface,
        renderer,

        frame: None,
        xdg_window,
    };

    set_resizable(&window_state, window.resizable);

    state.windows.push(window_state);
    app.add_window(data, ui, window);

    Ok(())
}

fn set_resizable(window: &WindowState, resizable: bool) {
    if resizable {
        window.xdg_window.set_min_size(None);
        window.xdg_window.set_max_size(None);
    } else {
        let size = Some((window.physical_width, window.physical_height));
        window.xdg_window.set_min_size(size);
        window.xdg_window.set_max_size(size);
    }

    window.xdg_window.commit();
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

        let cursor_icon = window.frame_cursor_icon.unwrap_or(window.cursor_icon);

        for pointer in &state.pointers {
            if !window.pointers.contains(&pointer.pointer.pointer().id()) {
                continue;
            }

            if let Err(err) = pointer.pointer.set_cursor(&state.conn, cursor_icon) {
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

        Event::Scaled { id, scale } => {
            app.window_scaled(data, id, scale);
        }

        Event::State {
            id,
            state: win_state,
        } => {
            if let Some(window) = window_by_id(&mut state.windows, id) {
                let app_window = app.get_window_mut(id).expect("Window exists in state.app");

                let maximized = win_state.contains(CsdWindowState::MAXIMIZED);

                if window.resizable {
                    app_window.maximized = maximized;
                }
            }
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

        Event::PointerScroll {
            id,
            object_id,
            delta,
        } => {
            let pointer_id = PointerId::from_hash(&object_id);
            app.pointer_scrolled(data, id, pointer_id, delta);
        }

        Event::Keyboard {
            id,
            key,
            code,
            text,
            pressed,
        } => {
            app.keyboard_key(data, id, key, code, text, pressed);
        }

        Event::Modifiers { modifiers } => {
            app.modifiers_changed(modifiers);
        }
    }

    Ok(())
}

struct State {
    running: bool,

    egl_context: EglContext,
    xkb_context: XkbContext,

    conn: Connection,
    loop_handle: LoopHandle<'static, State>,

    compositor: Arc<CompositorState>,
    subcompositor: Arc<SubcompositorState>,
    xdg_shell: XdgShell,
    seat: SeatState,
    shm: Shm,

    output: OutputState,
    registry: RegistryState,

    pointers: Vec<PointerState>,
    keyboards: Vec<KeyboardState>,

    events: Vec<Event>,
    windows: Vec<WindowState>,
}

impl State {
    fn needs_redraw(&self) -> bool {
        self.windows.iter().any(|w| w.needs_redraw)
    }
}

struct PointerState {
    seat: WlSeat,
    pointer: ThemedPointer,
    last_button_serial: u32,
}

#[allow(unused)]
struct KeyboardState {
    seat: WlSeat,
    keyboard: WlKeyboard,
    xkb_keyboard: XkbKeyboard,

    // gap, delay
    repeat: Option<(Duration, Duration)>,
    repeat_keysym: Option<Keysym>,
    repeat_token: Option<RegistrationToken>,
}

enum Event {
    Resized {
        id: WindowId,
        width: u32,
        height: u32,
    },

    Scaled {
        id: WindowId,
        scale: f32,
    },

    State {
        id: WindowId,
        state: CsdWindowState,
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

    PointerScroll {
        id: WindowId,
        object_id: ObjectId,
        delta: Vector,
    },

    Keyboard {
        id: WindowId,
        key: Key,
        code: Option<Code>,
        text: Option<String>,
        pressed: bool,
    },

    Modifiers {
        modifiers: ori_core::event::Modifiers,
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
    frame_cursor_icon: Option<CursorIcon>,
    set_cursor_icon: bool,
    title: String,
    resizable: bool,
    decorated: bool,
    last_configure: Option<WindowConfigure>,

    pointers: Vec<ObjectId>,
    keyboards: Vec<ObjectId>,

    wl_egl_surface: WlEglSurface,
    egl_surface: EglSurface,
    renderer: GlowRenderer,

    frame: Option<AdwaitaFrame<State>>,
    xdg_window: XdgWindow,
}

impl WindowState {
    fn resize(&mut self) -> Option<Event> {
        let Some(ref configure) = self.last_configure else {
            warn!("No last configure event for window {}", self.id);
            return None;
        };

        let (width, height) = configure.new_size;

        match configure.decoration_mode {
            DecorationMode::Client if self.decorated => {
                let Some(ref mut frame) = self.frame else {
                    warn!("No frame for window {}", self.id);
                    return None;
                };

                frame.set_hidden(false);
                frame.update_state(configure.state);
                frame.update_wm_capabilities(configure.capabilities);

                let (current_width, current_height) = frame.add_borders(
                    //
                    self.physical_width,
                    self.physical_height,
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

                self.physical_width = width.get();
                self.physical_height = height.get();
                self.needs_redraw = true;

                let (outer_width, outer_height) = frame.add_borders(width.get(), height.get());

                // i have no idea why this is necessary, but it is
                //
                // KEEP MAKE CURRENT HERE!
                self.egl_surface.make_current().unwrap();
                (self.wl_egl_surface).resize(width.get() as i32, height.get() as i32, 0, 0);
                self.xdg_window.xdg_surface().set_window_geometry(
                    x,
                    y,
                    outer_width as i32,
                    outer_height as i32,
                );

                Some(Event::Resized {
                    id: self.id,
                    width: width.get(),
                    height: height.get(),
                })
            }
            _ => {
                if let Some(ref mut frame) = self.frame {
                    frame.set_hidden(true);
                }

                let width = width.map_or(self.physical_width, |w| w.get());
                let height = height.map_or(self.physical_height, |h| h.get());

                self.physical_width = width;
                self.physical_height = height;
                self.needs_redraw = true;

                // i have no idea why this is necessary, but it is
                //
                // KEEP MAKE CURRENT HERE!
                self.egl_surface.make_current().unwrap();
                (self.wl_egl_surface).resize(width as i32, height as i32, 0, 0);
                self.xdg_window.set_window_geometry(0, 0, width, height);

                Some(Event::Resized {
                    id: self.id,
                    width,
                    height,
                })
            }
        }
    }
}

fn window_index_by_id(windows: &[WindowState], id: WindowId) -> Option<usize> {
    windows.iter().position(|w| w.id == id)
}

fn window_by_id(windows: &mut [WindowState], id: WindowId) -> Option<&mut WindowState> {
    (windows.iter_mut()).find(|w| w.id == id)
}

fn pointer_by_id(pointers: &mut [PointerState], id: ObjectId) -> Option<&mut PointerState> {
    pointers.iter_mut().find(|p| p.pointer.pointer().id() == id)
}

fn keyboard_by_id(keyboards: &mut [KeyboardState], id: ObjectId) -> Option<&mut KeyboardState> {
    keyboards.iter_mut().find(|k| k.keyboard.id() == id)
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
        surface: &WlSurface,
        new_factor: i32,
    ) {
        if let Some(window) = window_by_surface(&mut self.windows, surface) {
            if let Some(ref mut frame) = window.frame {
                frame.set_scaling_factor(new_factor as f64);
            }

            window.scale_factor = new_factor as f32;
            window.needs_redraw = true;

            self.events.push(Event::Scaled {
                id: window.id,
                scale: new_factor as f32,
            });
        }
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
            window.last_configure = Some(configure.clone());

            self.events.push(Event::State {
                id: window.id,
                state: configure.state,
            });

            if configure.decoration_mode == DecorationMode::Client
                && window.decorated
                && window.frame.is_none()
            {
                let mut frame = AdwaitaFrame::new(
                    &window.xdg_window,
                    &self.shm,
                    self.compositor.clone(),
                    self.subcompositor.clone(),
                    qh.clone(),
                    FrameConfig::auto(),
                )
                .unwrap();

                frame.set_title(&window.title);
                window.frame = Some(frame);
            }

            if let Some(event) = window.resize() {
                self.events.push(event);
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
                let state = PointerState {
                    seat: seat.clone(),
                    pointer,
                    last_button_serial: 0,
                };

                self.pointers.push(state);
            }
        }

        if capability == Capability::Keyboard {
            let keyboard = seat.get_keyboard(qh, ());
            let xkb_keyboard = XkbKeyboard::new(&self.xkb_context).unwrap();

            let state = KeyboardState {
                seat,
                keyboard,
                xkb_keyboard,

                repeat: None,
                repeat_keysym: None,
                repeat_token: None,
            };

            self.keyboards.push(state);
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
                pointer.pointer.pointer().release();
            }
        }

        if capability == Capability::Keyboard {
            for keyboard in self.keyboards.drain(..) {
                keyboard.keyboard.release();
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
            let surface = &event.surface;

            let parent_surface = match event.surface.data::<SurfaceData>() {
                Some(data) => data.parent_surface().unwrap_or(surface),
                None => continue,
            };

            let Some(window) = window_by_surface(&mut self.windows, parent_surface) else {
                continue;
            };

            match event.kind {
                PointerEventKind::Enter { .. } | PointerEventKind::Motion { .. }
                    if surface != parent_surface =>
                {
                    let (x, y) = event.position;

                    if let Some(ref mut frame) = window.frame {
                        window.frame_cursor_icon = frame.click_point_moved(
                            // winit uses Duration::ZERO, and so will we
                            Duration::ZERO,
                            &event.surface.id(),
                            x,
                            y,
                        );
                        window.set_cursor_icon = true;
                    }
                }

                PointerEventKind::Leave { .. } if surface != parent_surface => {
                    if let Some(ref mut frame) = window.frame {
                        frame.click_point_left();
                    }
                }

                PointerEventKind::Press {
                    button,
                    serial,
                    time,
                }
                | PointerEventKind::Release {
                    button,
                    serial,
                    time,
                } if surface != parent_surface => {
                    let pressed = matches!(event.kind, PointerEventKind::Press { .. });

                    let click = match button {
                        0x110 => FrameClick::Normal,
                        0x111 => FrameClick::Alternate,
                        _ => continue,
                    };

                    if let Some(ref mut frame) = window.frame {
                        let pointer_data = pointer.data::<PointerData>().unwrap();
                        let seat = pointer_data.seat();

                        match frame.on_click(Duration::from_millis(time as u64), click, pressed) {
                            Some(FrameAction::Close) => {
                                self.events.push(Event::CloseRequested { id: window.id });
                            }
                            Some(FrameAction::Minimize) => {
                                window.xdg_window.set_minimized();
                                window.xdg_window.commit();
                            }
                            Some(FrameAction::Maximize) => {
                                window.xdg_window.set_maximized();
                                window.xdg_window.commit();
                            }
                            Some(FrameAction::UnMaximize) => {
                                window.xdg_window.unset_maximized();
                                window.xdg_window.commit();
                            }
                            Some(FrameAction::ShowMenu(x, y)) => {
                                window.xdg_window.show_window_menu(seat, serial, (x, y));
                            }
                            Some(FrameAction::Resize(edge)) => {
                                let edge = match edge {
                                    ResizeEdge::None => XdgResizeEdge::None,
                                    ResizeEdge::Top => XdgResizeEdge::Top,
                                    ResizeEdge::Bottom => XdgResizeEdge::Bottom,
                                    ResizeEdge::Left => XdgResizeEdge::Left,
                                    ResizeEdge::TopLeft => XdgResizeEdge::TopLeft,
                                    ResizeEdge::BottomLeft => XdgResizeEdge::BottomLeft,
                                    ResizeEdge::Right => XdgResizeEdge::Right,
                                    ResizeEdge::TopRight => XdgResizeEdge::TopRight,
                                    ResizeEdge::BottomRight => XdgResizeEdge::BottomRight,
                                    _ => continue,
                                };

                                window.xdg_window.resize(seat, serial, edge);
                            }
                            Some(FrameAction::Move) => {
                                window.xdg_window.move_(seat, serial);
                            }
                            Some(_) => {}
                            None => {}
                        }
                    }
                }

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

                PointerEventKind::Press { button, serial, .. }
                | PointerEventKind::Release { button, serial, .. } => {
                    let pressed = matches!(event.kind, PointerEventKind::Press { .. });

                    if let Some(pointer) = pointer_by_id(&mut self.pointers, pointer.id()) {
                        pointer.last_button_serial = serial;
                    }

                    self.events.push(Event::PointerButton {
                        id: window.id,
                        object_id: pointer.id(),
                        button: pointer_button(button),
                        pressed,
                    });
                }

                PointerEventKind::Axis {
                    horizontal,
                    vertical,
                    ..
                } => {
                    let delta = Vector::new(-horizontal.discrete as f32, -vertical.discrete as f32);

                    self.events.push(Event::PointerScroll {
                        id: window.id,
                        object_id: pointer.id(),
                        delta,
                    });
                }
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

impl Dispatch<WlKeyboard, ()> for State {
    fn event(
        state: &mut Self,
        proxy: &WlKeyboard,
        event: <WlKeyboard as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        let Some(keyboard) = keyboard_by_id(&mut state.keyboards, proxy.id()) else {
            return;
        };

        match event {
            KeyboardEvent::Keymap { format, fd, size } => {
                if !matches!(format, WEnum::Value(KeymapFormat::XkbV1)) {
                    warn!("Unsupported keymap format: {:?}", format);
                    return;
                }

                (keyboard.xkb_keyboard)
                    .set_keymap_from_fd(fd, size as usize)
                    .unwrap();
            }
            KeyboardEvent::Enter { surface, .. } => {
                if let Some(window) = window_by_surface(&mut state.windows, &surface) {
                    window.keyboards.push(keyboard.keyboard.id());
                }
            }
            KeyboardEvent::Leave { surface, .. } => {
                if let Some(window) = window_by_surface(&mut state.windows, &surface) {
                    window.keyboards.retain(|id| *id != keyboard.keyboard.id());
                }
            }
            KeyboardEvent::Key {
                key: scancode,
                state: key_state,
                ..
            } => {
                let (Some(xkb_state), Some(keymap)) = (
                    keyboard.xkb_keyboard.state(),
                    keyboard.xkb_keyboard.keymap(),
                ) else {
                    warn!("No keymap or state found for keyboard");
                    return;
                };

                let layout = xkb_state.layout();

                let keycode = scancode + 8;
                let code = Code::from_linux_scancode(scancode as u8);
                let keysym_raw = keymap.first_keysym(layout, keycode).unwrap();
                let keysym = xkb_state.get_one_sym(keycode);
                let key = keyboard.xkb_keyboard.keysym_to_key(keysym_raw);
                let text = keyboard.xkb_keyboard.keysym_to_utf8(keysym);

                let pressed = matches!(key_state, WEnum::Value(KeyState::Pressed));

                let mut window_ids = Vec::new();

                for window in &mut state.windows {
                    if window.keyboards.contains(&keyboard.keyboard.id()) {
                        window_ids.push(window.id);
                    }
                }

                for &window_id in &window_ids {
                    state.events.push(Event::Keyboard {
                        id: window_id,
                        key,
                        code,
                        text: text.clone(),
                        pressed,
                    });
                }

                if !pressed {
                    keyboard.repeat_keysym = None;

                    if let Some(token) = keyboard.repeat_token.take() {
                        state.loop_handle.remove(token);
                    }

                    return;
                }

                let Some((_, delay)) = keyboard.repeat else {
                    return;
                };

                if !keymap.key_repeats(keycode) {
                    return;
                }

                keyboard.repeat_keysym = Some(keysym);

                let timer = Timer::from_duration(delay);
                let kb_id = keyboard.keyboard.id();
                let token = state.loop_handle.insert_source(timer, move |_, _, state| {
                    let Some(keyboard) = keyboard_by_id(&mut state.keyboards, kb_id.clone()) else {
                        return TimeoutAction::Drop;
                    };

                    if keyboard.repeat_keysym != Some(keysym) {
                        return TimeoutAction::Drop;
                    }

                    for &window in &window_ids {
                        state.events.push(Event::Keyboard {
                            id: window,
                            key,
                            code,
                            text: text.clone(),
                            pressed: true,
                        });
                    }

                    match keyboard.repeat {
                        Some((gap, _)) => TimeoutAction::ToDuration(gap),
                        None => TimeoutAction::Drop,
                    }
                });

                keyboard.repeat_token = token.ok();
            }
            KeyboardEvent::Modifiers {
                mods_depressed,
                mods_latched,
                mods_locked,
                group,
                ..
            } => {
                if let Some(xkb_state) = keyboard.xkb_keyboard.state() {
                    xkb_state.update_modifiers(
                        mods_depressed,
                        mods_latched,
                        mods_locked,
                        0,
                        0,
                        group,
                    );
                    let modifiers = xkb_state.modifiers();
                    state.events.push(Event::Modifiers { modifiers });
                }
            }
            KeyboardEvent::RepeatInfo { rate, delay } => match rate {
                0 => {
                    keyboard.repeat_keysym = None;
                    keyboard.repeat = None;

                    if let Some(token) = keyboard.repeat_token.take() {
                        state.loop_handle.remove(token);
                    }
                }
                _ => {
                    let rate = Duration::from_micros(1_000_000 / rate as u64);
                    let delay = Duration::from_millis(delay as u64);

                    keyboard.repeat = Some((rate, delay));
                }
            },
            _ => {}
        }
    }
}

impl ProvidesRegistryState for State {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry
    }

    registry_handlers!(OutputState);
}

struct WaylandClipboard {
    clipboard: smithay_clipboard::Clipboard,
}

impl ClipboardBackend for WaylandClipboard {
    fn get_text(&mut self) -> String {
        self.clipboard.load().unwrap_or_default()
    }

    fn set_text(&mut self, text: &str) {
        self.clipboard.store(text);
    }
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
