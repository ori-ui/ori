use std::time::{Duration, Instant};

use crate::{
    AnyState, BaseCx, BoxedView, BuildCx, Canvas, DrawCx, Event, EventCx, LayoutCx, RebuildCx,
    Scene, SceneRender, Size, Space, Update, View, ViewState, Window,
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

    pub fn request_rebuild(&mut self) {
        self.view_state.update.insert(Update::TREE);
    }

    pub fn request_layout(&mut self) {
        self.view_state.update.insert(Update::LAYOUT | Update::DRAW);
    }

    pub fn request_draw(&mut self) {
        self.view_state.update.insert(Update::DRAW);
    }

    pub fn needs_rebuild(&self) -> bool {
        self.view_state.update.contains(Update::TREE)
    }

    pub fn needs_layout(&self) -> bool {
        self.view_state.update.contains(Update::LAYOUT)
    }

    pub fn needs_draw(&self) -> bool {
        self.view_state.update.contains(Update::DRAW)
    }

    pub fn rebuild(&mut self, base: &mut BaseCx, data: &mut T) {
        self.view_state.update.remove(Update::TREE);

        let mut new_view = (self.builder)(data);

        base.set_delta_time(self.timers.rebuild());
        let mut cx = RebuildCx::new(base, &mut self.view_state);
        new_view.rebuild(&mut self.state, &mut cx, data, &self.view);

        self.view = new_view;

        if self.needs_draw() {
            self.window.request_draw();
        }
    }

    /// Handle an event.
    ///
    /// This will rebuild or layout the view if necessary.
    pub fn event(&mut self, base: &mut BaseCx, data: &mut T, event: &Event) {
        if self.needs_rebuild() {
            self.rebuild(base, data);
        }

        if self.needs_layout() {
            self.layout(base, data);
        }

        base.set_delta_time(self.timers.event());
        let mut cx = EventCx::new(base, &mut self.view_state);
        self.view.event(&mut self.state, &mut cx, data, event);

        if self.needs_rebuild() {
            self.rebuild(base, data);
        }

        if self.needs_layout() {
            self.layout(base, data);
        }

        if self.needs_draw() {
            self.draw(base, data);
            self.window.request_draw();
        }
    }

    /// Layout the view.
    pub fn layout(&mut self, base: &mut BaseCx, data: &mut T) {
        self.view_state.update.remove(Update::LAYOUT);

        base.set_delta_time(self.timers.layout());
        let mut cx = LayoutCx::new(base, &mut self.view_state);
        let size = self.view.layout(
            &mut self.state,
            &mut cx,
            data,
            Space::new(Size::ZERO, self.window.size()),
        );
        self.view_state.size = size;

        if self.needs_draw() {
            self.window.request_draw();
        }
    }

    /// Draw the view.
    pub fn draw(&mut self, base: &mut BaseCx, data: &mut T) {
        self.view_state.update.remove(Update::DRAW);

        self.scene.clear();
        let mut canvas = Canvas::new(&mut self.scene, self.window.size());

        base.set_delta_time(self.timers.draw());
        let mut cx = DrawCx::new(base, &mut self.view_state);
        self.view.draw(&mut self.state, &mut cx, data, &mut canvas);

        if self.needs_draw() {
            self.window.request_draw();
        }
    }

    /// Render the scene.
    ///
    /// This will rebuild, layout or draw the view if necessary.
    pub fn render(&mut self, base: &mut BaseCx, data: &mut T) {
        if self.needs_rebuild() {
            self.rebuild(base, data);
        }

        if self.needs_layout() {
            self.layout(base, data);
        }

        if self.needs_draw() {
            self.draw(base, data);
        }

        let width = self.window.width();
        let height = self.window.height();
        self.render.render_scene(&mut self.scene, width, height);
    }
}
