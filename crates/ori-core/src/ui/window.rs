use std::time::Instant;

use crate::{
    canvas::{Canvas, Color, Scene},
    debug::{BuildItem, DrawItem, EventItem, History, LayoutItem, RebuildItem},
    event::{AnimationFrame, Event},
    layout::{Size, Space},
    theme::{Palette, Theme},
    view::{
        BaseCx, BoxedView, BuildCx, DrawCx, EventCx, LayoutCx, Pod, RebuildCx, State, View,
        ViewState,
    },
    window::Window,
};

use super::{UiRequest, UiRequests};

/// A type that can build a view.
pub type UiBuilder<T> = Box<dyn FnMut(&mut T) -> BoxedView<T>>;

/// User interface for a single window.
pub struct WindowUi<T> {
    builder: UiBuilder<T>,
    view: Pod<BoxedView<T>>,
    state: State<T, BoxedView<T>>,
    scene: Scene,
    theme: Theme,
    view_state: ViewState,
    needs_rebuild: bool,
    animation_frame: Option<Instant>,
    window: Window,
}

impl<T> WindowUi<T> {
    /// Create a new [´WindowUi´] for the given window.
    pub fn new(
        mut builder: UiBuilder<T>,
        base: &mut BaseCx,
        data: &mut T,
        mut theme: Theme,
        mut window: Window,
    ) -> Self {
        let mut view_state = ViewState::default();
        let mut animation_frame = None;
        let mut cx = BuildCx::new(base, &mut view_state, &mut window, &mut animation_frame);

        // we build the view tree and state tree, with the global theme
        let (view, state) = theme.as_context(|| {
            let start = Instant::now();

            let mut view = Pod::new(builder(data));

            let item = BuildItem::new(start, cx.window().id());
            cx.context_mut::<History>().push(item);

            let state = view.build(&mut cx, data);

            (view, state)
        });

        Self {
            builder,
            view,
            state,
            scene: Scene::new(),
            theme,
            view_state,
            needs_rebuild: false,
            animation_frame,
            window,
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

    /// Get the theme.
    pub fn theme(&self) -> &Theme {
        &self.theme
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

    /// Get the scene.
    pub fn scene_mut(&mut self) -> &mut Scene {
        &mut self.scene
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

    /// Get whether the window needs to be re-drawn.
    pub fn needs_redraw(&self) -> bool {
        !self.view_state.update.is_empty() || self.animation_frame.is_some()
    }

    fn redraw_requests(&self) -> UiRequests<T> {
        if !self.view_state.update.is_empty() || self.animation_frame.is_some() {
            UiRequests::one(UiRequest::RedrawWindow(self.window.id()))
        } else {
            UiRequests::new()
        }
    }

    /// Get the background color of the window.
    pub fn color(&self) -> Color {
        if let Some(color) = self.window.color() {
            color
        } else {
            self.theme().get(Palette::BACKGROUND)
        }
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

    fn update(&mut self) {
        let cursor = self.view_state.cursor().unwrap_or_default();
        self.window.set_cursor(cursor);
    }

    fn rebuild(&mut self, base: &mut BaseCx, data: &mut T) {
        self.view_state.prepare();

        // mark the window as not needing to rebuilt
        self.needs_rebuild = false;

        let mut cx = RebuildCx::new(
            base,
            &mut self.view_state,
            &mut self.window,
            &mut self.animation_frame,
        );

        // rebuild the new view tree (new_view) comparing it to the old one (self.view)
        let new_view = self.theme.as_context(|| {
            let start = Instant::now();

            let mut new_view = Pod::new((self.builder)(data));

            let item = BuildItem::new(start, cx.window().id());
            cx.context_mut::<History>().push(item);

            let start = Instant::now();

            new_view.rebuild(&mut self.state, &mut cx, data, &self.view);

            let item = RebuildItem::new(start, cx.window().id());
            cx.context_mut::<History>().push(item);

            new_view
        });

        // replace the old view tree with the new one
        self.view = new_view;
    }

    /// Handle an event.
    ///
    /// This will rebuild or layout the view if necessary.
    pub fn event(&mut self, base: &mut BaseCx, data: &mut T, event: &Event) -> UiRequests<T> {
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

        let mut cx = EventCx::new(
            base,
            &mut self.view_state,
            &mut self.window,
            &mut self.animation_frame,
        );

        let start = Instant::now();

        // handle the event, with the global theme
        self.theme.as_context(|| {
            self.view.event(&mut self.state, &mut cx, data, event);
        });

        let item = EventItem::new(start, cx.window().id(), event);
        cx.context_mut::<History>().push(item);

        self.update();
        self.redraw_requests()
    }

    fn layout(&mut self, base: &mut BaseCx, data: &mut T) {
        self.view_state.prepare();

        // mark the view tree as not needing to be laid out
        self.view_state.mark_layed_out();

        let space = Space::new(Size::ZERO, self.window.size());

        let mut cx = LayoutCx::new(
            base,
            &mut self.view_state,
            &mut self.window,
            &mut self.animation_frame,
        );

        let start = Instant::now();

        // layout the view tree, with the global theme
        let size = self.theme.as_context(|| {
            // have a comment here to make rustfmt do what i want
            self.view.layout(&mut self.state, &mut cx, data, space)
        });

        let item = LayoutItem::new(start, cx.window().id());
        cx.context_mut::<History>().push(item);

        self.view_state.size = size;
    }

    fn draw(&mut self, base: &mut BaseCx, data: &mut T) {
        self.view_state.prepare();

        // mark the view tree as not needing to be drawn
        self.view_state.mark_drawn();

        // clear the scene and prepare the canvas
        self.scene.clear();
        let mut canvas = Canvas::new(&mut self.scene, self.window.size());

        let mut cx = DrawCx::new(
            base,
            &mut self.view_state,
            &mut self.window,
            &mut self.animation_frame,
        );

        let start = Instant::now();

        // draw the view tree, with the global theme
        self.theme.as_context(|| {
            self.view.draw(&mut self.state, &mut cx, data, &mut canvas);
        });

        let item = DrawItem::new(start, cx.window().id());
        cx.context_mut::<History>().push(item);
    }

    /// Render the scene.
    ///
    /// This will rebuild, layout or draw the view if necessary.
    pub fn render(&mut self, base: &mut BaseCx, data: &mut T) -> UiRequests<T> {
        if let Some(animation_frame) = self.animation_frame.take() {
            let dt = animation_frame.elapsed().as_secs_f32();
            let event = Event::new(AnimationFrame(dt));
            let _ = self.event(base, data, &event);
        }

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

        self.update();
        self.redraw_requests()
    }
}
