use std::{any::Any, collections::HashMap};

use instant::Instant;
use ori_core::{
    canvas::{Canvas, Color},
    command::{CommandProxy, CommandReceiver},
    context::{BaseCx, BuildCx, Contexts, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::{
        CloseRequested, Code, Event, Ime, Key, KeyPressed, KeyReleased, Modifiers, PointerButton,
        PointerId, PointerLeft, PointerMoved, PointerPressed, PointerReleased, PointerScrolled,
        WindowMaximized, WindowResized, WindowScaled,
    },
    layout::{Point, Size, Space, Vector},
    log::trace,
    style::{Styles, BACKGROUND},
    view::{any, AnyState, BoxedView, View, ViewState},
    views::opaque,
    window::{Cursor, Window, WindowId, WindowSizing, WindowSnapshot, WindowUpdate},
};

use crate::{AppBuilder, AppCommand, AppRequest, Delegate, DelegateCx, UiBuilder};

/// Information needed to render a window.
pub struct WindowRenderState<'a> {
    /// The canvas to render.
    pub canvas: &'a Canvas,

    /// The size of the window.
    pub logical_size: Size,

    /// The clear color of the window.
    pub clear_color: Color,
}

pub(crate) struct WindowState<T> {
    ui: UiBuilder<T>,
    view: BoxedView<T>,
    cursor: Cursor,
    ime: Option<Ime>,
    state: AnyState,
    canvas: Canvas,
    view_state: ViewState,
    window: Window,
    snapshot: WindowSnapshot,
    animate: Option<Instant>,
}

impl<T> WindowState<T> {
    fn rebuild(&mut self, data: &mut T, base: &mut BaseCx) {
        let t = Instant::now();

        self.view_state.prepare();

        let mut cx = RebuildCx::new(base, &mut self.view_state);

        let mut new_view = (self.ui)(data);

        cx.insert_context(self.window.clone());
        new_view.rebuild(&mut self.state, &mut cx, data, &self.view);
        self.window = cx.remove_context().expect("Window context missing");

        self.view = new_view;

        trace!(
            window = ?self.window.id(),
            elapsed = ?t.elapsed(),
            "Window rebuilt"
        );
    }

    fn event(&mut self, data: &mut T, base: &mut BaseCx, rebuild: &mut bool, event: &Event) {
        let t = Instant::now();

        let hot = self.window.is_hovered(self.view_state.id());
        self.view_state.set_hot(hot);
        self.view_state.prepare();

        let mut cx = EventCx::new(base, &mut self.view_state, rebuild);

        cx.insert_context(self.window.clone());
        self.view.event(&mut self.state, &mut cx, data, event);
        self.window = cx.remove_context().expect("Window context missing");

        trace!(
            window = ?self.window.id(),
            elapsed = ?t.elapsed(),
            "Window event"
        );
    }

    fn layout(&mut self, data: &mut T, base: &mut BaseCx) {
        let t = Instant::now();

        self.view_state.mark_layed_out();

        // we need to calculate the max size of the window
        // depending on the sizing of the window
        let max_size = match self.window.sizing {
            WindowSizing::Fixed => self.window.size,
            WindowSizing::Content => Size::INFINITY,
        };

        let space = Space::new(Size::ZERO, max_size);
        let mut cx = LayoutCx::new(base, &mut self.view_state);

        cx.insert_context(self.window.clone());
        let size = self.view.layout(&mut self.state, &mut cx, data, space);
        self.window = cx.remove_context().expect("Window context missing");

        self.view_state.set_size(size);

        // if the window is content sized we set the
        // window size to the content size
        if let WindowSizing::Content = self.window.sizing {
            if size.is_infinite() {
                ori_core::log::warn!("Window content size is non-finite.");
            }

            self.window.size = size;
        }

        trace!(
            window = ?self.window.id(),
            elapsed = ?t.elapsed(),
            "Window layout"
        );
    }

    fn draw(&mut self, data: &mut T, base: &mut BaseCx) {
        let t = Instant::now();

        self.view_state.mark_drawn();

        self.canvas.clear();

        let mut cx = DrawCx::new(base, &mut self.view_state, &mut self.canvas);

        cx.insert_context(self.window.clone());
        self.view.draw(&mut self.state, &mut cx, data);
        self.window = cx.remove_context().expect("Window context missing");

        trace!(
            window = ?self.window.id(),
            elapsed = ?t.elapsed(),
            "Window draw"
        );
    }

    fn animate(&mut self, animate: Instant) -> Vec<AppRequest<T>> {
        if self.view_state.needs_animate() && self.animate.is_none() {
            self.animate = Some(animate);
            return vec![AppRequest::RequestRedraw(self.window.id())];
        }

        Vec::new()
    }
}

/// The main application state.
pub struct App<T> {
    pub(crate) windows: HashMap<WindowId, WindowState<T>>,
    pub(crate) modifiers: Modifiers,
    pub(crate) delegates: Vec<Box<dyn Delegate<T>>>,
    pub(crate) proxy: CommandProxy,
    pub(crate) receiver: CommandReceiver,
    pub(crate) requests: Vec<AppRequest<T>>,
    pub(crate) contexts: Contexts,
}

impl<T> App<T> {
    /// Create a new application builder.
    pub fn build() -> AppBuilder<T> {
        AppBuilder::new()
    }

    /// A window was requested to be closed.
    ///
    /// Returns `true` if the window was closed, i.e. the event was not handled.
    pub fn close_requested(&mut self, data: &mut T, window_id: WindowId) -> bool {
        let event = Event::CloseRequested(CloseRequested { window: window_id });

        let handled = self.window_event(data, window_id, &event);

        if !handled {
            self.remove_window(window_id);
            self.requests.push(AppRequest::CloseWindow(window_id));

            if self.windows.is_empty() {
                self.requests.push(AppRequest::Quit);
            }
        }

        !handled
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

    /// A window was scaled.
    pub fn window_scaled(&mut self, data: &mut T, window_id: WindowId, scale: f32) {
        if let Some(window_state) = self.windows.get_mut(&window_id) {
            window_state.view_state.request_layout();
            window_state.window.scale = scale;
            window_state.snapshot.scale = scale;
        }

        let event = Event::WindowScaled(WindowScaled {
            window: window_id,
            scale_factor: scale,
        });

        self.window_event(data, window_id, &event);
    }

    /// The maximized state of a window changed.
    pub fn window_maximized(&mut self, data: &mut T, window_id: WindowId, maximized: bool) {
        if let Some(window_state) = self.windows.get_mut(&window_id) {
            window_state.view_state.request_layout();
            window_state.window.maximized = maximized;
            window_state.snapshot.maximized = maximized;
        }

        let event = Event::WindowMaximized(WindowMaximized {
            window: window_id,
            maximized,
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
    ) -> bool {
        let Some(window_state) = self.windows.get_mut(&window_id) else {
            return false;
        };

        let delta = window_state.window.move_pointer(pointer_id, position);
        self.update_hovered(window_id);

        let event = Event::PointerMoved(PointerMoved {
            id: pointer_id,
            modifiers: self.modifiers,
            position,
            delta,
        });

        self.window_event(data, window_id, &event)
    }

    /// A pointer left the window.
    pub fn pointer_left(&mut self, data: &mut T, window_id: WindowId, pointer_id: PointerId) {
        let Some(window_state) = self.windows.get_mut(&window_id) else {
            return;
        };

        window_state.window.remove_pointer(pointer_id);

        let event = Event::PointerLeft(PointerLeft { id: pointer_id });

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
    ) -> bool {
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

            self.window_event(data, window_id, &event)
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

            self.window_event(data, window_id, &event)
        }
    }

    /// A keyboard key was pressed or released.
    pub fn keyboard_key(
        &mut self,
        data: &mut T,
        window_id: WindowId,
        key: Key,
        code: Option<Code>,
        text: Option<String>,
        pressed: bool,
    ) {
        if pressed {
            let event = Event::KeyPressed(KeyPressed {
                key,
                code,
                text,
                modifiers: self.modifiers,
            });

            self.window_event(data, window_id, &event);
        } else {
            let event = Event::KeyReleased(KeyReleased {
                key,
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
        let mut view = ui(data);
        let mut view_state = ViewState::default();

        let mut base = BaseCx::new(&mut self.contexts, &mut self.proxy);

        let snapshot = window.snapshot();

        let mut cx = BuildCx::new(&mut base, &mut view_state);

        cx.insert_context(window.clone());
        let state = view.build(&mut cx, data);
        window = cx.remove_context().expect("Window context missing");

        let window_id = window.id();
        let window_state = WindowState {
            ui,
            view,
            cursor: Cursor::Default,
            ime: None,
            state,
            canvas: Canvas::new(),
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

    /// Add a context.
    pub fn add_context(&mut self, context: impl Any) {
        self.contexts.insert(context);
    }

    /// Take all pending requests.
    pub fn take_requests(&mut self) -> impl Iterator<Item = AppRequest<T>> {
        std::mem::take(&mut self.requests).into_iter()
    }

    fn handle_app_command(&mut self, data: &mut T, command: AppCommand) {
        match command {
            AppCommand::OpenWindow(window, mut ui) => {
                let builder: UiBuilder<T> = Box::new(move |_| any(opaque(ui())));
                self.add_window(data, builder, window);
            }
            AppCommand::CloseWindow(window_id) => {
                self.requests.push(AppRequest::CloseWindow(window_id));
            }
            AppCommand::DragWindow(window_id) => {
                self.requests.push(AppRequest::DragWindow(window_id));
            }
            AppCommand::Quit => {
                self.requests.push(AppRequest::Quit);
            }
        }
    }

    /// Handle all pending commands.
    pub fn handle_commands(&mut self, data: &mut T) {
        while let Some(command) = self.receiver.try_recv() {
            // if the command is an AppCommand we handle it here
            if command.is::<AppCommand>() {
                let app_command = command.to_any().downcast().unwrap();
                self.handle_app_command(data, *app_command);

                continue;
            }

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
                let hovered = window_state.canvas.view_at(position);

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
            self.handle_window_requests();
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
            self.handle_window_requests();
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

    fn handle_window_requests(&mut self) {
        for window_state in self.windows.values_mut() {
            let id = window_state.window.id();

            let updates = window_state.snapshot.difference(&window_state.window);
            window_state.snapshot = window_state.window.snapshot();

            for update in updates {
                self.requests.push(AppRequest::UpdateWindow(id, update));
            }

            if window_state.view_state.needs_draw()
                || window_state.view_state.needs_layout()
                || window_state.view_state.needs_animate()
            {
                self.requests.push(AppRequest::RequestRedraw(id));
            }

            let cursor = window_state.view_state.cursor().unwrap_or_default();
            if window_state.cursor != cursor {
                let update = WindowUpdate::Cursor(cursor);
                self.requests.push(AppRequest::UpdateWindow(id, update));

                window_state.cursor = cursor;
            }

            if window_state.ime.as_ref() != window_state.view_state.ime() {
                let update = WindowUpdate::Ime(window_state.view_state.ime().cloned());
                self.requests.push(AppRequest::UpdateWindow(id, update));

                window_state.ime = window_state.view_state.ime().cloned();
            }
        }
    }

    /// Rebuild all windows.
    pub fn rebuild(&mut self, data: &mut T) {
        let mut base = BaseCx::new(&mut self.contexts, &mut self.proxy);

        for window_state in self.windows.values_mut() {
            window_state.rebuild(data, &mut base);
        }
    }

    /// Handle an event for the entire application.
    ///
    /// Returns true if the event was handled by a delegate.
    pub fn event(&mut self, data: &mut T, event: &Event) -> bool {
        trace!(event = ?event, "Event");

        // we need to animate the window before handling the event
        let animate = Instant::now();

        // we first send the event to the delegates
        let event_handled = self.delegate_event(data, event);

        let mut rebuild = false;

        // if the event was handled by a delegate we don't send it to the windows
        if !event_handled {
            for window_state in self.windows.values_mut() {
                let mut base = BaseCx::new(&mut self.contexts, &mut self.proxy);

                window_state.event(data, &mut base, &mut rebuild, event);
            }
        }

        // rebuild the view tree if requested
        if rebuild {
            self.rebuild(data);
        }

        // update the window state after handling the event
        for window_state in self.windows.values_mut() {
            let requests = window_state.animate(animate);
            self.requests.extend(requests);
        }

        // handle any pending commands
        self.handle_commands(data);
        self.handle_window_requests();

        event_handled
    }

    /// Handle an event for a single window.
    ///
    /// Returns true if the event was handled by a delegate.
    pub fn window_event(&mut self, data: &mut T, window_id: WindowId, event: &Event) -> bool {
        trace!(event = ?event, window = ?window_id, "Window event");

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
                window_state.event(data, &mut base, &mut rebuild, event);
            }
        }

        // rebuild the view tree if requested
        if rebuild {
            self.rebuild(data);
        }

        // update the window state after handling the event
        if let Some(window_state) = self.windows.get_mut(&window_id) {
            let requests = window_state.animate(animate);
            self.requests.extend(requests);
        }

        // handle any pending commands
        self.handle_commands(data);
        self.handle_window_requests();

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

                let delta_time = match window_state.animate.take() {
                    Some(t) => t.elapsed().as_secs_f32(),
                    None => 0.0,
                };

                // we send an Animate event to the window, this uses the time since the last frame
                // set in either the event, window_event, or draw_window functions
                let event = Event::Animate(delta_time);
                self.window_event(data, window_id, &event);
            }
        }
    }

    /// Draw a single window, returning the scene if it needs to be rendered.
    pub fn draw_window(
        &mut self,
        data: &mut T,
        window_id: WindowId,
    ) -> Option<WindowRenderState<'_>> {
        trace!(window = ?window_id, "Draw window");

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
            window_state.layout(data, &mut base);
        }

        // draw if needed
        if window_state.view_state.needs_draw() {
            window_state.draw(data, &mut base);

            // since hover state is determined by the scene, and since draw modifies the scene,
            // we must update the hover state, and send an UpdateHovered event if needed
            if self.update_hovered(window_id) {
                self.window_event(data, window_id, &Event::Update);
            }
        }

        let window_state = self.windows.get_mut(&window_id)?;

        // we need to update the window state after layout and draw
        //
        // if somehow the a layout or draw has been requested we must tell the window to redraw
        let requests = window_state.animate(animate);
        self.requests.extend(requests);

        // handle any pending commands
        self.handle_commands(data);
        self.handle_window_requests();

        let window_state = self.windows.get(&window_id)?;

        // the clear color is the palette background color, but can be overridden by the window
        let clear_color = match window_state.window.color {
            Some(color) => color,
            None => {
                let styles = (self.contexts.get::<Styles>()).expect("app has styles context");
                styles.get_or(Color::WHITE, BACKGROUND)
            }
        };

        Some(WindowRenderState {
            canvas: &window_state.canvas,
            logical_size: window_state.window.size,
            clear_color,
        })
    }
}
