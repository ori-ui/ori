//! User interface state.

use std::{collections::HashMap, sync::Arc};

use crossbeam_channel::Receiver;
use ori_macro::font;

use crate::{
    command::{Command, CommandProxy},
    delegate::{Delegate, DelegateCx},
    event::{
        CloseRequested, CloseWindow, Code, Event, KeyboardEvent, Modifiers, OpenWindow,
        PointerButton, PointerId, PointerLeft, PointerMoved, PointerPressed, PointerReleased,
        PointerScrolled, RequestFocus, SwitchFocus,
    },
    layout::{Point, Vector},
    text::Fonts,
    theme::{Theme, ThemeBuilder, SCALE_FACTOR, WINDOW_SIZE},
    view::{BaseCx, Contexts},
    window::{Window, WindowId},
};

use super::{UiBuilder, UiRequest, UiRequests, WindowUi};

macro_rules! base_cx {
    ($self:expr, $needs_rebuild:ident, $base:ident) => {
        let mut $needs_rebuild = false;
        let mut $base = BaseCx::new(
            &mut $self.fonts,
            &mut $self.contexts,
            &mut $self.command_proxy,
            &mut $needs_rebuild,
        );
    };
}

/// State for running a user interface.
pub struct Ui<T: 'static> {
    windows: HashMap<WindowId, WindowUi<T>>,
    modifiers: Modifiers,
    delegates: Vec<Box<dyn Delegate<T>>>,
    theme_builder: ThemeBuilder,
    command_proxy: CommandProxy,
    command_rx: Receiver<Command>,
    requests: UiRequests<T>,
    /// The contexts used by the UI.
    pub contexts: Contexts,
    /// The fonts used by the UI.
    pub fonts: Fonts,
    /// The data used by the UI.
    pub data: T,
}

impl<T> Ui<T> {
    /// Create a new [`Ui`] with the given data.
    pub fn new(data: T, waker: Arc<dyn Fn() + Send + Sync>) -> Self {
        let mut fonts = Fonts::default();

        fonts.load_font(font!("font/NotoSans-Regular.ttf")).unwrap();

        let (command_proxy, command_rx) = CommandProxy::new(waker);

        Self {
            windows: HashMap::new(),
            modifiers: Modifiers::default(),
            delegates: Vec::new(),
            theme_builder: ThemeBuilder::default(),
            command_proxy,
            command_rx,
            requests: UiRequests::new(),
            contexts: Contexts::new(),
            fonts,
            data,
        }
    }

    /// Push a delegate.
    pub fn push_delegate<D: Delegate<T> + 'static>(&mut self, delegate: D) {
        self.delegates.push(Box::new(delegate));
    }

    /// Get the delegates.
    pub fn delegates(&self) -> &[Box<dyn Delegate<T>>] {
        &self.delegates
    }

    /// Add a new theme.
    pub fn push_theme(&mut self, theme: impl FnMut() -> Theme + 'static) {
        self.theme_builder.push(Box::new(theme));
    }

    fn build_theme(builder: &mut ThemeBuilder, window: &Window) -> Theme {
        let mut theme = Theme::new();
        theme.set(SCALE_FACTOR, window.scale_factor());
        theme.set(WINDOW_SIZE, window.size());

        builder.build(&mut theme);

        theme
    }

    /// Add a new window.
    pub fn add_window(&mut self, builder: UiBuilder<T>, window: Window) {
        let theme = Self::build_theme(&mut self.theme_builder, &window);

        base_cx!(self, needs_rebuild, base);

        let window_id = window.id();
        let window_ui = WindowUi::new(builder, &mut base, &mut self.data, theme, window);
        self.windows.insert(window_id, window_ui);

        if needs_rebuild {
            self.request_rebuild();
        }

        self.handle_commands();
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

    /// Get whether the UI should exit.
    pub fn should_exit(&self) -> bool {
        self.windows.is_empty()
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
    pub fn init(&mut self) {
        self.init_delegate();
    }

    fn init_delegate(&mut self) {
        base_cx!(self, needs_rebuild, base);
        let mut cx = DelegateCx::new(&mut base);

        for delegate in &mut self.delegates {
            delegate.init(&mut cx, &mut self.data);
        }

        if needs_rebuild {
            self.request_rebuild();
        }

        self.handle_commands();
    }

    /// Tell the UI that the event loop idle.
    pub fn idle(&mut self) {
        base_cx!(self, needs_rebuild, base);
        let mut cx = DelegateCx::new(&mut base);

        for delegate in &mut self.delegates {
            delegate.idle(&mut cx, &mut self.data);
        }

        if needs_rebuild {
            self.request_rebuild();
        }

        self.handle_commands();
    }

    /// Rebuild the theme for a window.
    pub fn rebuild_theme(&mut self, window_id: WindowId) {
        if let Some(window_ui) = self.windows.get_mut(&window_id) {
            let theme = Self::build_theme(&mut self.theme_builder, window_ui.window());
            self.window_mut(window_id).set_theme(theme);
        }
    }

    /// Request a rebuild of the view tree.
    pub fn request_rebuild(&mut self) {
        for window_ui in self.windows.values_mut() {
            window_ui.request_rebuild();
        }
    }

    /// Tell the UI that the scale factor of a window has changed.
    pub fn scale_factor_changed(&mut self, window_id: WindowId) {
        self.rebuild_theme(window_id);
    }

    /// Tell the UI that a window has been resized.
    pub fn resized(&mut self, window_id: WindowId) {
        self.rebuild_theme(window_id);
        self.window_mut(window_id).request_layout();
    }

    /// Tell the UI that a window wants to close.
    pub fn close_requested(&mut self, window_id: WindowId) {
        let event = Event::new(CloseRequested::new(window_id));
        self.event(window_id, &event);

        if !event.is_handled() {
            self.requests.push(UiRequest::RemoveWindow(window_id));
        }
    }

    fn pointer_position(&self, window_id: WindowId, id: PointerId) -> Point {
        let pointer = self.window(window_id).window().pointer(id);
        pointer.map_or(Point::ZERO, |p| p.position())
    }

    /// Tell the UI that a pointer has moved.
    pub fn pointer_moved(&mut self, window_id: WindowId, pointer: PointerId, position: Point) {
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

        self.event(window_id, &Event::new(event));
    }

    /// Tell the UI that a pointer has left the window.
    pub fn pointer_left(&mut self, window_id: WindowId, pointer: PointerId) {
        let event = PointerLeft { id: pointer };

        let window_ui = self.window_mut(window_id).window_mut();
        window_ui.pointer_left(pointer);

        self.event(window_id, &Event::new(event))
    }

    /// Tell the UI that a pointer has scrolled.
    pub fn pointer_scroll(&mut self, window_id: WindowId, pointer: PointerId, delta: Vector) {
        let event = PointerScrolled {
            id: pointer,
            position: self.pointer_position(window_id, pointer),
            modifiers: self.modifiers,
            delta,
        };

        self.event(window_id, &Event::new(event));
    }

    /// Tell the UI that a pointer button has been pressed or released.
    pub fn pointer_button(
        &mut self,
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

            self.event(window_id, &Event::new(event));
        } else {
            let event = PointerReleased {
                id: pointer,
                position: self.pointer_position(window_id, pointer),
                modifiers: self.modifiers,
                button,
            };

            self.event(window_id, &Event::new(event));
        }
    }

    /// Tell the UI that a keyboard key has been pressed or released.
    pub fn keyboard_key(&mut self, window_id: WindowId, key: Code, pressed: bool) {
        let event = KeyboardEvent {
            modifiers: self.modifiers,
            code: Some(key),
            pressed,
            ..Default::default()
        };

        self.event(window_id, &Event::new(event));

        if key == Code::Tab && pressed {
            let event = Event::new(SwitchFocus::new(!self.modifiers.shift));
            self.event(window_id, &event);

            if !event.is_handled() {
                let event = Event::new(RequestFocus::new(!self.modifiers.shift));
                self.event(window_id, &event);
            }
        }
    }

    /// Tell the UI that a keyboard character has been entered.
    pub fn keyboard_text(&mut self, window_id: WindowId, text: String) {
        let event = KeyboardEvent {
            modifiers: self.modifiers,
            text: Some(text),
            ..Default::default()
        };

        self.event(window_id, &Event::new(event));
    }

    /// Tell the UI that the modifiers have changed.
    pub fn modifiers_changed(&mut self, modifiers: Modifiers) {
        self.modifiers = modifiers;
    }

    fn handle_window_commands(&mut self, event: Event) {
        if let Some(close) = event.get::<CloseWindow>() {
            self.requests.push(UiRequest::RemoveWindow(close.window));
        }

        if event.is::<OpenWindow<T>>() && !event.is_handled() {
            let open = event.take::<OpenWindow<T>>().unwrap();
            let request = UiRequest::CreateWindow(open.desc, open.builder);
            self.requests.push(request);
        }
    }

    fn handle_command(&mut self, command: Command) {
        let event = Event::from_command(command);

        base_cx!(self, needs_rebuild, base);
        let mut cx = DelegateCx::new(&mut base);

        for delegate in &mut self.delegates {
            delegate.event(&mut cx, &mut self.data, &event);
        }

        if needs_rebuild {
            self.request_rebuild();
        }

        if !event.is_handled() {
            for window_id in self.window_ids() {
                self.event(window_id, &event);
            }
        }

        self.handle_window_commands(event);
    }

    /// Handle all pending commands.
    pub fn handle_commands(&mut self) {
        while let Ok(command) = self.command_rx.try_recv() {
            self.handle_command(command);
        }
    }

    fn event_delegate(&mut self, event: &Event) {
        base_cx!(self, needs_rebuild, base);
        let mut cx = DelegateCx::new(&mut base);

        for delegate in &mut self.delegates {
            delegate.event(&mut cx, &mut self.data, event);
        }

        if needs_rebuild {
            self.request_rebuild();
        }
    }

    /// Handle an event for a single window.
    pub fn event(&mut self, window_id: WindowId, event: &Event) {
        #[cfg(feature = "tracing")]
        tracing::trace!("event: {} -> {}", event.name(), window_id);

        self.event_delegate(event);

        base_cx!(self, needs_rebuild, base);

        if !event.is_handled() {
            if let Some(window_ui) = self.windows.get_mut(&window_id) {
                window_ui.event(&mut base, &mut self.data, event);
            }
        }

        if needs_rebuild {
            self.request_rebuild();
        }

        self.handle_commands();
    }

    /// Handle an event for all windows.
    pub fn event_all(&mut self, event: &Event) {
        #[cfg(feature = "tracing")]
        tracing::trace!("event: {}", event.name());

        self.event_delegate(event);

        base_cx!(self, needs_rebuild, base);

        if !event.is_handled() {
            for window_ui in self.windows.values_mut() {
                window_ui.event(&mut base, &mut self.data, event);
            }
        }

        if needs_rebuild {
            self.request_rebuild();
        }

        self.handle_commands();
    }

    /// Render a window.
    pub fn render(&mut self, window_id: WindowId) {
        base_cx!(self, needs_rebuild, base);

        if let Some(window_ui) = self.windows.get_mut(&window_id) {
            (self.requests).extend(window_ui.render(&mut base, &mut self.data));
        }

        if needs_rebuild {
            self.request_rebuild();
        }

        self.handle_commands();
    }
}
