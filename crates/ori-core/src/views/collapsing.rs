use std::f32::consts::{PI, SQRT_2};

use ori_macro::{example, Build};

use crate::{
    canvas::{Background, BorderRadius, BorderWidth, Canvas, Color, Mesh, Vertex},
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    layout::{Point, Rect, Size, Space, Vector},
    rebuild::Rebuild,
    style::{style, Style, Styles},
    transition::Transition,
    view::{Pod, State, Update, View},
};

/// Create a new [`Collapsing`].
pub fn collapsing<T, H, V>(header: H, content: V) -> Collapsing<T, H, V> {
    Collapsing::new(header, content)
}

/// The style of a collapsing view.
#[derive(Clone, Debug)]
pub struct CollapsingStyle {
    /// The transition of the view.
    pub transition: Transition,
    /// The size of the icon.
    pub icon_size: f32,
    /// The color of the icon.
    pub icon_color: Color,
    /// The background color of the header.
    pub background: Background,
    /// The border width of the header.
    pub border_width: BorderWidth,
    /// The border radius of the header.
    pub border_radius: BorderRadius,
    /// The color of the border of the header.
    pub border_color: Color,
}

impl Style for CollapsingStyle {
    fn style(style: &Styles) -> Self {
        Self {
            transition: Transition::ease(0.1),
            icon_size: 16.0,
            icon_color: style.palette().outline,
            background: Background::color(Color::TRANSPARENT),
            border_width: BorderWidth::new(0.0, 0.0, 1.0, 0.0),
            border_radius: BorderRadius::all(0.0),
            border_color: style.palette().outline_variant,
        }
    }
}

/// A collapsing view.
///
/// Can be styled using the [`CollapsingStyle`].
#[example(name = "collapsing", width = 400, height = 300)]
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
    /// Create a new [`Collapsing`] view.
    pub fn new(header: H, content: V) -> Self {
        Self::styled(header, content, style())
    }

    /// Create a new [`Collapsing`] view with a style.
    pub fn styled(header: H, content: V, style: CollapsingStyle) -> Self {
        Self {
            header: Pod::new(header),
            content: Pod::new(content),
            on_open: None,
            open: None,
            default_open: false,
            transition: style.transition,
            icon_size: style.icon_size,
            icon_color: style.icon_color,
            background: style.background,
            border_width: style.border_width,
            border_radius: style.border_radius,
            border_color: style.border_color,
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
                cx.animate();
            }
        }

        Rebuild::rebuild(self, cx, old);

        (self.header).rebuild(&mut state.header, cx, data, &old.header);

        // if we can't see the content, we still need to rebuild it
        // but we don't need to request either a layout or a draw
        //
        // FIXME: this is a bit of a hack, we should have a better way to do this
        let update = cx.view_state.update;
        (self.content).rebuild(&mut state.content, cx, data, &old.content);

        if self.transition.get(state.t) == 0.0 {
            cx.view_state.update = update & !Update::DRAW;
        }
    }

    fn event(&mut self, state: &mut Self::State, cx: &mut EventCx, data: &mut T, event: &Event) {
        self.header.event(&mut state.header, cx, data, event);

        // same deal as in rebuild
        let update = cx.view_state.update;
        self.content.event(&mut state.content, cx, data, event);

        if self.transition.get(state.t) != 0.0 {
            cx.view_state.update = update & !Update::DRAW;
        }

        if matches!(event, Event::PointerPressed(_)) && state.header.has_hot() {
            state.open = !state.open;
            cx.animate();
            cx.request_layout();
        }

        if let Event::Animate(dt) = event {
            if self.transition.step(&mut state.t, state.open, *dt) {
                cx.animate();
                cx.request_layout();
            }
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

        let header_space = space.loosen_height() - Size::new(self.icon_size, 0.0);
        let header_size = (self.header).layout(&mut state.header, cx, data, header_space);

        let header_width = header_size.width + self.icon_size;
        let header_height = header_size.height.max(self.icon_size);

        let header_offset = (header_height - header_size.height) / 2.0;
        (state.header).translate(Vector::new(self.icon_size, header_offset));

        let content_space = space - Size::new(0.0, header_size.height);
        let content_size = (self.content).layout(&mut state.content, cx, data, content_space);

        (state.content).translate(Vector::new(0.0, header_size.height));

        let width = f32::max(header_width, content_size.width);
        let height = header_height + content_size.height * t;

        Size::new(width, height)
    }

    fn draw(
        &mut self,
        state: &mut Self::State,
        cx: &mut DrawCx,
        data: &mut T,
        canvas: &mut Canvas,
    ) {
        let t = self.transition.get(state.t);

        let header_height = self.icon_size.max(state.header.size().height);
        let header_size = Size::new(cx.rect().width(), header_height);
        let header_rect = Rect::min_size(cx.rect().top_left(), header_size);

        canvas.trigger(state.header.id(), header_rect);

        canvas.draw_quad(
            header_rect,
            self.background.clone(),
            self.border_radius,
            self.border_width,
            self.border_color,
        );

        canvas.forked(|canvas| {
            canvas.translate(Vector::new(self.icon_size / 2.0, header_height / 2.0));
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
