//! User interface state.

use std::{collections::HashMap, sync::Arc};

use glam::Vec2;

use crate::{
    canvas::SceneRender,
    delegate::{Delegate, DelegateCx},
    event::{Code, Event, KeyboardEvent, Modifiers, PointerButton, PointerEvent, PointerId},
    proxy::{Command, Proxy, ProxyWaker},
    style::{set_style, styled, Theme, SCALE_FACTOR},
    text::Fonts,
    view::BaseCx,
    window::{UiBuilder, Window, WindowId, WindowUi},
};

/// State for running a user interface.
pub struct Ui<T, R: SceneRender> {
    windows: HashMap<WindowId, WindowUi<T, R>>,
    modifiers: Modifiers,
    delegate: Box<dyn Delegate<T>>,
    themes: Vec<Box<dyn FnMut() -> Theme>>,
    commands: Proxy,
    /// The fonts used by the UI.
    pub fonts: Fonts,
    /// The data used by the UI.
    pub data: T,
}

impl<T, R: SceneRender> Ui<T, R> {
    /// Create a new [`Ui`] with the given data.
    pub fn new(data: T, waker: Arc<dyn ProxyWaker>) -> Self {
        Self {
            windows: HashMap::new(),
            modifiers: Modifiers::default(),
            delegate: Box::new(()),
            themes: Vec::new(),
            commands: Proxy::new(waker),
            fonts: Fonts::default(),
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

    /// Build the theme.
    pub fn build_theme(&mut self, scale_factor: f32) -> Theme {
        styled(|| {
            set_style(SCALE_FACTOR, scale_factor);

            let mut theme = Theme::builtin();

            for theme_fn in &mut self.themes {
                theme.extend(theme_fn());
            }

            theme.set(SCALE_FACTOR, scale_factor);

            theme
        })
    }

    /// Add a new window.
    pub fn add_window(&mut self, builder: UiBuilder<T>, window: Window, render: R) {
        let theme = self.build_theme(window.scale_factor());

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
            Some(window) => window,
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
            Some(window) => window,
            None => panic!("window with id {:?} not found", window_id),
        }
    }

    /// Get the Ids of all windows.
    pub fn window_ids(&self) -> Vec<WindowId> {
        self.windows.keys().copied().collect()
    }

    /// Tell the UI that the event loop idle.
    pub fn idle(&mut self) {
        for window in self.windows.values_mut() {
            window.idle();
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
    ///
    /// This should be called when the scale factor of the window changes.
    pub fn rebuild_theme(&mut self, window_id: WindowId) {
        let scale_factor = self.window(window_id).window().scale_factor();
        let theme = self.build_theme(scale_factor);
        self.window_mut(window_id).set_theme(theme);
    }

    /// Request a rebuild of the view tree.
    pub fn request_rebuild(&mut self) {
        for window in self.windows.values_mut() {
            window.request_rebuild();
        }
    }

    /// Tell the UI that a window has been resized.
    pub fn resized(&mut self, window_id: WindowId) {
        self.window_mut(window_id).request_layout();
    }

    fn pointer_position(&self, window_id: WindowId, id: PointerId) -> Vec2 {
        let pointer = self.window(window_id).window().pointer(id);
        pointer.map_or(Vec2::ZERO, |p| p.position())
    }

    /// Tell the UI that a pointer has moved.
    pub fn pointer_moved(&mut self, window_id: WindowId, id: PointerId, position: Vec2) {
        let window = self.window_mut(window_id).window_mut();

        let prev = window.pointer(id).map_or(Vec2::ZERO, |p| p.position);
        let delta = position - prev;

        window.pointer_moved(id, position);

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

        let window = self.window_mut(window_id).window_mut();
        window.pointer_left(id);

        self.event(window_id, &Event::new(event));
    }

    /// Tell the UI that a pointer has scrolled.
    pub fn pointer_scroll(&mut self, window_id: WindowId, id: PointerId, delta: Vec2) {
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

    /// Handle an event for a window.
    pub fn event(&mut self, window_id: WindowId, event: &Event) {
        let mut needs_rebuild = false;
        let mut base = BaseCx::new(&mut self.fonts, &mut self.commands, &mut needs_rebuild);

        if let Some(window_ui) = self.windows.get_mut(&window_id) {
            window_ui.event(&mut base, &mut self.data, event);
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
