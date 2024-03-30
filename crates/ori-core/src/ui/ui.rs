//! User interface state.

use std::collections::HashMap;

use crate::{
    clipboard::{Clipboard, ClipboardContext},
    command::{Command, CommandProxy, CommandReceiver, CommandWaker},
    debug::History,
    delegate::Delegate,
    event::{
        CloseRequested, CloseWindow, Code, Event, KeyPressed, KeyReleased, Modifiers, OpenWindow,
        PointerButton, PointerId, PointerLeft, PointerMoved, PointerPressed, PointerReleased,
        PointerScrolled, Quit, RequestFocus, SwitchFocus,
    },
    layout::{Point, Vector},
    style::{IntoStyle, Style},
    text::Fonts,
    view::{BaseCx, Contexts, DelegateCx},
    window::{Window, WindowId},
};

use super::{UiBuilder, UiRequest, UiRequests, WindowUi};

/// State for running a user interface.
pub struct Ui<T: 'static> {
    windows: HashMap<WindowId, WindowUi<T>>,
    modifiers: Modifiers,
    delegates: Vec<Box<dyn Delegate<T>>>,
    style: Style,
    command_proxy: CommandProxy,
    command_rx: CommandReceiver,
    requests: UiRequests<T>,
    quit_requested: bool,
    /// The contexts used by the UI.
    pub contexts: Contexts,
}

impl<T> Ui<T> {
    /// Create a new [`Ui`].
    pub fn new(waker: CommandWaker) -> Self {
        let (command_proxy, command_rx) = CommandProxy::new(waker);

        let mut contexts = Contexts::new();
        contexts.insert(Fonts::default());
        contexts.insert(History::with_max_items(1000));

        Self {
            windows: HashMap::new(),
            modifiers: Modifiers::default(),
            delegates: Vec::new(),
            style: Style::default(),
            command_proxy,
            command_rx,
            quit_requested: false,
            requests: UiRequests::new(),
            contexts,
        }
    }

    /// Push a [`Delegate`] wrapped in a [`Box`].
    pub fn push_delegate(&mut self, delegate: Box<dyn Delegate<T>>) {
        self.delegates.push(delegate);
    }

    /// Get the delegates.
    pub fn delegates(&self) -> &[Box<dyn Delegate<T>>] {
        &self.delegates
    }

    /// Add a new style.
    pub fn push_style(&mut self, style: impl IntoStyle) {
        self.style.extend(style.into_style());
    }

    /// Set the clipboard provider.
    pub fn set_clipboard(&mut self, provider: impl Clipboard + 'static) {
        self.contexts.insert(ClipboardContext::new(provider));
    }

    /// Add a new window.
    pub fn add_window(&mut self, data: &mut T, builder: UiBuilder<T>, window: Window) {
        let mut needs_rebuild = false;
        let mut base = BaseCx::new(
            &mut self.contexts,
            &mut self.command_proxy,
            &mut needs_rebuild,
        );

        let window_id = window.id();
        let window_ui = WindowUi::new(builder, &mut base, data, self.style.clone(), window);
        self.windows.insert(window_id, window_ui);

        if needs_rebuild {
            self.request_rebuild();
        }

        self.handle_commands(data);
    }

    /// Remove a window.
    pub fn remove_window(&mut self, window_id: WindowId) {
        self.windows.remove(&window_id);
    }

    /// Get a reference to a window.
    ///
    /// # Panics
    /// - If the window does not exist.
    #[track_caller]
    pub fn window(&self, window_id: WindowId) -> &WindowUi<T> {
        match self.windows.get(&window_id) {
            Some(window_ui) => window_ui,
            None => panic!("window with id {:?} not found", window_id),
        }
    }

    /// Get a mutable reference to a window.
    ///
    /// # Panics
    /// - If the window does not exist.
    #[track_caller]
    pub fn window_mut(&mut self, window_id: WindowId) -> &mut WindowUi<T> {
        match self.windows.get_mut(&window_id) {
            Some(window_ui) => window_ui,
            None => panic!("window with id {:?} not found", window_id),
        }
    }

    /// Get an iterator over all windows.
    pub fn windows(&self) -> impl ExactSizeIterator<Item = &WindowUi<T>> {
        self.windows.values()
    }

    /// Get the Ids of all windows.
    pub fn window_ids(&self) -> Vec<WindowId> {
        self.windows.keys().copied().collect()
    }

    /// Get a mutable reference to the [`Fonts`] context.
    pub fn fonts(&mut self) -> &mut Fonts {
        self.contexts.get_or_default()
    }

    /// Get whether the UI should quit.
    pub fn should_quit(&self) -> bool {
        self.windows.is_empty() || self.quit_requested
    }

    /// Get a command proxy to the UI.
    pub fn proxy(&self) -> CommandProxy {
        self.command_proxy.clone()
    }

    /// Take the current [`UiRequests`].
    ///
    /// This should be done often and each [`UiRequest`] should be handled appropriately.
    pub fn take_requests(&mut self) -> UiRequests<T> {
        std::mem::take(&mut self.requests)
    }

    /// Initialize the UI.
    ///
    /// This should be called after all initial windows have been added.
    pub fn init(&mut self, data: &mut T) {
        self.init_delegate(data);
        self.event_all(data, &Event::new(()));
    }

    fn init_delegate(&mut self, data: &mut T) {
        let mut needs_rebuild = false;
        let mut base = BaseCx::new(
            &mut self.contexts,
            &mut self.command_proxy,
            &mut needs_rebuild,
        );

        let mut cx = DelegateCx::new(&mut base);

        for delegate in &mut self.delegates {
            delegate.init(&mut cx, data);
        }

        if needs_rebuild {
            self.request_rebuild();
        }

        self.handle_commands(data);
    }

    /// Tell the UI that the event loop idle.
    pub fn idle(&mut self, data: &mut T) {
        let mut needs_rebuild = false;
        let mut base = BaseCx::new(
            &mut self.contexts,
            &mut self.command_proxy,
            &mut needs_rebuild,
        );

        let mut cx = DelegateCx::new(&mut base);

        for delegate in &mut self.delegates {
            delegate.idle(&mut cx, data);
        }

        if needs_rebuild {
            self.request_rebuild();
        }

        self.handle_commands(data);
    }

    /// Request a rebuild of the view tree.
    pub fn request_rebuild(&mut self) {
        for window_ui in self.windows.values_mut() {
            window_ui.request_rebuild();
        }
    }

    /// Tell the UI that the scale factor of a window has changed.
    pub fn scale_factor_changed(&mut self, window_id: WindowId) {
        if let Some(window) = self.windows.get_mut(&window_id) {
            window.request_layout();
        }
    }

    /// Tell the UI that a window has been resized.
    pub fn resized(&mut self, window_id: WindowId) {
        if let Some(window) = self.windows.get_mut(&window_id) {
            window.request_layout();
        }
    }

    /// Tell the UI that a window wants to close.
    pub fn close_requested(&mut self, data: &mut T, window_id: WindowId) {
        let event = Event::new(CloseRequested::new(window_id));
        self.event(data, window_id, &event);

        if !event.is_handled() {
            self.requests.push(UiRequest::RemoveWindow(window_id));
        }
    }

    fn pointer_position(&self, window_id: WindowId, id: PointerId) -> Point {
        let pointer = self.window(window_id).window().pointer(id);
        pointer.map_or(Point::ZERO, |p| p.position())
    }

    /// Tell the UI that a pointer has moved.
    pub fn pointer_moved(
        &mut self,
        data: &mut T,
        window_id: WindowId,
        pointer: PointerId,
        position: Point,
    ) {
        let window = self.window_mut(window_id).window_mut();

        let prev = (window.pointer(pointer)).map_or(Point::ZERO, |p| p.position);
        let delta = position - prev;

        window.pointer_moved(pointer, position);

        let scene = self.window_mut(window_id).scene_mut();
        let view = scene.view_at(position);

        #[cfg(feature = "tracing")]
        tracing::trace!("pointer_moved: {} -> {:?}", position, view);

        let window = self.window_mut(window_id).window_mut();
        window.pointer_hovered(pointer, view);

        let event = PointerMoved {
            id: pointer,
            position,
            delta,
            modifiers: self.modifiers,
        };

        self.event(data, window_id, &Event::new(event));
    }

    /// Tell the UI that a pointer has left the window.
    pub fn pointer_left(&mut self, data: &mut T, window_id: WindowId, pointer: PointerId) {
        let event = PointerLeft { id: pointer };

        let window_ui = self.window_mut(window_id).window_mut();
        window_ui.pointer_left(pointer);

        self.event(data, window_id, &Event::new(event))
    }

    /// Tell the UI that a pointer has scrolled.
    pub fn pointer_scroll(
        &mut self,
        data: &mut T,
        window_id: WindowId,
        pointer: PointerId,
        delta: Vector,
    ) {
        let event = PointerScrolled {
            id: pointer,
            position: self.pointer_position(window_id, pointer),
            modifiers: self.modifiers,
            delta,
        };

        self.event(data, window_id, &Event::new(event));
    }

    /// Tell the UI that a pointer button has been pressed or released.
    pub fn pointer_button(
        &mut self,
        data: &mut T,
        window_id: WindowId,
        pointer: PointerId,
        button: PointerButton,
        pressed: bool,
    ) {
        if pressed {
            let event = PointerPressed {
                id: pointer,
                position: self.pointer_position(window_id, pointer),
                modifiers: self.modifiers,
                button,
            };

            self.event(data, window_id, &Event::new(event));
        } else {
            let event = PointerReleased {
                id: pointer,
                position: self.pointer_position(window_id, pointer),
                modifiers: self.modifiers,
                button,
            };

            self.event(data, window_id, &Event::new(event));
        }
    }

    /// Tell the UI that a keyboard key has been pressed or released.
    pub fn keyboard_key(
        &mut self,
        data: &mut T,
        window_id: WindowId,
        code: Option<Code>,
        text: Option<String>,
        pressed: bool,
    ) {
        if pressed {
            let event = KeyPressed {
                code,
                text,
                modifiers: self.modifiers,
            };

            self.event(data, window_id, &Event::new(event));
        } else {
            let event = KeyReleased {
                code,
                modifiers: self.modifiers,
            };

            self.event(data, window_id, &Event::new(event));
        }

        if code == Some(Code::Tab) && pressed {
            let event = Event::new(SwitchFocus::new(!self.modifiers.shift));
            self.event(data, window_id, &event);

            if !event.is_handled() {
                let event = Event::new(RequestFocus::new(!self.modifiers.shift));
                self.event(data, window_id, &event);
            }
        }
    }

    /// Tell the UI that the modifiers have changed.
    pub fn modifiers_changed(&mut self, modifiers: Modifiers) {
        self.modifiers = modifiers;
    }

    fn handle_builtin_commands(&mut self, event: Event) {
        if let Some(close) = event.get::<CloseWindow>() {
            self.requests.push(UiRequest::RemoveWindow(close.window));
            return;
        }

        if event.is::<Quit>() && !event.is_handled() {
            self.quit_requested = true;
            return;
        }

        if event.is::<OpenWindow<T>>() && !event.is_handled() {
            let open = event.take::<OpenWindow<T>>().unwrap();
            let request = UiRequest::CreateWindow(open.desc, open.builder);
            self.requests.push(request);
        }
    }

    fn handle_command(&mut self, data: &mut T, command: Command) {
        let event = Event::from(command);
        self.event_all(data, &event);
        self.handle_builtin_commands(event);
    }

    /// Handle all pending commands.
    pub fn handle_commands(&mut self, data: &mut T) {
        while let Some(command) = self.command_rx.try_recv() {
            self.handle_command(data, command);
        }
    }

    fn event_delegate(&mut self, data: &mut T, event: &Event) {
        let mut needs_rebuild = false;
        let mut base = BaseCx::new(
            &mut self.contexts,
            &mut self.command_proxy,
            &mut needs_rebuild,
        );

        let mut cx = DelegateCx::new(&mut base);

        for delegate in &mut self.delegates {
            delegate.event(&mut cx, data, event);
        }

        if needs_rebuild {
            self.request_rebuild();
        }
    }

    /// Handle an event for a single window.
    pub fn event(&mut self, data: &mut T, window_id: WindowId, event: &Event) {
        #[cfg(feature = "tracing")]
        tracing::trace!("event: {} -> {}", event.name(), window_id);

        self.event_delegate(data, event);

        let mut needs_rebuild = false;
        let mut base = BaseCx::new(
            &mut self.contexts,
            &mut self.command_proxy,
            &mut needs_rebuild,
        );

        if !event.is_handled() {
            if let Some(window_ui) = self.windows.get_mut(&window_id) {
                let requests = window_ui.event(&mut base, data, event);
                self.requests.extend(requests);
            }
        }

        if needs_rebuild {
            self.request_rebuild();
        }

        self.handle_commands(data);
    }

    /// Handle an event for all windows.
    pub fn event_all(&mut self, data: &mut T, event: &Event) {
        #[cfg(feature = "tracing")]
        tracing::trace!("event: {}", event.name());

        self.event_delegate(data, event);

        let mut needs_rebuild = false;
        let mut base = BaseCx::new(
            &mut self.contexts,
            &mut self.command_proxy,
            &mut needs_rebuild,
        );

        if !event.is_handled() {
            for window_ui in self.windows.values_mut() {
                let requests = window_ui.event(&mut base, data, event);
                self.requests.extend(requests);
            }
        }

        if needs_rebuild {
            self.request_rebuild();
        }

        self.handle_commands(data);
    }

    /// Render a window.
    pub fn render(&mut self, data: &mut T, window_id: WindowId) {
        let mut needs_rebuild = false;
        let mut base = BaseCx::new(
            &mut self.contexts,
            &mut self.command_proxy,
            &mut needs_rebuild,
        );

        if let Some(window_ui) = self.windows.get_mut(&window_id) {
            (self.requests).extend(window_ui.render(&mut base, data));
        }

        if needs_rebuild {
            self.request_rebuild();
        }

        self.handle_commands(data);
    }
}
