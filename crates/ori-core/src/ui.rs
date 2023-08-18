use std::collections::HashMap;

use glam::Vec2;

use crate::{
    BaseCx, Code, Delegate, Event, Fonts, KeyboardEvent, Modifiers, Palette, PointerButton,
    PointerEvent, PointerId, SceneRender, Theme, UiBuilder, Window, WindowId, WindowUi,
};

/// State for running a user interface.
pub struct Ui<T, R: SceneRender> {
    windows: HashMap<WindowId, WindowUi<T, R>>,
    modifiers: Modifiers,
    delegate: Box<dyn Delegate<T>>,
    /// The fonts used by the UI.
    pub fonts: Fonts,
    /// The theme used by the UI.
    pub theme: Theme,
    /// The data used by the UI.
    pub data: T,
}

impl<T, R: SceneRender> Ui<T, R> {
    /// Create a new [`Ui`] with the given data.
    pub fn new(data: T) -> Self {
        let mut fonts = Fonts::default();
        fonts.load_system_fonts();

        let mut theme = Theme::builtin();
        theme.extend(Palette::light());

        Self {
            windows: HashMap::new(),
            modifiers: Modifiers::default(),
            delegate: Box::new(()),
            fonts,
            theme,
            data,
        }
    }

    /// Override the delegate.
    pub fn set_delegate<D: Delegate<T> + 'static>(&mut self, delegate: D) {
        self.delegate = Box::new(delegate);
    }

    /// Add a new window.
    pub fn add_window(&mut self, builder: UiBuilder<T>, window: Window, render: R) {
        Theme::with_global(&mut self.theme, || {
            let mut commands = Vec::new();
            let mut base = BaseCx::new(&mut self.fonts, &mut commands);

            let window_id = window.id();
            let window_ui = WindowUi::new(builder, &mut base, &mut self.data, window, render);
            self.windows.insert(window_id, window_ui);
        });
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
        window.pointer_moved(id, position);

        let event = PointerEvent {
            position,
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
            scroll_delta: delta,
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
            key: Some(key),
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

    /// Handle an event for a window.
    pub fn event(&mut self, window_id: WindowId, event: &Event) {
        if let Some(window_ui) = self.windows.get_mut(&window_id) {
            let mut commands = Vec::new();
            let mut base = BaseCx::new(&mut self.fonts, &mut commands);

            Theme::with_global(&mut self.theme, || {
                window_ui.event(&mut *self.delegate, &mut base, &mut self.data, event);
            });
        }
    }

    /// Render a window.
    pub fn render(&mut self, window_id: WindowId) {
        if let Some(window_ui) = self.windows.get_mut(&window_id) {
            let mut commands = Vec::new();
            let mut base = BaseCx::new(&mut self.fonts, &mut commands);

            Theme::with_global(&mut self.theme, || {
                window_ui.render(&mut *self.delegate, &mut base, &mut self.data);
            });
        }
    }
}
