use std::f32::consts::{PI, SQRT_2};

use ori_macro::Build;

use crate::{
    canvas::{Background, BorderRadius, BorderWidth, Canvas, Color, Mesh, Vertex},
    event::{AnimationFrame, Event, PointerPressed},
    layout::{Point, Rect, Size, Space, Vector},
    rebuild::Rebuild,
    theme::{collapsing, style},
    transition::Transition,
    view::{BuildCx, DrawCx, EventCx, LayoutCx, Pod, RebuildCx, State, View},
};

/// Create a new [`Collapsing`].
pub fn collapsing<T, H, V>(header: H, content: V) -> Collapsing<T, H, V> {
    Collapsing::new(header, content)
}

/// A collapsing view.
#[derive(Build, Rebuild)]
pub struct Collapsing<T, H, V> {
    /// The header.
    #[build(ignore)]
    pub header: Pod<H>,

    /// The content.
    #[build(ignore)]
    pub content: Pod<V>,

    /// A callback for when the view is opened or closed.
    #[build(ignore)]
    #[allow(clippy::type_complexity)]
    pub on_open: Option<Box<dyn FnMut(&mut EventCx, &mut T, bool)>>,

    /// Whether the view is open.
    #[rebuild(layout)]
    pub open: Option<bool>,

    /// Whether the view is open by default.
    pub default_open: bool,

    /// The transition of the view.
    pub transition: Transition,

    /// The size of the icon.
    #[rebuild(layout)]
    pub icon_size: f32,

    /// The color of the icon.
    #[rebuild(draw)]
    pub icon_color: Color,

    /// The background color of the header.
    #[rebuild(draw)]
    pub background: Background,

    /// The border width of the header.
    #[rebuild(draw)]
    pub border_width: BorderWidth,

    /// The border radius of the header.
    #[rebuild(draw)]
    pub border_radius: BorderRadius,

    /// The color of the border of the header.
    #[rebuild(draw)]
    pub border_color: Color,
}

impl<T, H, V> Collapsing<T, H, V> {
    /// Create a new collapsing view.
    pub fn new(header: H, content: V) -> Self {
        Self {
            header: Pod::new(header),
            content: Pod::new(content),
            on_open: None,
            open: None,
            default_open: false,
            transition: style(collapsing::TRANSITION),
            icon_size: style(collapsing::ICON_SIZE),
            icon_color: style(collapsing::ICON_COLOR),
            background: style(collapsing::BACKGROUND),
            border_width: style(collapsing::BORDER_WIDTH),
            border_radius: style(collapsing::BORDER_RADIUS),
            border_color: style(collapsing::BORDER_COLOR),
        }
    }

    /// Set a callback for when the view is opened or closed.
    pub fn on_open(mut self, on_open: impl FnMut(&mut EventCx, &mut T, bool) + 'static) -> Self {
        self.on_open = Some(Box::new(on_open));
        self
    }
}

#[doc(hidden)]
pub struct CollapsingState<T, H: View<T>, V: View<T>> {
    pub header: State<T, H>,
    pub content: State<T, V>,
    pub open: bool,
    pub t: f32,
}

impl<T, H: View<T>, V: View<T>> View<T> for Collapsing<T, H, V> {
    type State = CollapsingState<T, H, V>;

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        let open = self.open.unwrap_or(self.default_open);

        CollapsingState {
            header: self.header.build(cx, data),
            content: self.content.build(cx, data),
            open,
            t: open as u32 as f32,
        }
    }

    fn rebuild(&mut self, state: &mut Self::State, cx: &mut RebuildCx, data: &mut T, old: &Self) {
        if let Some(open) = self.open {
            if state.open != open {
                state.open = open;
                cx.request_animation_frame();
            }
        }

        Rebuild::rebuild(self, cx, old);

        (self.header).rebuild(&mut state.header, cx, data, &old.header);
        (self.content).rebuild(&mut state.content, cx, data, &old.content);
    }

    fn event(&mut self, state: &mut Self::State, cx: &mut EventCx, data: &mut T, event: &Event) {
        self.header.event(&mut state.header, cx, data, event);
        self.content.event(&mut state.content, cx, data, event);

        if event.is::<PointerPressed>() && (state.header.is_hot() || state.header.has_hot()) {
            state.open = !state.open;
            cx.request_animation_frame();
            cx.request_layout();
        }

        if let Some(AnimationFrame(dt)) = event.get() {
            if self.transition.step(&mut state.t, state.open, *dt) {
                cx.request_animation_frame();
            }

            cx.request_layout();
        }
    }

    fn layout(
        &mut self,
        state: &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
        let t = self.transition.get(state.t);

        state.header.translate(Vector::new(self.icon_size, 0.0));

        let header_space = space.loosen_height() - Size::new(self.icon_size, 0.0);
        let header_size = (self.header).layout(&mut state.header, cx, data, header_space);

        (state.content).translate(Vector::new(0.0, header_size.height));

        if t == 0.0 {
            return header_size;
        }

        let content_space = space - Size::new(0.0, header_size.height);
        let content_size = (self.content).layout(&mut state.content, cx, data, content_space);

        Size::new(
            f32::max(header_size.width, content_size.width),
            header_size.height + content_size.height * t,
        )
    }

    fn draw(
        &mut self,
        state: &mut Self::State,
        cx: &mut DrawCx,
        data: &mut T,
        canvas: &mut Canvas,
    ) {
        let t = self.transition.get(state.t);

        let header_rect = state.header.rect() + Size::new(self.icon_size, 0.0);
        canvas.trigger(state.header.id(), header_rect);

        canvas.draw_quad(
            header_rect,
            self.background.clone(),
            self.border_radius,
            self.border_width,
            self.border_color,
        );

        canvas.forked(|canvas| {
            canvas.translate(Vector::all(self.icon_size / 2.0));
            canvas.scale(Vector::all(self.icon_size));

            canvas.rotate(PI / 2.0 * t);

            canvas.draw(icon(self.icon_color));
        });

        self.header.draw(&mut state.header, cx, data, canvas);

        let content_offset = Vector::new(0.0, state.header.size().height);
        let content_height = Vector::new(0.0, state.content.size().height) * (1.0 - t);
        state.content.translate(content_offset - content_height);

        if t == 0.0 {
            return;
        }

        let content_min = cx.rect().top_left() + content_offset;
        let content_rect = Rect::min_size(content_min, state.content.size());

        canvas.clip(content_rect.transform(canvas.transform));
        self.content.draw(&mut state.content, cx, data, canvas);
    }
}

fn icon(color: Color) -> Mesh {
    let mut mesh = Mesh::new();

    let d = 0.25;

    mesh.vertices.extend([
        Vertex::new_color(Point::new(-d, -d * SQRT_2), color),
        Vertex::new_color(Point::new(d * SQRT_2, 0.0), color),
        Vertex::new_color(Point::new(-d, d * SQRT_2), color),
    ]);

    mesh.indices.extend([0, 1, 2]);

    mesh
}
