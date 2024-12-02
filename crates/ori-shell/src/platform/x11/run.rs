use std::{
    collections::{hash_map::Entry, HashMap},
    sync::{
        mpsc::{Receiver, RecvTimeoutError, Sender},
        Arc,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use as_raw_xcb_connection::AsRawXcbConnection;
use ori_app::{App, AppBuilder, AppRequest, UiBuilder};
use ori_core::{
    clipboard::Clipboard,
    command::CommandWaker,
    event::{Code, Modifiers, PointerButton, PointerId},
    image::Image,
    layout::{Point, Vector},
    text::Fonts,
    window::{Cursor, Window, WindowId, WindowUpdate},
};
use ori_skia::{SkiaFonts, SkiaRenderer};

use tracing::warn;
use x11rb::{
    atom_manager,
    connection::{Connection, RequestConnection},
    cursor::Handle as CursorHandle,
    properties::WmSizeHints,
    protocol::{
        render::{ConnectionExt as _, PictType},
        sync::{ConnectionExt as _, Int64},
        xkb::{
            ConnectionExt as _, EventType as XkbEventType, MapPart as XkbMapPart,
            SelectEventsAux as XkbSelectEventsAux, ID as XkbID,
        },
        xproto::{
            AtomEnum, ChangeWindowAttributesAux, ClientMessageData, ClientMessageEvent,
            ColormapAlloc, ConfigureWindowAux, ConnectionExt as _, CreateWindowAux,
            Cursor as XCursor, EventMask, ModMask, PropMode, VisualClass, Visualid, WindowClass,
            CLIENT_MESSAGE_EVENT,
        },
        Event as XEvent,
    },
    resource_manager::Database,
    wrapper::ConnectionExt as _,
    x11_utils::Serialize,
    xcb_ffi::XCBConnection,
};

use crate::platform::{
    egl::{EglContext, EglNativeDisplay, EglSurface},
    linux::xkb::{XkbContext, XkbKeyboard},
};

use super::{clipboard::X11ClipboardServer, X11Error};

/// Options for running an X11 application.
#[derive(Debug, Default)]
pub struct X11RunOptions {
    window_parents: HashMap<WindowId, u32>,
}

impl X11RunOptions {
    /// Create a new set of X11 run options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the X11 window parent for a window.
    pub fn with_window_parent(mut self, id: WindowId, x11_id: u32) -> Self {
        self.window_parents.insert(id, x11_id);
        self
    }
}

atom_manager! {
    pub Atoms: AtomsCookie {
        TARGETS,
        XSEL_DATA,
        CLIPBOARD,
        UTF8_STRING,
        WM_PROTOCOLS,
        WM_DELETE_WINDOW,
        _MOTIF_WM_HINTS,
        _NET_WM_NAME,
        _NET_WM_ICON,
        _NET_WM_SYNC_REQUEST,
        _NET_WM_SYNC_REQUEST_COUNTER,
        _NET_WM_ALLOWED_ACTIONS,
        _NET_WM_ACTION_MOVE,
        _NET_WM_ACTION_RESIZE,
        _NET_WM_STATE,
        _NET_WM_STATE_MAXIMIZED_VERT,
        _NET_WM_STATE_MAXIMIZED_HORZ,
        _NET_WM_WINDOW_TYPE,
        _NET_WM_WINDOW_TYPE_NORMAL,
        _NET_WM_WINDOW_TYPE_DIALOG,
        _NET_WM_WINDOW_TYPE_DOCK,
    }
}

struct X11Window {
    x11_id: u32,
    ori_id: WindowId,
    physical_width: u32,
    physical_height: u32,
    scale_factor: f32,
    egl_surface: EglSurface,
    renderer: SkiaRenderer,
    needs_redraw: bool,
    sync_counter: Option<u32>,
}

impl X11Window {
    fn set_title(
        window: u32,
        conn: &XCBConnection,
        atoms: &Atoms,
        title: &str,
    ) -> Result<(), X11Error> {
        conn.change_property8(
            PropMode::REPLACE,
            window,
            AtomEnum::WM_NAME,
            AtomEnum::STRING,
            title.as_bytes(),
        )?;

        conn.change_property8(
            PropMode::REPLACE,
            window,
            atoms._NET_WM_NAME,
            atoms.UTF8_STRING,
            title.as_bytes(),
        )?;

        Ok(())
    }

    fn set_size_hints(
        window: u32,
        conn: &XCBConnection,
        width: i32,
        height: i32,
        resizable: bool,
    ) -> Result<(), X11Error> {
        let size_hints = WmSizeHints {
            min_size: (!resizable).then_some((width, height)),
            max_size: (!resizable).then_some((width, height)),
            ..Default::default()
        };

        size_hints.set_normal_hints(conn, window)?;
        conn.flush()?;

        Ok(())
    }

    fn set_icon(
        window: u32,
        conn: &XCBConnection,
        atoms: &Atoms,
        image: &Image,
    ) -> Result<(), X11Error> {
        let mut data = Vec::with_capacity(image.width() as usize * image.height() as usize + 2);
        data.push(image.width());
        data.push(image.height());

        for pixel in image.chunks_exact(4) {
            let r = pixel[0];
            let g = pixel[1];
            let b = pixel[2];
            let a = pixel[3];

            let pixel = u32::from_ne_bytes([r, g, b, a]);
            data.push(pixel);
        }

        conn.change_property32(
            PropMode::REPLACE,
            window,
            atoms._NET_WM_ICON,
            AtomEnum::CARDINAL,
            &data,
        )?
        .check()?;

        Ok(())
    }

    fn unset_icon(window: u32, conn: &XCBConnection, atoms: &Atoms) -> Result<(), X11Error> {
        conn.delete_property(window, atoms._NET_WM_ICON)?;

        Ok(())
    }

    fn get_allowed_actions(
        window: u32,
        conn: &XCBConnection,
        atoms: &Atoms,
    ) -> Result<Vec<u32>, X11Error> {
        let reply = conn.get_property(
            false,
            window,
            atoms._NET_WM_ALLOWED_ACTIONS,
            AtomEnum::ATOM,
            0,
            u32::MAX,
        )?;

        Ok(reply.reply()?.value32().into_iter().flatten().collect())
    }

    fn set_allowed_actions(
        window: u32,
        conn: &XCBConnection,
        atoms: &Atoms,
        actions: &[u32],
    ) -> Result<(), X11Error> {
        conn.change_property32(
            PropMode::REPLACE,
            window,
            atoms._NET_WM_ALLOWED_ACTIONS,
            AtomEnum::ATOM,
            actions,
        )?;

        Ok(())
    }

    fn set_resizable(
        window: u32,
        conn: &XCBConnection,
        atoms: &Atoms,
        resizable: bool,
    ) -> Result<(), X11Error> {
        let mut actions = Self::get_allowed_actions(window, conn, atoms)?;

        if resizable {
            actions.push(atoms._NET_WM_ACTION_MOVE);
            actions.push(atoms._NET_WM_ACTION_RESIZE);
        } else {
            actions.retain(|&action| {
                action != atoms._NET_WM_ACTION_MOVE && action != atoms._NET_WM_ACTION_RESIZE
            });
        }

        Self::set_allowed_actions(window, conn, atoms, &actions)?;

        Ok(())
    }

    fn get_motif_hints(
        window: u32,
        conn: &XCBConnection,
        atoms: &Atoms,
    ) -> Result<Vec<u32>, X11Error> {
        let reply = conn.get_property(
            false,
            window,
            atoms._MOTIF_WM_HINTS,
            AtomEnum::ATOM,
            0,
            u32::MAX,
        )?;

        let hints = reply
            .reply()?
            .value32()
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        Ok(hints)
    }

    fn set_motif_hints(
        window: u32,
        conn: &XCBConnection,
        atoms: &Atoms,
        hints: &[u32],
    ) -> Result<(), X11Error> {
        conn.change_property32(
            PropMode::REPLACE,
            window,
            atoms._MOTIF_WM_HINTS,
            AtomEnum::ATOM,
            hints,
        )?
        .check()?;

        Ok(())
    }

    fn set_decorated(
        window: u32,
        conn: &XCBConnection,
        atoms: &Atoms,
        decorated: bool,
    ) -> Result<(), X11Error> {
        let mut hints = Self::get_motif_hints(window, conn, atoms)?;
        hints.resize(5, 0);

        hints[0] |= 1 << 1;

        // magic numbers go brrr
        hints[2] = if decorated { 1 } else { 0 };

        Self::set_motif_hints(window, conn, atoms, &hints)?;

        Ok(())
    }

    fn set_maximized(
        window: u32,
        screen: usize,
        conn: &XCBConnection,
        atoms: &Atoms,
        maximized: bool,
    ) -> Result<(), X11Error> {
        let mut data = [0u32; 5];

        data[0] = maximized as u32;
        data[1] = atoms._NET_WM_STATE_MAXIMIZED_HORZ;
        data[2] = atoms._NET_WM_STATE_MAXIMIZED_VERT;

        let screen = conn.setup().roots[screen].root;

        conn.send_event(
            false,
            screen,
            EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY,
            ClientMessageEvent {
                response_type: CLIENT_MESSAGE_EVENT,
                format: 32,
                sequence: 0,
                window,
                type_: atoms._NET_WM_STATE,
                data: ClientMessageData::from(data),
            }
            .serialize(),
        )?
        .check()?;
        conn.flush()?;

        Ok(())
    }

    fn is_maximized(window: u32, conn: &XCBConnection, atoms: &Atoms) -> Result<bool, X11Error> {
        let reply = conn.get_property(
            false,
            window,
            atoms._NET_WM_STATE,
            AtomEnum::ATOM,
            0,
            u32::MAX,
        )?;

        let states = reply
            .reply()?
            .value32()
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        Ok(states.contains(&atoms._NET_WM_STATE_MAXIMIZED_HORZ)
            && states.contains(&atoms._NET_WM_STATE_MAXIMIZED_VERT))
    }
}

/// Create a new X11 application.
pub fn run<T>(app: AppBuilder<T>, data: &mut T, options: X11RunOptions) -> Result<(), X11Error> {
    let (conn, screen_num) = XCBConnection::connect(None)?;
    let conn = Arc::new(conn);

    X11App::<T>::init_xkb(&conn)?;

    let atoms = Atoms::new(&conn)?.reply()?;
    let (clipboard_server, clipboard) = X11ClipboardServer::new(&conn, atoms)?;

    let egl_context = EglContext::new(EglNativeDisplay::X11)?;

    let (event_tx, event_rx) = std::sync::mpsc::channel();

    let thread = thread::spawn({
        let conn = conn.clone();
        let tx = event_tx.clone();

        move || loop {
            let event = conn.wait_for_event().unwrap();
            clipboard_server.handle_event(&conn, &event).unwrap();

            if tx.send(Some(event)).is_err() {
                break;
            }
        }
    });

    let waker = CommandWaker::new({
        let tx = event_tx.clone();

        move || {
            tx.send(None).unwrap();
        }
    });

    let reply = conn
        .get_property(
            Database::GET_RESOURCE_DATABASE.delete,
            conn.setup().roots[screen_num].root,
            Database::GET_RESOURCE_DATABASE.property,
            Database::GET_RESOURCE_DATABASE.type_,
            Database::GET_RESOURCE_DATABASE.long_offset,
            Database::GET_RESOURCE_DATABASE.long_length,
        )?
        .reply()?;

    let hostname = std::env::var_os("HOSTNAME").unwrap_or_default();
    let database = Database::new_from_default(&reply, hostname);
    let cursor_handle = CursorHandle::new(&conn, screen_num, &database)?.reply()?;

    let xcb_conn = conn.as_raw_xcb_connection() as *mut _;
    let xkb_context = unsafe { XkbContext::from_xcb(xcb_conn).unwrap() };
    let core_keyboard = unsafe { XkbKeyboard::new_xcb(&xkb_context, xcb_conn).unwrap() };

    let fonts = Box::new(SkiaFonts::new(Some("Roboto")));

    let mut app = app.build(waker, fonts);
    app.add_context(Clipboard::new(Box::new(clipboard)));

    let mut state = X11App {
        options,
        app,
        conn,
        atoms,
        running: true,
        screen: screen_num,
        event_rx,
        event_tx,
        thread,
        windows: Vec::new(),
        database,
        cursor_handle,
        cursors: HashMap::new(),

        egl_context,
        xkb_context,
        core_keyboard,
    };

    state.app.init(data);
    state.handle_app_requests(data)?;

    while state.running {
        state.conn.flush()?;

        let mut event_option = if state.needs_redraw() {
            state.event_rx.try_recv().ok()
        } else {
            match state.event_rx.recv_timeout(Duration::from_millis(2)) {
                Ok(event) => Some(event),
                Err(err) => match err {
                    RecvTimeoutError::Timeout => None,
                    RecvTimeoutError::Disconnected => break,
                },
            }
        };

        while let Some(event) = event_option {
            match event {
                Some(event) => state.handle_event(data, event)?,
                None => state.handle_commands(data)?,
            }

            state.handle_app_requests(data)?;
            event_option = state.event_rx.try_recv().ok();
        }

        state.render_windows(data)?;
        state.handle_app_requests(data)?;

        state.app.idle(data);
        state.handle_app_requests(data)?;
    }

    Ok(())
}

#[allow(unused)]
struct X11App<T> {
    options: X11RunOptions,
    app: App<T>,
    conn: Arc<XCBConnection>,
    atoms: Atoms,
    running: bool,
    screen: usize,
    event_rx: Receiver<Option<XEvent>>,
    event_tx: Sender<Option<XEvent>>,
    thread: JoinHandle<()>,
    windows: Vec<X11Window>,
    database: Database,
    cursor_handle: CursorHandle,
    cursors: HashMap<Cursor, XCursor>,

    egl_context: EglContext,
    xkb_context: XkbContext,
    core_keyboard: XkbKeyboard,
}

impl<T> X11App<T> {
    fn get_window_ori(&self, id: WindowId) -> Option<usize> {
        self.windows.iter().position(|w| w.ori_id == id)
    }

    fn get_window_x11(&self, id: u32) -> Option<usize> {
        self.windows.iter().position(|w| w.x11_id == id)
    }

    fn needs_redraw(&self) -> bool {
        self.windows.iter().any(|w| w.needs_redraw)
    }

    fn handle_commands(&mut self, data: &mut T) -> Result<(), X11Error> {
        self.app.handle_commands(data);

        Ok(())
    }

    fn open_window(
        &mut self,
        data: &mut T,
        window: Window,
        ui: UiBuilder<T>,
    ) -> Result<(), X11Error> {
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

        let scale_factor = 1.0;
        let physical_width = (window.size.width * scale_factor) as u32;
        let physical_height = (window.size.height * scale_factor) as u32;

        let parent = match self.options.window_parents.get(&window.id()) {
            Some(&parent) => parent,
            None => screen.root,
        };

        self.conn.create_window(
            depth,
            win_id,
            parent,
            0,
            0,
            physical_width as u16,
            physical_height as u16,
            0,
            WindowClass::INPUT_OUTPUT,
            visual,
            &aux,
        )?;

        self.conn.change_property32(
            PropMode::REPLACE,
            win_id,
            self.atoms.WM_PROTOCOLS,
            AtomEnum::ATOM,
            &[self.atoms.WM_DELETE_WINDOW, self.atoms._NET_WM_SYNC_REQUEST],
        )?;

        self.conn.change_property8(
            PropMode::REPLACE,
            win_id,
            AtomEnum::WM_CLASS,
            AtomEnum::STRING,
            b"ori\0",
        )?;

        let sync_counter = if self
            .conn
            .extension_information(x11rb::protocol::sync::X11_EXTENSION_NAME)
            .is_ok()
        {
            let counter = self.conn.generate_id()?;

            self.conn.sync_create_counter(counter, Int64::default())?;

            self.conn.change_property32(
                PropMode::REPLACE,
                win_id,
                self.atoms._NET_WM_SYNC_REQUEST_COUNTER,
                AtomEnum::CARDINAL,
                &[counter],
            )?;

            Some(counter)
        } else {
            None
        };

        X11Window::set_title(win_id, &self.conn, &self.atoms, &window.title)?;
        X11Window::set_decorated(win_id, &self.conn, &self.atoms, window.decorated)?;

        if !window.resizable {
            X11Window::set_resizable(win_id, &self.conn, &self.atoms, window.resizable)?;
            X11Window::set_size_hints(
                win_id,
                &self.conn,
                physical_width as i32,
                physical_height as i32,
                window.resizable,
            )?;
        }

        if let Some(ref icon) = window.icon {
            X11Window::set_icon(win_id, &self.conn, &self.atoms, icon)?;
        }

        self.conn.flush()?;

        let egl_surface = EglSurface::new(&self.egl_context, win_id as _)?;
        egl_surface.make_current()?;
        egl_surface.swap_interval(0)?;

        let renderer = unsafe {
            SkiaRenderer::new(|name| {
                //
                self.egl_context.get_proc_address(name)
            })
        };

        let x11_window = X11Window {
            x11_id: win_id,
            ori_id: window.id(),
            physical_width,
            physical_height,
            scale_factor,
            egl_surface,
            renderer,
            needs_redraw: true,
            sync_counter,
        };

        if window.visible {
            self.conn.map_window(win_id)?;
        }

        self.conn.flush()?;

        self.windows.push(x11_window);
        self.app.add_window(data, ui, window);

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

    fn render_windows(&mut self, data: &mut T) -> Result<(), X11Error> {
        for window in &mut self.windows {
            if !window.needs_redraw {
                continue;
            }

            window.needs_redraw = false;

            if let Some(state) = self.app.draw_window(data, window.ori_id) {
                window.egl_surface.make_current()?;

                let fonts = self.app.contexts.get_mut::<Box<dyn Fonts>>().unwrap();

                window.renderer.render(
                    fonts.downcast_mut().unwrap(),
                    &state.canvas,
                    state.clear_color,
                    window.physical_width,
                    window.physical_height,
                    window.scale_factor,
                );

                window.egl_surface.swap_buffers()?;
            }
        }

        Ok(())
    }

    fn handle_app_requests(&mut self, data: &mut T) -> Result<(), X11Error> {
        for request in self.app.take_requests() {
            self.handle_app_request(data, request)?;
        }

        Ok(())
    }

    fn set_cursor(&mut self, x_window: u32, cursor: Cursor) -> Result<(), X11Error> {
        let cursor = match self.cursors.entry(cursor) {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => {
                let cursor = self.cursor_handle.load_cursor(&self.conn, cursor.name())?;
                *entry.insert(cursor)
            }
        };

        let aux = ChangeWindowAttributesAux::new().cursor(cursor);
        self.conn.change_window_attributes(x_window, &aux)?;

        Ok(())
    }

    fn handle_app_request(&mut self, data: &mut T, request: AppRequest<T>) -> Result<(), X11Error> {
        match request {
            AppRequest::OpenWindow(window, ui) => self.open_window(data, window, ui)?,
            AppRequest::CloseWindow(id) => self.close_window(id)?,
            AppRequest::DragWindow(_id) => {
                warn!("DragWindow is not supported on X11");
            }
            AppRequest::RequestRedraw(id) => self.request_redraw(id),
            AppRequest::UpdateWindow(id, update) => {
                let Some(index) = self.windows.iter().position(|w| w.ori_id == id) else {
                    return Ok(());
                };
                let window = &mut self.windows[index];

                match update {
                    WindowUpdate::Title(title) => {
                        X11Window::set_title(window.x11_id, &self.conn, &self.atoms, &title)?;
                    }
                    WindowUpdate::Icon(icon) => match icon {
                        Some(icon) => {
                            X11Window::set_icon(window.x11_id, &self.conn, &self.atoms, &icon)?;
                        }
                        None => {
                            X11Window::unset_icon(window.x11_id, &self.conn, &self.atoms)?;
                        }
                    },
                    WindowUpdate::Size(size) => {
                        let physical_width = (size.width * window.scale_factor) as u32;
                        let physical_height = (size.height * window.scale_factor) as u32;

                        let resizable = self.app.get_window(id).map_or(true, |w| w.resizable);
                        X11Window::set_size_hints(
                            window.x11_id,
                            &self.conn,
                            physical_width as i32,
                            physical_height as i32,
                            resizable,
                        )?;

                        let aux = ConfigureWindowAux::new()
                            .width(physical_width)
                            .height(physical_height);

                        window.physical_width = physical_width;
                        window.physical_height = physical_height;

                        self.conn.configure_window(window.x11_id, &aux)?;
                    }
                    WindowUpdate::Scale(_) => {}
                    WindowUpdate::Resizable(resizable) => {
                        X11Window::set_resizable(
                            window.x11_id,
                            &self.conn,
                            &self.atoms,
                            resizable,
                        )?;
                        X11Window::set_size_hints(
                            window.x11_id,
                            &self.conn,
                            window.physical_width as i32,
                            window.physical_height as i32,
                            resizable,
                        )?;
                    }
                    WindowUpdate::Decorated(decorated) => {
                        X11Window::set_decorated(
                            window.x11_id,
                            &self.conn,
                            &self.atoms,
                            decorated,
                        )?;
                    }
                    WindowUpdate::Maximized(maximized) => {
                        X11Window::set_maximized(
                            window.x11_id,
                            self.screen,
                            &self.conn,
                            &self.atoms,
                            maximized,
                        )?;
                    }
                    WindowUpdate::Visible(visible) => {
                        if visible {
                            self.conn.map_window(window.x11_id)?;
                        } else {
                            self.conn.unmap_window(window.x11_id)?;
                        }
                    }
                    WindowUpdate::Color(_) => {
                        self.request_redraw(id);
                    }
                    WindowUpdate::Cursor(cursor) => {
                        let x_window = window.x11_id;
                        self.set_cursor(x_window, cursor)?;
                    }
                    WindowUpdate::Ime(_) => {}
                }
            }
            AppRequest::Quit => self.running = false,
        }

        Ok(())
    }

    fn handle_event(&mut self, data: &mut T, event: XEvent) -> Result<(), X11Error> {
        match event {
            XEvent::Expose(event) => {
                if let Some(index) = self.get_window_x11(event.window) {
                    self.windows[index].needs_redraw = true;
                }
            }
            XEvent::ConfigureNotify(event) => {
                let physical_width = event.width as u32;
                let physical_height = event.height as u32;

                if let Some(index) = self.get_window_x11(event.window) {
                    let window = &mut self.windows[index];

                    let logical_width = (physical_width as f32 / window.scale_factor) as u32;
                    let logical_height = (physical_height as f32 / window.scale_factor) as u32;

                    if window.physical_width != physical_width
                        || window.physical_height != physical_height
                    {
                        window.physical_width = physical_width;
                        window.physical_height = physical_height;

                        if let Some(app_window) = self.app.get_window_mut(window.ori_id) {
                            app_window.maximized = X11Window::is_maximized(
                                //
                                window.x11_id,
                                &self.conn,
                                &self.atoms,
                            )?
                        }

                        let id = window.ori_id;
                        (self.app).window_resized(data, id, logical_width, logical_height);
                        window.needs_redraw = true;
                    }
                }
            }
            XEvent::ClientMessage(event) => {
                if event.data.as_data32()[0] == self.atoms.WM_DELETE_WINDOW {
                    let Some(index) = self.get_window_x11(event.window) else {
                        return Ok(());
                    };

                    let window = &self.windows[index];
                    self.app.close_requested(data, window.ori_id);
                }

                if event.data.as_data32()[0] == self.atoms._NET_WM_SYNC_REQUEST {
                    let Some(index) = self.get_window_x11(event.window) else {
                        return Ok(());
                    };

                    let window = &mut self.windows[index];

                    let Some(counter) = window.sync_counter else {
                        return Ok(());
                    };

                    let lo = event.data.as_data32()[1];
                    let hi = i32::from_ne_bytes(event.data.as_data32()[2].to_ne_bytes());

                    self.conn.sync_set_counter(counter, Int64 { hi, lo })?;
                    window.needs_redraw = true;
                }
            }
            XEvent::MotionNotify(event) => {
                let position = Point::new(event.event_x as f32, event.event_y as f32);

                if let Some(index) = self.get_window_x11(event.event) {
                    let pointer_id = PointerId::from_hash(&event.child);

                    let window = &self.windows[index];
                    let id = window.ori_id;
                    self.app
                        .pointer_moved(data, id, pointer_id, position / window.scale_factor);
                }
            }
            XEvent::LeaveNotify(event) => {
                if let Some(index) = self.get_window_x11(event.event) {
                    let pointer_id = PointerId::from_hash(&event.child);

                    let id = self.windows[index].ori_id;
                    self.app.pointer_left(data, id, pointer_id);
                }
            }
            XEvent::ButtonPress(event) => {
                if let Some(index) = self.get_window_x11(event.event) {
                    self.pointer_button(data, self.windows[index].ori_id, event.detail, true);
                }
            }
            XEvent::ButtonRelease(event) => {
                if let Some(index) = self.get_window_x11(event.event) {
                    self.pointer_button(data, self.windows[index].ori_id, event.detail, false);
                }
            }
            XEvent::XkbStateNotify(event) => {
                let state = self.core_keyboard.state().unwrap();

                state.update_modifiers(
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
                    let keymap = self.core_keyboard.keymap().unwrap();
                    let state = self.core_keyboard.state().unwrap();

                    let layout = state.layout();
                    let code = Code::from_linux_scancode(event.detail - 8);
                    let keysym_raw = keymap.first_keysym(layout, event.detail as _).unwrap();
                    let keysym = state.get_one_sym(event.detail as _);

                    let key = self.core_keyboard.keysym_to_key(keysym_raw);
                    let text = self.core_keyboard.keysym_to_utf8(keysym);

                    let id = self.windows[index].ori_id;
                    (self.app).keyboard_key(data, id, key, code, text, true);
                }
            }
            XEvent::KeyRelease(event) => {
                if let Some(index) = self.get_window_x11(event.event) {
                    let keymap = self.core_keyboard.keymap().unwrap();
                    let state = self.core_keyboard.state().unwrap();

                    let layout = state.layout();
                    let code = Code::from_linux_scancode(event.detail - 8);
                    let keysym_raw = keymap.first_keysym(layout, event.detail as _).unwrap();
                    let keysym = state.get_one_sym(event.detail as _);

                    let key = self.core_keyboard.keysym_to_key(keysym_raw);
                    let text = self.core_keyboard.keysym_to_utf8(keysym);

                    let id = self.windows[index].ori_id;
                    (self.app).keyboard_key(data, id, key, code, text, false);
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn pointer_button(&mut self, data: &mut T, id: WindowId, code: u8, pressed: bool) {
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

                (self.app).pointer_scrolled(data, id, pointer_id, delta);
            }
            _ => {
                let button = PointerButton::from_u16(code as u16);

                (self.app).pointer_button(data, id, pointer_id, button, pressed);
            }
        }
    }

    /// Choose a direct bgra8888 visual with 32-bit depth.
    fn choose_visual(&self) -> Result<(u8, Visualid), X11Error> {
        let screen = &self.conn.setup().roots[self.screen];

        let formats = self.conn.render_query_pict_formats()?.reply()?;

        for format in formats.formats {
            if format.type_ != PictType::DIRECT {
                continue;
            }

            if format.depth != 32 {
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
                    if visual.format != format.id {
                        continue;
                    }

                    for allowed in &screen.allowed_depths {
                        if allowed.depth != depth.depth {
                            continue;
                        }

                        for allowed_visual in &allowed.visuals {
                            if allowed_visual.visual_id != visual.visual {
                                continue;
                            }

                            if allowed_visual.class != VisualClass::TRUE_COLOR {
                                continue;
                            }

                            return Ok((depth.depth, visual.visual));
                        }
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
