use std::collections::HashMap;

use instant::Instant;
use ori_core::{
    canvas::{Canvas, Color, Scene},
    clipboard::{Clipboard, ClipboardContext},
    command::{CommandProxy, CommandReceiver},
    context::{BaseCx, BuildCx, Contexts, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::{
        Code, Event, KeyPressed, KeyReleased, Modifiers, PointerButton, PointerId, PointerMoved,
        PointerPressed, PointerReleased, PointerScrolled, WindowResized,
    },
    layout::{Point, Size, Space, Vector},
    style::Styles,
    view::{AnyState, BoxedView, View, ViewState},
    window::{Cursor, Window, WindowId, WindowSizing, WindowSnapshot, WindowUpdate},
};

use crate::{AppBuilder, AppRequest, Delegate, DelegateCx, UiBuilder};

/// Information needed to render a window.
pub struct WindowRenderScene<'a> {
    /// The scene to render.
    pub scene: &'a Scene,
    /// The size of the window.
    pub logical_size: Size,
    /// The clear color of the window.
    pub clear_color: Color,
}

pub(crate) struct WindowState<T> {
    ui: UiBuilder<T>,
    view: BoxedView<T>,
    cursor: Cursor,
    state: AnyState,
    scene: Scene,
    view_state: ViewState,
    window: Window,
    snapshot: WindowSnapshot,
    animate: Option<Instant>,
}

impl<T> WindowState<T> {
    fn rebuild(&mut self, data: &mut T, base: &mut BaseCx) {
        let mut cx = RebuildCx::new(base, &mut self.view_state, &mut self.window);

        let mut new_view = (self.ui)(data);
        new_view.rebuild(&mut self.state, &mut cx, data, &self.view);
        self.view = new_view;
    }

    fn event(&mut self, data: &mut T, base: &mut BaseCx, rebuild: &mut bool, event: &Event) {
        let hot = self.window.is_hovered(self.view_state.id());
        self.view_state.set_hot(hot);

        let mut cx = EventCx::new(base, &mut self.view_state, rebuild, &mut self.window);

        self.view.event(&mut self.state, &mut cx, data, event);
    }

    fn layout(&mut self, data: &mut T, base: &mut BaseCx) {
        self.view_state.prepare();
        self.view_state.mark_layed_out();

        // we need to calculate the max size of the window
        // depending on the sizing of the window
        let max_size = match self.window.sizing {
            WindowSizing::Fixed => self.window.size,
            WindowSizing::Content => Size::INFINITY,
        };

        let space = Space::new(Size::ZERO, max_size);
        let mut cx = LayoutCx::new(base, &mut self.view_state, &mut self.window);

        let size = self.view.layout(&mut self.state, &mut cx, data, space);
        self.view_state.set_size(size);

        // if the window is content sized we set the
        // window size to the content size
        if let WindowSizing::Content = self.window.sizing {
            self.window.size = size;
        }
    }

    fn draw(&mut self, data: &mut T, base: &mut BaseCx) {
        self.view_state.prepare();
        self.view_state.mark_drawn();

        self.scene.clear();

        let mut canvas = Canvas::new(&mut self.scene, self.window.size);
        let mut cx = DrawCx::new(base, &mut self.view_state, &mut self.window);

        self.view.draw(&mut self.state, &mut cx, data, &mut canvas);

        self.scene.sort();
    }

    fn animate(&mut self) -> f32 {
        match self.animate.take() {
            Some(animate) => animate.elapsed().as_secs_f32(),
            None => 0.0,
        }
    }

    fn update_and_request_draw(&mut self, animate: Instant) -> Vec<AppRequest<T>> {
        let mut redraw = false;

        if self.view_state.needs_draw() {
            redraw = true;
        }

        if self.view_state.needs_animate() && self.animate.is_none() {
            self.animate = Some(animate);

            redraw = true;
        }

        let cursor = self.view_state.cursor().unwrap_or_default();

        let mut requests = Vec::new();

        if redraw {
            requests.push(AppRequest::RequestRedraw(self.window.id()));
        }

        if cursor != self.cursor {
            requests.push(AppRequest::UpdateWindow(
                self.window.id(),
                WindowUpdate::Cursor(cursor),
            ));

            self.cursor = cursor;
        }

        requests
    }
}

/// The main application state.
pub struct App<T> {
    pub(crate) windows: HashMap<WindowId, WindowState<T>>,
    pub(crate) modifiers: Modifiers,
    pub(crate) delegates: Vec<Box<dyn Delegate<T>>>,
    pub(crate) proxy: CommandProxy,
    pub(crate) receiver: CommandReceiver,
    pub(crate) style: Styles,
    pub(crate) requests: Vec<AppRequest<T>>,
    pub(crate) contexts: Contexts,
}

impl<T> App<T> {
    /// Create a new application builder.
    pub fn build() -> AppBuilder<T> {
        AppBuilder::new()
    }

    /// A window was requested to be closed.
    pub fn close_requested(&mut self, _data: &mut T, window_id: WindowId) {
        self.remove_window(window_id);

        if self.windows.is_empty() {
            self.requests.push(AppRequest::Quit);
        }
    }

    /// A window was resized.
    pub fn window_resized(&mut self, data: &mut T, window_id: WindowId, width: u32, height: u32) {
        if let Some(window_state) = self.windows.get_mut(&window_id) {
            window_state.view_state.request_layout();
            window_state.window.size = Size::new(width as f32, height as f32);
            window_state.snapshot.size = Size::new(width as f32, height as f32);
        }

        let event = Event::WindowResized(WindowResized {
            window: window_id,
            width,
            height,
        });

        self.window_event(data, window_id, &event);
    }

    /// A pointer moved.
    pub fn pointer_moved(
        &mut self,
        data: &mut T,
        window_id: WindowId,
        pointer_id: PointerId,
        position: Point,
    ) {
        let Some(window_state) = self.windows.get_mut(&window_id) else {
            return;
        };

        let delta = window_state.window.move_pointer(pointer_id, position);
        self.update_hovered(window_id);

        let event = Event::PointerMoved(PointerMoved {
            id: pointer_id,
            modifiers: self.modifiers,
            position,
            delta,
        });

        self.window_event(data, window_id, &event);
    }

    /// A pointer left the window.
    pub fn pointer_left(&mut self, data: &mut T, window_id: WindowId, pointer_id: PointerId) {
        let Some(window_state) = self.windows.get_mut(&window_id) else {
            return;
        };

        window_state.window.remove_pointer(pointer_id);

        let event = Event::PointerLeft(pointer_id);

        self.window_event(data, window_id, &event);
    }

    fn pointer_position(&self, window_id: WindowId, pointer_id: PointerId) -> Option<Point> {
        let window = self.get_window(window_id)?;
        let pointer = window.get_pointer(pointer_id)?;
        Some(pointer.position)
    }

    /// A pointer scrolled.
    pub fn pointer_scrolled(
        &mut self,
        data: &mut T,
        window_id: WindowId,
        pointer_id: PointerId,
        delta: Vector,
    ) {
        let position = self
            .pointer_position(window_id, pointer_id)
            .unwrap_or(Point::ZERO);

        let event = Event::PointerScrolled(PointerScrolled {
            id: pointer_id,
            modifiers: self.modifiers,
            position,
            delta,
        });

        self.window_event(data, window_id, &event);
    }

    /// A pointer button was pressed or released.
    pub fn pointer_button(
        &mut self,
        data: &mut T,
        window_id: WindowId,
        pointer_id: PointerId,
        button: PointerButton,
        pressed: bool,
    ) {
        let position = self
            .pointer_position(window_id, pointer_id)
            .unwrap_or(Point::ZERO);

        if pressed {
            if let Some(window_state) = self.windows.get_mut(&window_id) {
                window_state.window.press_pointer(pointer_id, button);
            }

            let event = Event::PointerPressed(PointerPressed {
                id: pointer_id,
                modifiers: self.modifiers,
                position,
                button,
            });

            self.window_event(data, window_id, &event);
        } else {
            let clicked = (self.windows.get_mut(&window_id)).map_or(false, move |window_state| {
                window_state.window.release_pointer(pointer_id, button)
            });

            let event = Event::PointerReleased(PointerReleased {
                id: pointer_id,
                modifiers: self.modifiers,
                clicked,
                position,
                button,
            });

            self.window_event(data, window_id, &event);
        }
    }

    /// A keyboard key was pressed or released.
    pub fn keyboard_key(
        &mut self,
        data: &mut T,
        window_id: WindowId,
        code: Option<Code>,
        text: Option<String>,
        pressed: bool,
    ) {
        if pressed {
            let event = Event::KeyPressed(KeyPressed {
                code,
                text,
                modifiers: self.modifiers,
            });

            self.window_event(data, window_id, &event);
        } else {
            let event = Event::KeyReleased(KeyReleased {
                code,
                modifiers: self.modifiers,
            });

            self.window_event(data, window_id, &event);
        }
    }

    /// The modifiers changed.
    pub fn modifiers_changed(&mut self, modifiers: Modifiers) {
        self.modifiers = modifiers;
    }
}

impl<T> App<T> {
    /// Add a window to the application.
    pub fn add_window(&mut self, data: &mut T, mut ui: UiBuilder<T>, mut window: Window) {
        let mut view = self.style.as_context(|| ui(data));
        let mut view_state = ViewState::default();

        let mut base = BaseCx::new(&mut self.contexts, &mut self.proxy);

        let snapshot = window.snapshot();

        let mut cx = BuildCx::new(&mut base, &mut view_state, &mut window);
        let state = self.style.as_context(|| view.build(&mut cx, data));

        let window_id = window.id();
        let window_state = WindowState {
            ui,
            view,
            cursor: Cursor::Default,
            state,
            scene: Scene::new(),
            view_state,
            window,
            snapshot,
            animate: None,
        };

        self.windows.insert(window_id, window_state);
    }

    /// Remove a window from the application.
    pub fn remove_window(&mut self, window_id: WindowId) {
        self.windows.remove(&window_id);
    }

    /// Get a window by id.
    pub fn get_window(&self, window_id: WindowId) -> Option<&Window> {
        self.windows.get(&window_id).map(|w| &w.window)
    }

    /// Get a mutable window by id.
    pub fn get_window_mut(&mut self, window_id: WindowId) -> Option<&mut Window> {
        self.windows.get_mut(&window_id).map(|w| &mut w.window)
    }

    /// Set the clipboard.
    pub fn set_clipboard(&mut self, clipboard: impl Clipboard + 'static) {
        self.contexts.insert(ClipboardContext::new(clipboard));
    }

    /// Take all pending requests.
    pub fn take_requests(&mut self) -> impl Iterator<Item = AppRequest<T>> {
        std::mem::take(&mut self.requests).into_iter()
    }

    /// Handle all pending commands.
    pub fn handle_commands(&mut self, data: &mut T) {
        while let Some(command) = self.receiver.try_recv() {
            self.event(data, &Event::Command(command));
        }
    }

    /// Update the hovered state of a window.
    pub fn update_hovered(&mut self, window_id: WindowId) -> bool {
        let mut changed = false;

        if let Some(window_state) = self.windows.get_mut(&window_id) {
            for i in 0..window_state.window.pointers().len() {
                let pointer = &window_state.window.pointers()[i];
                let position = pointer.position;
                let hovered = window_state.scene.view_at(position);

                let pointer = &mut window_state.window.pointers_mut()[i];
                changed |= pointer.hovering != hovered;

                pointer.hovering = hovered;
            }
        }

        changed
    }

    /// Initialize the application.
    pub fn init(&mut self, data: &mut T) {
        let mut rebuild = false;
        let mut base = BaseCx::new(&mut self.contexts, &mut self.proxy);

        for delegate in &mut self.delegates {
            let mut cx = DelegateCx::new(&mut base, &mut self.requests, &mut rebuild);

            delegate.init(&mut cx, data);
        }

        if rebuild {
            self.rebuild(data);
        }
    }

    /// The application is idle.
    pub fn idle(&mut self, data: &mut T) {
        let mut rebuild = false;
        let mut base = BaseCx::new(&mut self.contexts, &mut self.proxy);

        for delegate in &mut self.delegates {
            let mut cx = DelegateCx::new(&mut base, &mut self.requests, &mut rebuild);

            delegate.idle(&mut cx, data);
        }

        if rebuild {
            self.rebuild(data);
        }
    }

    fn delegate_event(&mut self, data: &mut T, event: &Event) -> bool {
        let mut rebuild = false;
        let mut base = BaseCx::new(&mut self.contexts, &mut self.proxy);

        for delegate in &mut self.delegates {
            let mut cx = DelegateCx::new(&mut base, &mut self.requests, &mut rebuild);

            if delegate.event(&mut cx, data, event) {
                rebuild = true;
                break;
            }
        }

        if rebuild {
            self.rebuild(data);
        }

        false
    }

    fn request_window_updates(&mut self) {
        for window_state in self.windows.values() {
            let updates = window_state
                .window
                .snapshot()
                .difference(&window_state.window);

            for update in updates {
                let request = AppRequest::UpdateWindow(window_state.window.id(), update);
                self.requests.push(request);
            }
        }
    }

    /// Rebuild all windows.
    pub fn rebuild(&mut self, data: &mut T) {
        let mut base = BaseCx::new(&mut self.contexts, &mut self.proxy);

        for window_state in self.windows.values_mut() {
            (self.style).as_context(|| window_state.rebuild(data, &mut base));
        }
    }

    /// Handle an event for the entire application.
    ///
    /// Returns true if the event was handled by a delegate.
    pub fn event(&mut self, data: &mut T, event: &Event) -> bool {
        ori_core::log::trace!(event = ?event, "Event");

        // we need to animate the window before handling the event
        let animate = Instant::now();

        // we first send the event to the delegates
        let event_handled = self.delegate_event(data, event);

        let mut rebuild = false;

        // if the event was handled by a delegate we don't send it to the windows
        if !event_handled {
            for window_state in self.windows.values_mut() {
                let mut base = BaseCx::new(&mut self.contexts, &mut self.proxy);

                self.style.as_context(|| {
                    window_state.event(data, &mut base, &mut rebuild, event);
                });
            }
        }

        // rebuild the view tree if requested
        if rebuild {
            self.rebuild(data);
        }

        // update the window state after handling the event
        for window_state in self.windows.values_mut() {
            let requests = window_state.update_and_request_draw(animate);
            self.requests.extend(requests);
        }

        // handle any pending commands
        self.handle_commands(data);
        self.request_window_updates();

        event_handled
    }

    /// Handle an event for a single window.
    ///
    /// Returns true if the event was handled by a delegate.
    pub fn window_event(&mut self, data: &mut T, window_id: WindowId, event: &Event) -> bool {
        ori_core::log::trace!(event = ?event, window = ?window_id, "Window event");

        // we need to animate the window before handling the event
        let animate = Instant::now();

        // we first send the event to the delegates
        let event_handled = self.delegate_event(data, event);

        let mut rebuild = false;

        // if the event was handled by a delegate we don't send it to the window
        if !event_handled {
            if let Some(window_state) = self.windows.get_mut(&window_id) {
                let mut base = BaseCx::new(&mut self.contexts, &mut self.proxy);

                // we send the event to the window, remembering to set the style context
                self.style.as_context(|| {
                    window_state.event(data, &mut base, &mut rebuild, event);
                });
            }
        }

        // rebuild the view tree if requested
        if rebuild {
            self.rebuild(data);
        }

        // update the window state after handling the event
        if let Some(window_state) = self.windows.get_mut(&window_id) {
            let requests = window_state.update_and_request_draw(animate);
            self.requests.extend(requests);
        }

        // handle any pending commands
        self.handle_commands(data);
        self.request_window_updates();

        event_handled
    }

    // animate the window if needed
    fn animate_window(&mut self, data: &mut T, window_id: WindowId) {
        if let Some(window_state) = self.windows.get_mut(&window_id) {
            // if the window needs to animate, we send an Animate event
            if window_state.view_state.needs_animate() {
                // we need to mark the view state of the root as animated manually
                // because there is no pod around the root
                window_state.view_state.mark_animated();

                // we send an Animate event to the window, this uses the time since the last frame
                // set in either the event, window_event, or draw_window functions
                let event = Event::Animate(window_state.animate());
                self.window_event(data, window_id, &event);
            }
        }
    }

    /// Draw a single window, returning the scene if it needs to be rendered.
    pub fn draw_window(
        &mut self,
        data: &mut T,
        window_id: WindowId,
    ) -> Option<WindowRenderScene<'_>> {
        ori_core::log::trace!(window = ?window_id, "Draw window");

        // animate the window before drawing it
        //
        // this will send an Animate event if needed
        self.animate_window(data, window_id);

        // we prepare for layout and draw here
        //
        // animate is used to calculate the time since the last frame
        // and is set here so the time is as accurate as possible
        let animate = Instant::now();
        let window_state = self.windows.get_mut(&window_id)?;

        let mut base = BaseCx::new(&mut self.contexts, &mut self.proxy);

        // layout if needed
        if window_state.view_state.needs_layout() {
            (self.style).as_context(|| window_state.layout(data, &mut base));
        }

        // draw if needed
        if window_state.view_state.needs_draw() {
            self.style.as_context(|| window_state.draw(data, &mut base));

            // since hover state is determined by the scene, and since draw modifies the scene,
            // we must update the hover state, and send an UpdateHovered event if needed
            if self.update_hovered(window_id) {
                self.window_event(data, window_id, &Event::UpdateHovered);
            }
        }

        let window_state = self.windows.get_mut(&window_id)?;

        // we need to update the window state after layout and draw
        //
        // if somehow the a layout or draw has been requested we must tell the window to redraw
        let requests = window_state.update_and_request_draw(animate);
        self.requests.extend(requests);

        // handle any pending commands
        self.handle_commands(data);
        self.request_window_updates();

        let window_state = self.windows.get(&window_id)?;

        // the clear color is the palette background color, but can be overridden by the window
        let clear_color = match window_state.window.color {
            Some(color) => color,
            None => self.style.palette().background,
        };

        Some(WindowRenderScene {
            scene: &window_state.scene,
            logical_size: window_state.window.size,
            clear_color,
        })
    }
}
