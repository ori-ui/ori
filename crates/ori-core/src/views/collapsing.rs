use std::f32::consts::{PI, SQRT_2};

use ori_macro::{example, Build};

use crate::{
    canvas::{BorderRadius, BorderWidth, Color, Curve, FillRule},
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::{Event, PointerButton},
    layout::{Affine, Point, Rect, Size, Space, Vector},
    rebuild::Rebuild,
    style::{Stylable, Style, StyleBuilder, Theme},
    transition::Transition,
    view::{Pod, PodState, View},
};

/// Create a new [`Collapsing`].
pub fn collapsing<T, H, V>(header: H, content: V) -> Collapsing<T, H, V> {
    Collapsing::new(header, content)
}

/// The style of a collapsing view.
#[derive(Clone, Rebuild)]
pub struct CollapsingStyle {
    /// The transition of the view.
    #[rebuild(draw)]
    pub transition: Transition,

    /// The size of the icon.
    #[rebuild(layout)]
    pub icon_size: f32,

    /// The color of the icon.
    #[rebuild(draw)]
    pub icon_color: Color,

    /// The background color of the header.
    #[rebuild(draw)]
    pub background: Color,

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

impl Style for CollapsingStyle {
    fn default_style() -> StyleBuilder<Self> {
        StyleBuilder::new(|theme: &Theme| CollapsingStyle {
            transition: Transition::ease(0.1),
            icon_size: 16.0,
            icon_color: theme.primary,
            background: Color::TRANSPARENT,
            border_width: BorderWidth::new(0.0, 0.0, 1.0, 0.0),
            border_radius: BorderRadius::all(0.0),
            border_color: theme.outline,
        })
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
    pub transition: Option<Transition>,

    /// The size of the icon.
    pub icon_size: Option<f32>,

    /// The color of the icon.
    pub icon_color: Option<Color>,

    /// The background color of the header.
    pub background: Option<Color>,

    /// The border width of the header.
    pub border_width: Option<BorderWidth>,

    /// The border radius of the header.
    pub border_radius: Option<BorderRadius>,

    /// The color of the border of the header.
    pub border_color: Option<Color>,
}

impl<T, H, V> Collapsing<T, H, V> {
    /// Create a new [`Collapsing`] view.
    pub fn new(header: H, content: V) -> Self {
        Self {
            header: Pod::new(header),
            content: Pod::new(content),
            on_open: None,
            open: None,
            default_open: false,
            transition: None,
            icon_size: None,
            icon_color: None,
            background: None,
            border_width: None,
            border_radius: None,
            border_color: None,
        }
    }

    /// Set a callback for when the view is opened or closed.
    pub fn on_open(mut self, on_open: impl FnMut(&mut EventCx, &mut T, bool) + 'static) -> Self {
        self.on_open = Some(Box::new(on_open));
        self
    }
}

impl<T, H, V> Stylable for Collapsing<T, H, V> {
    type Style = CollapsingStyle;

    fn style(&self, style: &Self::Style) -> Self::Style {
        CollapsingStyle {
            transition: self.transition.unwrap_or(style.transition),
            icon_size: self.icon_size.unwrap_or(style.icon_size),
            icon_color: self.icon_color.unwrap_or(style.icon_color),
            background: self.background.unwrap_or(style.background),
            border_width: self.border_width.unwrap_or(style.border_width),
            border_radius: self.border_radius.unwrap_or(style.border_radius),
            border_color: self.border_color.unwrap_or(style.border_color),
        }
    }
}

#[doc(hidden)]
pub struct CollapsingState<T, H: View<T>, V: View<T>> {
    style: CollapsingStyle,
    header: PodState<T, H>,
    content: PodState<T, V>,
    open: bool,
    t: f32,
}

impl<T, H: View<T>, V: View<T>> View<T> for Collapsing<T, H, V> {
    type State = CollapsingState<T, H, V>;

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        let open = self.open.unwrap_or(self.default_open);

        CollapsingState {
            style: self.style(cx.style()),
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
        self.rebuild_style(cx, &mut state.style);

        (self.header).rebuild(&mut state.header, cx, data, &old.header);
        (self.content).rebuild(&mut state.content, cx, data, &old.content);
    }

    fn event(
        &mut self,
        state: &mut Self::State,
        cx: &mut EventCx,
        data: &mut T,
        event: &Event,
    ) -> bool {
        let mut handled = false;

        match event {
            Event::PointerPressed(event) if state.header.has_hovered() => {
                if matches!(event.button, PointerButton::Primary) {
                    state.open = !state.open;
                    cx.animate();
                    cx.layout();

                    if let Some(ref mut on_open) = self.on_open {
                        on_open(cx, data, state.open);
                    }

                    handled = true;
                }
            }
            Event::Animate(dt) => {
                if state.style.transition.step(&mut state.t, state.open, *dt) {
                    cx.animate();
                    cx.layout();
                }
            }
            _ => {}
        }

        handled |= (self.header).event_maybe(handled, &mut state.header, cx, data, event);
        handled |= (self.content).event_maybe(handled, &mut state.content, cx, data, event);

        handled
    }

    fn layout(
        &mut self,
        state: &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
        let t = state.style.transition.get(state.t);

        let header_space = space.loosen_height() - Size::new(state.style.icon_size, 0.0);
        let header_size = (self.header).layout(&mut state.header, cx, data, header_space);

        let header_width = header_size.width + state.style.icon_size;
        let header_height = header_size.height.max(state.style.icon_size);

        let header_offset = (header_height - header_size.height) / 2.0;
        (state.header).translate(Vector::new(state.style.icon_size, header_offset));

        let content_space = space - Size::new(0.0, header_size.height);
        let content_size = (self.content).layout(&mut state.content, cx, data, content_space);

        (state.content).translate(Vector::new(0.0, header_size.height));

        let width = f32::max(header_width, content_size.width);
        let height = header_height + content_size.height * t;

        Size::new(width, height)
    }

    fn draw(&mut self, state: &mut Self::State, cx: &mut DrawCx, data: &mut T) {
        let t = state.style.transition.get(state.t);

        let header_height = state.style.icon_size.max(state.header.size().height);
        let header_size = Size::new(cx.rect().width(), header_height);
        let header_rect = Rect::min_size(cx.rect().top_left(), header_size);

        cx.quad(
            header_rect,
            state.style.background,
            state.style.border_radius,
            state.style.border_width,
            state.style.border_color,
        );

        let mut transform = Affine::translate(Vector::new(
            state.style.icon_size / 2.0,
            header_height / 2.0,
        ));

        transform *= Affine::scale(Vector::all(state.style.icon_size));
        transform *= Affine::rotate(PI / 2.0 * t);

        cx.transformed(transform, |cx| {
            cx.fill(icon(), FillRule::EvenOdd, state.style.icon_color);
        });

        cx.canvas().trigger(state.header.rect(), state.header.id());
        self.header.draw(&mut state.header, cx, data);

        let content_offset = Vector::new(0.0, state.header.size().height);
        let content_height = Vector::new(0.0, state.content.size().height) * (1.0 - t);
        state.content.translate(content_offset - content_height);

        let content_min = cx.rect().top_left() + content_offset;
        let content_rect = Rect::min_size(content_min, state.content.size());

        cx.masked(content_rect, |cx| {
            self.content.draw(&mut state.content, cx, data);
        });
    }
}

fn icon() -> Curve {
    let mut curve = Curve::new();

    let d = 0.25;
    curve.move_to(Point::new(-d, -d * SQRT_2));
    curve.line_to(Point::new(d * SQRT_2, 0.0));
    curve.line_to(Point::new(-d, d * SQRT_2));
    curve.close();

    curve
}
