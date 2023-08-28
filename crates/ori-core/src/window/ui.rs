use std::time::{Duration, Instant};

use crate::{
    canvas::{Canvas, Scene, SceneRender},
    event::Event,
    layout::{Size, Space},
    theme::{Palette, Theme},
    view::{
        BaseCx, BoxedView, BuildCx, Content, DrawCx, EventCx, LayoutCx, RebuildCx, State, View,
        ViewState,
    },
};

use super::{Cursor, Window};

/// A type that can build a view.
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
    view: Content<BoxedView<T>>,
    state: State<T, BoxedView<T>>,
    scene: Scene,
    theme: Theme,
    view_state: ViewState,
    needs_rebuild: bool,
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
        mut theme: Theme,
        mut window: Window,
        render: R,
    ) -> Self {
        // we first build the view tree, with `theme` as the global theme
        let view = Theme::with_global(&mut theme, || builder(data));
        let mut view = Content::new(view);

        // then we build the state tree, with `theme` as the global theme
        let mut cx = BuildCx::new(base, &mut window);
        let state = Theme::with_global(&mut theme, || view.build(&mut cx, data));

        Self {
            builder,
            view,
            state,
            scene: Scene::new(),
            theme,
            view_state: ViewState::default(),
            needs_rebuild: false,
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

    /// Set the theme.
    ///
    /// This will also request a rebuild of the view-tree, as the theme
    /// is _very_ likely to affect the view-tree.
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
        self.request_rebuild();
    }

    /// Get the scene.
    pub fn scene(&self) -> &Scene {
        &self.scene
    }

    /// Get whether the view-tree needs to be rebuilt.
    pub fn needs_rebuild(&self) -> bool {
        self.needs_rebuild
    }

    /// Get whether the view-tree needs to be laid out.
    pub fn needs_layout(&self) -> bool {
        self.view_state.needs_layout()
    }

    /// Get whether the view-tree needs to be drawn.
    pub fn needs_draw(&self) -> bool {
        self.view_state.needs_draw()
    }

    /// Request a rebuild of the view-tree.
    pub fn request_rebuild(&mut self) {
        self.window.request_draw();
        self.needs_rebuild = true;
    }

    /// Request a layout of the view-tree.
    pub fn request_layout(&mut self) {
        self.view_state.request_layout();
    }

    /// Called when the application is idle.
    pub fn idle(&mut self) {
        self.render.idle();
    }

    fn rebuild(&mut self, base: &mut BaseCx, data: &mut T) {
        self.view_state.prepare();

        // mark the window as not needing to rebuilt
        self.needs_rebuild = false;

        // build the new view tree
        let new_view = Theme::with_global(&mut self.theme, || (self.builder)(data));
        let mut new_view = Content::new(new_view);

        let dt = self.timers.rebuild();

        let mut cx = RebuildCx::new(base, &mut self.view_state, &mut self.window, dt);

        // rebuild the new view tree (new_view) comparing it to the old one (self.view)
        Theme::with_global(&mut self.theme, || {
            new_view.rebuild(&mut self.state, &mut cx, data, &self.view);
        });

        // replace the old view tree with the new one
        self.view = new_view;
    }

    /// Handle an event.
    ///
    /// This will rebuild or layout the view if necessary.
    pub fn event(&mut self, base: &mut BaseCx, data: &mut T, event: &Event) {
        // if the view tree needs to be rebuilt, we do that first, as the
        // event and layout might depend on the new view tree
        if self.needs_rebuild() {
            self.rebuild(base, data);
        }

        // if the view tree needs to be laid out, we do that first, as the
        // event might depend on the new layout
        if self.needs_layout() {
            self.layout(base, data);
        }

        self.view_state.prepare();

        let dt = self.timers.event();

        let mut cx = EventCx::new(base, &mut self.view_state, &mut self.window, dt);

        // handle the event, with the global theme
        Theme::with_global(&mut self.theme, || {
            self.view.event(&mut self.state, &mut cx, data, event);
        });

        if !self.view_state.has_cursor {
            self.window_mut().set_cursor(Cursor::default());
        }

        // if anything needs to be updated after the event, we request a draw
        //
        // FIXME: this will sometimes cause unnecessary re-renders
        if !self.view_state.update.is_empty() {
            self.window.request_draw();
        }
    }

    fn layout(&mut self, base: &mut BaseCx, data: &mut T) {
        self.view_state.prepare();

        // mark the view tree as not needing to be laid out
        self.view_state.layed_out();

        let space = Space::new(Size::ZERO, self.window.size());
        let dt = self.timers.layout();

        let mut cx = LayoutCx::new(base, &mut self.view_state, &mut self.window, dt);

        // layout the view tree, with the global theme
        let size = Theme::with_global(&mut self.theme, || {
            self.view.layout(&mut self.state, &mut cx, data, space)
        });

        self.view_state.size = size;
    }

    fn draw(&mut self, base: &mut BaseCx, data: &mut T) {
        self.view_state.prepare();

        // mark the view tree as not needing to be drawn
        self.view_state.drawn();

        // clear the scene and prepare the canvas
        self.scene.clear();
        let mut canvas = Canvas::new(&mut self.scene, self.window.size());

        let dt = self.timers.draw();

        let mut cx = DrawCx::new(base, &mut self.view_state, &mut self.window, dt);

        // draw the view tree, with the global theme
        Theme::with_global(&mut self.theme, || {
            self.view.draw(&mut self.state, &mut cx, data, &mut canvas);
        });

        if !self.view_state.has_cursor {
            self.window_mut().set_cursor(Cursor::default());
        }
    }

    /// Render the scene.
    ///
    /// This will rebuild, layout or draw the view if necessary.
    pub fn render(&mut self, base: &mut BaseCx, data: &mut T) {
        // if the view tree needs to be rebuilt, do that first, as the
        // layout and draw might depend on the new view tree
        if self.needs_rebuild() {
            self.rebuild(base, data);
        }

        // if the view tree needs to be laid out, do that first, as the
        // draw might depend on the new layout
        if self.needs_layout() {
            self.layout(base, data);
        }

        // then if the view tree needs to be drawn, do that
        if self.needs_draw() {
            self.draw(base, data);
        }

        // render the scene to the window
        let width = self.window.width();
        let height = self.window.height();
        let clear_color = self.theme.get(Palette::BACKGROUND);
        (self.render).render_scene(&mut self.scene, clear_color, width, height);

        // if anything needs to be updated after the draw, we request a drawn
        //
        // FIXME: this will sometimes cause unnecessary re-renders
        if !self.view_state.update.is_empty() {
            self.window.request_draw();
        }
    }
}
