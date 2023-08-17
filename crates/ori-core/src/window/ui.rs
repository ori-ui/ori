use std::{
    mem,
    time::{Duration, Instant},
};

use crate::{
    AnyState, BaseCx, BoxedView, BuildCx, Canvas, Delegate, DelegateCx, DrawCx, Event, EventCx,
    LayoutCx, RebuildCx, Scene, SceneRender, Size, Space, Update, View, ViewState, Window,
};

pub type UiBuilder<T> = Box<dyn FnMut(&mut T) -> BoxedView<T>>;

#[derive(Debug, Default)]
struct Timers {
    last_rebuild: Option<Instant>,
    last_event: Option<Instant>,
    last_layout: Option<Instant>,
    last_draw: Option<Instant>,
}

impl Timers {
    fn rebuild(&mut self) -> Duration {
        let now = Instant::now();
        let last = self.last_rebuild.unwrap_or(now);
        self.last_rebuild = Some(now);
        now.duration_since(last)
    }

    fn event(&mut self) -> Duration {
        let now = Instant::now();
        let last = self.last_event.unwrap_or(now);
        self.last_event = Some(now);
        now.duration_since(last)
    }

    fn layout(&mut self) -> Duration {
        let now = Instant::now();
        let last = self.last_layout.unwrap_or(now);
        self.last_layout = Some(now);
        now.duration_since(last)
    }

    fn draw(&mut self) -> Duration {
        let now = Instant::now();
        let last = self.last_draw.unwrap_or(now);
        self.last_draw = Some(now);
        now.duration_since(last)
    }
}

/// User interface for a single window.
pub struct WindowUi<T, R: SceneRender> {
    builder: UiBuilder<T>,
    view: BoxedView<T>,
    state: AnyState,
    scene: Scene,
    view_state: ViewState,
    window: Window,
    render: R,
    timers: Timers,
}

impl<T, R: SceneRender> WindowUi<T, R> {
    /// Create a new [´WindowUi´] for the given window.
    pub fn new(
        mut builder: UiBuilder<T>,
        base: &mut BaseCx,
        data: &mut T,
        window: Window,
        render: R,
    ) -> Self {
        let mut view = builder(data);
        let mut cx = BuildCx::new(base);
        let state = view.build(&mut cx, data);

        Self {
            builder,
            view,
            state,
            scene: Scene::new(),
            view_state: ViewState::default(),
            window,
            render,
            timers: Timers::default(),
        }
    }

    /// Get the window.
    pub fn window(&self) -> &Window {
        &self.window
    }

    /// Get the window.
    pub fn window_mut(&mut self) -> &mut Window {
        &mut self.window
    }

    /// Get the scene.
    pub fn scene(&self) -> &Scene {
        &self.scene
    }

    /// Get whether the view-tree needs to be rebuilt.
    pub fn needs_rebuild(&self) -> bool {
        self.view_state.needs_rebuild()
    }

    /// Get whether the view-tree needs to be laid out.
    pub fn needs_layout(&self) -> bool {
        self.view_state.needs_layout()
    }

    /// Get whether the view-tree needs to be drawn.
    pub fn needs_draw(&self) -> bool {
        self.view_state.needs_draw()
    }

    /// Request a layout of the view-tree.
    pub fn request_layout(&mut self) {
        self.view_state.request_layout();
    }

    fn handle_commands(&mut self, delegate: &mut dyn Delegate<T>, base: &mut BaseCx, data: &mut T) {
        for command in mem::take(base.commands) {
            let mut cx = DelegateCx::new(base, &mut self.view_state);
            delegate.event(&mut cx, data, command.event());
            self.event(delegate, base, data, command.event());
        }
    }

    fn rebuild(&mut self, delegate: &mut dyn Delegate<T>, base: &mut BaseCx, data: &mut T) {
        self.view_state.update.remove(Update::TREE);

        let mut new_view = (self.builder)(data);

        let dt = self.timers.rebuild();
        let mut cx = RebuildCx::new(base, &mut self.view_state, dt);
        new_view.rebuild(&mut self.state, &mut cx, data, &self.view);

        self.view = new_view;

        self.handle_commands(delegate, base, data);
    }

    /// Handle an event.
    ///
    /// This will rebuild or layout the view if necessary.
    pub fn event(
        &mut self,
        delegate: &mut dyn Delegate<T>,
        base: &mut BaseCx,
        data: &mut T,
        event: &Event,
    ) {
        if self.needs_rebuild() {
            self.rebuild(delegate, base, data);
        }

        if self.needs_layout() {
            self.layout(delegate, base, data);
        }

        let dt = self.timers.event();
        let mut cx = EventCx::new(base, &mut self.view_state, dt);
        self.view.event(&mut self.state, &mut cx, data, event);

        if self.needs_draw() {
            self.window.request_draw();
        }

        self.handle_commands(delegate, base, data);

        if !self.view_state.update.is_empty() {
            self.window.request_draw();
        }
    }

    fn layout(&mut self, delegate: &mut dyn Delegate<T>, base: &mut BaseCx, data: &mut T) {
        self.view_state.update.remove(Update::LAYOUT);

        let dt = self.timers.layout();
        let mut cx = LayoutCx::new(base, &mut self.view_state, dt);
        let size = self.view.layout(
            &mut self.state,
            &mut cx,
            data,
            Space::new(Size::ZERO, self.window.size()),
        );
        self.view_state.size = size;

        self.handle_commands(delegate, base, data);
    }

    fn draw(&mut self, delegate: &mut dyn Delegate<T>, base: &mut BaseCx, data: &mut T) {
        self.view_state.update.remove(Update::DRAW);

        self.scene.clear();
        let mut canvas = Canvas::new(&mut self.scene, self.window.size());

        let dt = self.timers.draw();
        let mut cx = DrawCx::new(base, &mut self.view_state, dt);
        self.view.draw(&mut self.state, &mut cx, data, &mut canvas);

        self.handle_commands(delegate, base, data);
    }

    /// Render the scene.
    ///
    /// This will rebuild, layout or draw the view if necessary.
    pub fn render(&mut self, delegate: &mut dyn Delegate<T>, base: &mut BaseCx, data: &mut T) {
        if self.needs_rebuild() {
            self.rebuild(delegate, base, data);
        }

        if self.needs_layout() {
            self.layout(delegate, base, data);
        }

        if self.needs_draw() {
            self.draw(delegate, base, data);
        }

        let width = self.window.width();
        let height = self.window.height();
        self.render.render_scene(&mut self.scene, width, height);

        if !self.view_state.update.is_empty() {
            self.window.request_draw();
        }
    }
}
