//! User interface state.

use std::{collections::HashMap, sync::Arc};

use ori_macro::font;

use crate::{
    canvas::SceneRender,
    command::{Command, CommandProxy, EventLoopWaker},
    delegate::{Delegate, DelegateCx},
    event::{
        CloseRequested, Code, Event, Focused, KeyboardEvent, Modifiers, PointerButton,
        PointerEvent, PointerId, SwitchFocus,
    },
    layout::{Point, Vector},
    text::Fonts,
    theme::{Theme, SCALE_FACTOR, WINDOW_SIZE},
    view::BaseCx,
    window::{UiBuilder, Window, WindowId, WindowUi},
};

/// State for running a user interface.
pub struct Ui<T, R: SceneRender> {
    windows: HashMap<WindowId, WindowUi<T, R>>,
    modifiers: Modifiers,
    delegate: Box<dyn Delegate<T>>,
    themes: Vec<Box<dyn FnMut() -> Theme>>,
    commands: CommandProxy,
    /// The fonts used by the UI.
    pub fonts: Fonts,
    /// The data used by the UI.
    pub data: T,
}

impl<T, R: SceneRender> Ui<T, R> {
    /// Create a new [`Ui`] with the given data.
    pub fn new(data: T, waker: Arc<dyn EventLoopWaker>) -> Self {
        let mut fonts = Fonts::default();

        fonts.load_font(font!("font/NotoSans-Regular.ttf")).unwrap();

        Self {
            windows: HashMap::new(),
            modifiers: Modifiers::default(),
            delegate: Box::new(()),
            themes: Vec::new(),
            commands: CommandProxy::new(waker),
            fonts,
            data,
        }
    }

    /// Override the delegate.
    pub fn set_delegate<D: Delegate<T> + 'static>(&mut self, delegate: D) {
        self.delegate = Box::new(delegate);
    }

    /// Add a new theme.
    pub fn add_theme(&mut self, theme: impl FnMut() -> Theme + 'static) {
        self.themes.push(Box::new(theme));
    }

    fn build_theme(themes: &mut Vec<Box<dyn FnMut() -> Theme>>, window: &Window) -> Theme {
        let mut theme = Theme::new();
        theme.set(SCALE_FACTOR, window.scale_factor());
        theme.set(WINDOW_SIZE, window.size());

        for theme_builder in themes {
            let new_theme = Theme::with_global(&mut theme, theme_builder);
            theme.extend(new_theme);
        }

        theme
    }

    /// Add a new window.
    pub fn add_window(&mut self, builder: UiBuilder<T>, window: Window, render: R) {
        let theme = Self::build_theme(&mut self.themes, &window);

        let mut needs_rebuild = false;
        let mut base = BaseCx::new(&mut self.fonts, &mut self.commands, &mut needs_rebuild);

        let window_id = window.id();
        let window_ui = WindowUi::new(builder, &mut base, &mut self.data, theme, window, render);
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
    pub fn window(&self, window_id: WindowId) -> &WindowUi<T, R> {
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
    pub fn window_mut(&mut self, window_id: WindowId) -> &mut WindowUi<T, R> {
        match self.windows.get_mut(&window_id) {
            Some(window_ui) => window_ui,
            None => panic!("window with id {:?} not found", window_id),
        }
    }

    /// Get an iterator over all windows.
    pub fn windows(&self) -> impl ExactSizeIterator<Item = &WindowUi<T, R>> {
        self.windows.values()
    }

    /// Get the Ids of all windows.
    pub fn window_ids(&self) -> Vec<WindowId> {
        self.windows.keys().copied().collect()
    }

    /// Get a command proxy to the UI.
    pub fn proxy(&self) -> CommandProxy {
        self.commands.clone()
    }

    /// Initialize the UI.
    ///
    /// This should be called after all initial windows have been added.
    pub fn init(&mut self) {
        self.init_delegate();
    }

    fn init_delegate(&mut self) {
        let mut needs_rebuild = false;
        let mut base = BaseCx::new(&mut self.fonts, &mut self.commands, &mut needs_rebuild);
        let mut cx = DelegateCx::new(&mut base);

        self.delegate.init(&mut cx, &mut self.data);

        if needs_rebuild {
            self.request_rebuild();
        }

        self.handle_commands();
    }

    /// Tell the UI that the event loop idle.
    pub fn idle(&mut self) {
        for window_ui in self.windows.values_mut() {
            window_ui.idle();
        }

        let mut needs_rebuild = false;
        let mut base = BaseCx::new(&mut self.fonts, &mut self.commands, &mut needs_rebuild);
        let mut cx = DelegateCx::new(&mut base);

        self.delegate.idle(&mut cx, &mut self.data);

        if needs_rebuild {
            self.request_rebuild();
        }

        self.handle_commands();
    }

    /// Rebuild the theme for a window.
    pub fn rebuild_theme(&mut self, window_id: WindowId) {
        if let Some(window_ui) = self.windows.get_mut(&window_id) {
            let theme = Self::build_theme(&mut self.themes, window_ui.window());
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
    pub fn close_requested(&mut self, window_id: WindowId) -> bool {
        let event = Event::new(CloseRequested::new(window_id));
        self.event(window_id, &event);
        !event.is_handled()
    }

    fn pointer_position(&self, window_id: WindowId, id: PointerId) -> Point {
        let pointer = self.window(window_id).window().pointer(id);
        pointer.map_or(Point::ZERO, |p| p.position())
    }

    /// Tell the UI that a pointer has moved.
    pub fn pointer_moved(&mut self, window_id: WindowId, id: PointerId, position: Point) {
        let window_ui = self.window_mut(window_id).window_mut();

        let prev = window_ui.pointer(id).map_or(Point::ZERO, |p| p.position);
        let delta = position - prev;

        window_ui.pointer_moved(id, position);

        let event = PointerEvent {
            position,
            delta,
            modifiers: self.modifiers,
            ..PointerEvent::new(id)
        };

        self.event(window_id, &Event::new(event));
    }

    /// Tell the UI that a pointer has left the window.
    pub fn pointer_left(&mut self, window_id: WindowId, id: PointerId) {
        let event = PointerEvent {
            position: self.pointer_position(window_id, id),
            modifiers: self.modifiers,
            left: true,
            ..PointerEvent::new(id)
        };

        let window_ui = self.window_mut(window_id).window_mut();
        window_ui.pointer_left(id);

        self.event(window_id, &Event::new(event));
    }

    /// Tell the UI that a pointer has scrolled.
    pub fn pointer_scroll(&mut self, window_id: WindowId, id: PointerId, delta: Vector) {
        let event = PointerEvent {
            position: self.pointer_position(window_id, id),
            modifiers: self.modifiers,
            scroll: delta,
            ..PointerEvent::new(id)
        };

        self.event(window_id, &Event::new(event));
    }

    /// Tell the UI that a pointer button has been pressed or released.
    pub fn pointer_button(
        &mut self,
        window_id: WindowId,
        id: PointerId,
        button: PointerButton,
        pressed: bool,
    ) {
        let event = PointerEvent {
            position: self.pointer_position(window_id, id),
            modifiers: self.modifiers,
            button: Some(button),
            pressed,
            ..PointerEvent::new(id)
        };

        self.event(window_id, &Event::new(event));
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
                let event = Event::new(Focused::new(!self.modifiers.shift));
                self.event(window_id, &event);
            }
        }
    }

    /// Tell the UI that a keyboard character has been entered.
    pub fn keyboard_char(&mut self, window_id: WindowId, c: char) {
        let event = KeyboardEvent {
            modifiers: self.modifiers,
            text: Some(String::from(c)),
            ..Default::default()
        };

        self.event(window_id, &Event::new(event));
    }

    /// Tell the UI that the modifiers have changed.
    pub fn modifiers_changed(&mut self, modifiers: Modifiers) {
        self.modifiers = modifiers;
    }

    fn handle_command(&mut self, command: Command) {
        let event = Event::from_command(command);

        let mut needs_rebuild = false;
        let mut base = BaseCx::new(&mut self.fonts, &mut self.commands, &mut needs_rebuild);
        let mut cx = DelegateCx::new(&mut base);

        self.delegate.event(&mut cx, &mut self.data, &event);

        if needs_rebuild {
            self.request_rebuild();
        }

        if !event.is_handled() {
            for window_id in self.window_ids() {
                self.event(window_id, &event);
            }
        }
    }

    /// Handle all pending commands.
    pub fn handle_commands(&mut self) {
        while let Ok(command) = self.commands.rx.try_recv() {
            self.handle_command(command);
        }
    }

    fn event_delegate(&mut self, event: &Event) {
        let mut needs_rebuild = false;
        let mut base = BaseCx::new(&mut self.fonts, &mut self.commands, &mut needs_rebuild);
        let mut cx = DelegateCx::new(&mut base);

        self.delegate.event(&mut cx, &mut self.data, event);

        if needs_rebuild {
            self.request_rebuild();
        }
    }

    /// Handle an event for a single window.
    pub fn event(&mut self, window_id: WindowId, event: &Event) {
        self.event_delegate(event);

        let mut needs_rebuild = false;
        let mut base = BaseCx::new(&mut self.fonts, &mut self.commands, &mut needs_rebuild);

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
        self.event_delegate(event);

        let mut needs_rebuild = false;
        let mut base = BaseCx::new(&mut self.fonts, &mut self.commands, &mut needs_rebuild);

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
        let mut needs_rebuild = false;
        let mut base = BaseCx::new(&mut self.fonts, &mut self.commands, &mut needs_rebuild);

        if let Some(window_ui) = self.windows.get_mut(&window_id) {
            window_ui.render(&mut base, &mut self.data);
        }

        if needs_rebuild {
            self.request_rebuild();
        }

        self.handle_commands();
    }
}
