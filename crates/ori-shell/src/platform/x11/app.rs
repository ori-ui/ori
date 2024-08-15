use std::{
    sync::{
        mpsc::{Receiver, Sender},
        Arc, LazyLock,
    },
    thread::{self, JoinHandle},
};

use ori_app::{App, AppBuilder, AppRequest, UiBuilder};
use ori_core::{
    command::CommandWaker,
    event::{Code, Modifiers, PointerButton, PointerId},
    layout::{Point, Vector},
    window::{Window, WindowId, WindowUpdate},
};
use ori_glow::GlowRenderer;

use libloading::Library;
use x11rb::{
    atom_manager,
    connection::Connection,
    properties::{WmSizeHints, WmSizeHintsSpecification},
    protocol::{
        render::{ConnectionExt as _, PictType},
        xkb::{
            ConnectionExt as _, EventType as XkbEventType, MapPart as XkbMapPart,
            SelectEventsAux as XkbSelectEventsAux, ID as XkbID,
        },
        xproto::{
            AtomEnum, ColormapAlloc, ConfigureWindowAux, ConnectionExt as _, CreateWindowAux,
            EventMask, ModMask, PropMode, Visualid, WindowClass,
        },
        Event as XEvent,
    },
    wrapper::ConnectionExt as _,
    xcb_ffi::XCBConnection,
};
use xkbcommon::xkb;

use crate::platform::linux::{EglContext, EglSurface, XkbKeyboard};

use super::X11Error;

static LIB_GL: LazyLock<Library> = LazyLock::new(|| {
    // load libGL.so
    unsafe { Library::new("libGL.so").unwrap() }
});

atom_manager! {
    Atoms: AtomsCookie {
        UTF8_STRING,
        WM_PROTOCOLS,
        WM_DELETE_WINDOW,
        _NET_WM_NAME,
        _NET_WM_ICON,
    }
}

struct X11Window {
    x11_id: u32,
    ori_id: WindowId,
    physical_width: u32,
    physical_height: u32,
    egl_surface: EglSurface,
    renderer: GlowRenderer,
    needs_redraw: bool,
}

impl X11Window {
    fn set_size_hints(
        &self,
        conn: &XCBConnection,
        width: i32,
        height: i32,
        resizable: bool,
    ) -> Result<(), X11Error> {
        use WmSizeHintsSpecification::*;

        let spec = match resizable {
            true => UserSpecified,
            false => ProgramSpecified,
        };

        let size_hints = WmSizeHints {
            size: Some((spec, width, height)),
            min_size: (!resizable).then_some((width, height)),
            max_size: (!resizable).then_some((width, height)),
            ..Default::default()
        };

        size_hints.set(conn, self.x11_id, AtomEnum::WM_NORMAL_HINTS)?;

        Ok(())
    }
}

/// An X11 application.
#[allow(unused)]
pub struct X11App<T> {
    data: T,
    app: App<T>,
    conn: Arc<XCBConnection>,
    atoms: Atoms,
    running: bool,
    screen: usize,
    event_rx: Receiver<Option<XEvent>>,
    event_tx: Sender<Option<XEvent>>,
    thread: JoinHandle<()>,
    windows: Vec<X11Window>,

    egl_context: EglContext,
    xkb_context: xkb::Context,
    core_keyboard: XkbKeyboard,
}

impl<T> X11App<T> {
    /// Create a new X11 application.
    pub fn new(app: AppBuilder<T>, data: T) -> Result<Self, X11Error> {
        let (conn, screen_num) = XCBConnection::connect(None)?;
        let conn = Arc::new(conn);

        Self::init_xkb(&conn)?;

        let atoms = Atoms::new(&conn)?.reply()?;

        let egl_context = EglContext::new();

        let (event_tx, event_rx) = std::sync::mpsc::channel();

        let thread = thread::spawn({
            let conn = conn.clone();
            let tx = event_tx.clone();

            move || loop {
                let event = conn.wait_for_event().unwrap();
                tx.send(Some(event)).unwrap();
            }
        });

        let waker = CommandWaker::new({
            let tx = event_tx.clone();

            move || {
                tx.send(None).unwrap();
            }
        });

        let xkb_context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
        let core_keyboard = XkbKeyboard::x11_new_core(&conn, &xkb_context);

        Ok(Self {
            data,
            app: app.build(waker),
            conn,
            atoms,
            running: true,
            screen: screen_num,
            event_rx,
            event_tx,
            thread,
            windows: Vec::new(),

            egl_context,
            xkb_context,
            core_keyboard,
        })
    }

    /// Run the application.
    pub fn run(mut self) -> Result<(), X11Error> {
        self.running = true;

        self.app.init(&mut self.data);
        self.handle_app_requests()?;

        while self.running {
            self.conn.flush()?;
            let mut event_option = if self.needs_redraw() {
                self.event_rx.try_recv().ok()
            } else {
                Some(self.event_rx.recv().unwrap())
            };

            while let Some(event) = event_option {
                match event {
                    Some(event) => self.handle_event(event)?,
                    None => self.handle_commands()?,
                }

                event_option = self.event_rx.try_recv().ok();
            }

            self.render_windows()?;
            self.handle_app_requests()?;
        }

        Ok(())
    }
}

impl<T> X11App<T> {
    fn get_window_ori(&self, id: WindowId) -> Option<usize> {
        self.windows.iter().position(|w| w.ori_id == id)
    }

    fn get_window_x11(&self, id: u32) -> Option<usize> {
        self.windows.iter().position(|w| w.x11_id == id)
    }

    fn needs_redraw(&self) -> bool {
        self.windows.iter().any(|window| window.needs_redraw)
    }

    fn handle_commands(&mut self) -> Result<(), X11Error> {
        self.app.handle_commands(&mut self.data);

        Ok(())
    }

    fn open_window(&mut self, window: Window, ui: UiBuilder<T>) -> Result<(), X11Error> {
        let win_id = self.conn.generate_id()?;
        let colormap_id = self.conn.generate_id()?;

        let screen = &self.conn.setup().roots[self.screen];

        let (depth, visual) = self.choose_visual()?;

        (self.conn).create_colormap(ColormapAlloc::NONE, colormap_id, screen.root, visual)?;

        // we want to enable transparency
        let aux = CreateWindowAux::new()
            .event_mask(
                EventMask::EXPOSURE
                    | EventMask::STRUCTURE_NOTIFY
                    | EventMask::POINTER_MOTION
                    | EventMask::LEAVE_WINDOW
                    | EventMask::BUTTON_PRESS
                    | EventMask::BUTTON_RELEASE
                    | EventMask::KEY_PRESS
                    | EventMask::KEY_RELEASE,
            )
            .background_pixel(0)
            .border_pixel(screen.black_pixel)
            .colormap(colormap_id);

        self.conn.create_window(
            depth,
            win_id,
            screen.root,
            0,
            0,
            window.width() as u16,
            window.height() as u16,
            0,
            WindowClass::INPUT_OUTPUT,
            visual,
            &aux,
        )?;

        if !window.resizable {
            let size_hints = WmSizeHints {
                size: Some((
                    WmSizeHintsSpecification::ProgramSpecified,
                    window.width() as i32,
                    window.height() as i32,
                )),
                min_size: Some((window.width() as i32, window.height() as i32)),
                max_size: Some((window.width() as i32, window.height() as i32)),
                ..Default::default()
            };

            size_hints.set(&self.conn, win_id, AtomEnum::WM_NORMAL_HINTS)?;
        }

        self.conn.change_property8(
            PropMode::REPLACE,
            win_id,
            AtomEnum::WM_NAME,
            AtomEnum::STRING,
            window.title.as_bytes(),
        )?;

        self.conn.change_property8(
            PropMode::REPLACE,
            win_id,
            self.atoms._NET_WM_NAME,
            self.atoms.UTF8_STRING,
            window.title.as_bytes(),
        )?;

        self.conn.change_property32(
            PropMode::REPLACE,
            win_id,
            self.atoms.WM_PROTOCOLS,
            AtomEnum::ATOM,
            &[self.atoms.WM_DELETE_WINDOW],
        )?;

        self.conn.change_property8(
            PropMode::REPLACE,
            win_id,
            AtomEnum::WM_CLASS,
            AtomEnum::STRING,
            b"ori\0",
        )?;

        self.conn.flush()?;

        let egl_surface = EglSurface::new(&self.egl_context, win_id as _);
        egl_surface.make_current();
        egl_surface.swap_interval(1);

        let renderer = unsafe {
            GlowRenderer::new(|name| {
                let name = std::ffi::CString::new(name).unwrap();
                *LIB_GL.get(name.as_bytes_with_nul()).unwrap()
            })
        };

        self.conn.map_window(win_id)?;

        let x11_window = X11Window {
            x11_id: win_id,
            ori_id: window.id(),
            physical_width: window.width(),
            physical_height: window.height(),
            egl_surface,
            renderer,
            needs_redraw: true,
        };
        self.windows.push(x11_window);

        self.app.add_window(&mut self.data, ui, window);

        Ok(())
    }

    fn close_window(&mut self, id: WindowId) -> Result<(), X11Error> {
        if let Some(index) = self.windows.iter().position(|w| w.ori_id == id) {
            let window = self.windows.remove(index);

            self.conn.destroy_window(window.x11_id)?;
            self.app.remove_window(id);
        }

        Ok(())
    }

    fn request_redraw(&mut self, id: WindowId) {
        if let Some(window) = self.get_window_ori(id) {
            self.windows[window].needs_redraw = true;
        }
    }

    fn render_windows(&mut self) -> Result<(), X11Error> {
        for window in &mut self.windows {
            if !window.needs_redraw {
                continue;
            }

            window.needs_redraw = false;

            if let Some(state) = self.app.draw_window(&mut self.data, window.ori_id) {
                unsafe {
                    window.egl_surface.make_current();

                    window.renderer.render(
                        state.canvas,
                        state.clear_color,
                        window.physical_width,
                        window.physical_height,
                    );

                    window.egl_surface.swap_buffers();
                }
            }
        }

        Ok(())
    }

    fn handle_app_requests(&mut self) -> Result<(), X11Error> {
        for request in self.app.take_requests() {
            self.handle_app_request(request)?;
        }

        Ok(())
    }

    fn handle_app_request(&mut self, request: AppRequest<T>) -> Result<(), X11Error> {
        match request {
            AppRequest::OpenWindow(window, ui) => self.open_window(window, ui)?,
            AppRequest::CloseWindow(id) => self.close_window(id)?,
            AppRequest::DragWindow(_) => {}
            AppRequest::RequestRedraw(id) => self.request_redraw(id),
            AppRequest::UpdateWindow(id, update) => {
                let Some(index) = self.windows.iter().position(|w| w.ori_id == id) else {
                    return Ok(());
                };
                let window = &mut self.windows[index];

                match update {
                    WindowUpdate::Title(title) => {
                        self.conn.change_property8(
                            PropMode::REPLACE,
                            window.x11_id,
                            AtomEnum::WM_NAME,
                            AtomEnum::STRING,
                            title.as_bytes(),
                        )?;

                        self.conn.change_property8(
                            PropMode::REPLACE,
                            window.x11_id,
                            self.atoms._NET_WM_NAME,
                            self.atoms.UTF8_STRING,
                            title.as_bytes(),
                        )?;
                    }
                    WindowUpdate::Icon(_) => {}
                    WindowUpdate::Size(size) => {
                        let width = size.width as u32;
                        let height = size.height as u32;

                        let aux = ConfigureWindowAux::new().width(width).height(height);

                        window.physical_width = width;
                        window.physical_height = height;

                        self.conn.configure_window(window.x11_id, &aux)?;

                        let resizable = self.app.get_window(id).map_or(false, |w| w.resizable);
                        window.set_size_hints(
                            &self.conn,
                            width as i32,
                            height as i32,
                            resizable,
                        )?;
                    }
                    WindowUpdate::Scale(_) => {}
                    WindowUpdate::Resizable(resizable) => {
                        window.set_size_hints(
                            &self.conn,
                            window.physical_width as i32,
                            window.physical_height as i32,
                            resizable,
                        )?;
                    }
                    WindowUpdate::Decorated(_) => {}
                    WindowUpdate::Maximized(_) => {}
                    WindowUpdate::Visible(visible) => {
                        if visible {
                            self.conn.map_window(window.x11_id)?;
                        } else {
                            self.conn.unmap_window(window.x11_id)?;
                        }
                    }
                    WindowUpdate::Color(_) => {}
                    WindowUpdate::Cursor(_) => {}
                }
            }
            AppRequest::Quit => self.running = false,
        }

        Ok(())
    }

    fn handle_event(&mut self, event: XEvent) -> Result<(), X11Error> {
        match event {
            XEvent::Expose(event) => {
                if let Some(index) = self.get_window_x11(event.window) {
                    self.windows[index].needs_redraw = true;
                }
            }
            XEvent::ConfigureNotify(event) => {
                let width = event.width as u32;
                let height = event.height as u32;

                if let Some(index) = self.get_window_x11(event.window) {
                    let window = &mut self.windows[index];

                    if window.physical_width != width || window.physical_height != height {
                        window.physical_width = width;
                        window.physical_height = height;

                        let id = window.ori_id;
                        self.app.window_resized(&mut self.data, id, width, height);
                        self.request_redraw(id);
                    }
                }
            }
            XEvent::ClientMessage(event) => {
                if let Some(index) = self.get_window_x11(event.window) {
                    let window = &self.windows[index];
                    self.app.close_requested(&mut self.data, window.ori_id);
                }
            }
            XEvent::MotionNotify(event) => {
                if let Some(index) = self.get_window_x11(event.event) {
                    let pointer_id = PointerId::from_hash(&event.child);

                    let id = self.windows[index].ori_id;
                    self.app.pointer_moved(
                        &mut self.data,
                        id,
                        pointer_id,
                        Point::new(event.event_x as f32, event.event_y as f32),
                    );
                }
            }
            XEvent::LeaveNotify(event) => {
                if let Some(index) = self.get_window_x11(event.event) {
                    let pointer_id = PointerId::from_hash(&event.child);

                    let id = self.windows[index].ori_id;
                    self.app.pointer_left(&mut self.data, id, pointer_id);
                }
            }
            XEvent::ButtonPress(event) => {
                if let Some(index) = self.get_window_x11(event.event) {
                    self.pointer_button(self.windows[index].ori_id, event.detail, true);
                }
            }
            XEvent::ButtonRelease(event) => {
                if let Some(index) = self.get_window_x11(event.event) {
                    self.pointer_button(self.windows[index].ori_id, event.detail, false);
                }
            }
            XEvent::XkbStateNotify(event) => {
                if event.device_id as i32 != self.core_keyboard.device_id() {
                    return Ok(());
                }

                self.core_keyboard.state.update_mask(
                    event.base_mods.into(),
                    event.latched_mods.into(),
                    event.locked_mods.into(),
                    event.base_group as _,
                    event.latched_group as _,
                    event.locked_group.into(),
                );

                let modifiers = Modifiers {
                    shift: event.mods.contains(ModMask::SHIFT),
                    ctrl: event.mods.contains(ModMask::CONTROL),
                    alt: event.mods.contains(ModMask::M1),
                    meta: event.mods.contains(ModMask::M4),
                };

                self.app.modifiers_changed(modifiers);
            }
            XEvent::KeyPress(event) => {
                if let Some(index) = self.get_window_x11(event.event) {
                    let utf8 = self.core_keyboard.key_get_utf8(event.detail.into());
                    let code = Code::from_linux_scancode(event.detail - 8);
                    let text = (!utf8.is_empty()).then_some(utf8);

                    let id = self.windows[index].ori_id;
                    self.app.keyboard_key(&mut self.data, id, code, text, true);
                }
            }
            XEvent::KeyRelease(event) => {
                if let Some(index) = self.get_window_x11(event.event) {
                    let code = Code::from_linux_scancode(event.detail - 8);

                    let id = self.windows[index].ori_id;
                    self.app.keyboard_key(&mut self.data, id, code, None, false);
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn pointer_button(&mut self, id: WindowId, code: u8, pressed: bool) {
        let pointer_id = PointerId::from_hash(&0);

        match code {
            4..=7 => {
                let delta = match code {
                    4 => Vector::Y,
                    5 => Vector::NEG_Y,
                    6 => Vector::X,
                    7 => Vector::NEG_X,
                    _ => unreachable!(),
                };

                (self.app).pointer_scrolled(&mut self.data, id, pointer_id, delta);
            }
            _ => {
                let button = PointerButton::from_u16(code as u16);

                (self.app).pointer_button(&mut self.data, id, pointer_id, button, pressed);
            }
        }
    }

    fn choose_visual(&self) -> Result<(u8, Visualid), X11Error> {
        let screen = &self.conn.setup().roots[self.screen];

        let formats = self.conn.render_query_pict_formats()?.reply()?;

        for format in formats.formats {
            if format.type_ == PictType::DIRECT {
                continue;
            }

            if format.direct.red_mask != 0xff
                || format.direct.green_mask != 0xff
                || format.direct.blue_mask != 0xff
                || format.direct.alpha_mask != 0xff
            {
                continue;
            }

            if format.direct.red_shift != 16
                || format.direct.green_shift != 8
                || format.direct.blue_shift != 0
                || format.direct.alpha_shift != 24
            {
                continue;
            }

            for depth in &formats.screens[self.screen].depths {
                for visual in &depth.visuals {
                    if visual.format == format.id {
                        return Ok((depth.depth, visual.visual));
                    }
                }
            }
        }

        Ok((screen.root_depth, screen.root_visual))
    }

    fn init_xkb(conn: &XCBConnection) -> Result<(), X11Error> {
        conn.xkb_use_extension(1, 0)?;

        let events = XkbEventType::MAP_NOTIFY | XkbEventType::STATE_NOTIFY;
        let map_parts = XkbMapPart::MODIFIER_MAP;
        conn.xkb_select_events(
            XkbID::USE_CORE_KBD.into(),
            XkbEventType::from(0u8),
            events,
            map_parts,
            map_parts,
            &XkbSelectEventsAux::new(),
        )?;

        Ok(())
    }
}
