use ori_macro::{example, Build};

use crate::{
    canvas::{BorderRadius, BorderWidth, Color},
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    layout::{Padding, Size, Space, Vector},
    rebuild::Rebuild,
    style::{Stylable, Style, StyleBuilder, Theme},
    transition::Transition,
    view::{Pod, PodState, View},
};

/// Create a new [`Button`].
pub fn button<V>(view: V) -> Button<V> {
    Button::new(view)
}

/// The style of a button.
#[derive(Clone, Default, Rebuild)]
pub struct ButtonStyle {
    /// The padding.
    #[rebuild(layout)]
    pub padding: Padding,

    /// The distance of the fancy effect.
    #[rebuild(draw)]
    pub fancy: f32,

    /// The transition of the button.
    #[rebuild(draw)]
    pub transition: Transition,

    /// The color of the button.
    #[rebuild(draw)]
    pub color: Color,

    /// The border radius.
    #[rebuild(draw)]
    pub border_radius: BorderRadius,

    /// The border width.
    #[rebuild(draw)]
    pub border_width: BorderWidth,

    /// The border color.
    #[rebuild(draw)]
    pub border_color: Color,
}

impl Style for ButtonStyle {
    fn default_style() -> StyleBuilder<Self> {
        StyleBuilder::new(|theme: &Theme| ButtonStyle {
            padding: Padding::all(8.0),
            fancy: 0.0,
            transition: Transition::ease(0.1),
            color: theme.surface(2),
            border_radius: BorderRadius::all(4.0),
            border_width: BorderWidth::all(0.0),
            border_color: theme.outline,
        })
    }
}

/// A button.
///
/// Can be styled using the [`ButtonStyle`].
#[example(name = "button", width = 400, height = 300)]
#[derive(Build)]
pub struct Button<V> {
    /// The content.
    #[build(ignore)]
    pub content: Pod<V>,

    /// The padding.
    pub padding: Option<Padding>,

    /// The distance of the fancy effect.
    pub fancy: Option<f32>,

    /// The transition of the button.
    pub transition: Option<Transition>,

    /// The color of the button.
    pub color: Option<Color>,

    /// The border radius.
    pub border_radius: Option<BorderRadius>,

    /// The border width.
    pub border_width: Option<BorderWidth>,

    /// The border color.
    pub border_color: Option<Color>,
}

impl<V> Button<V> {
    /// Create a new [`Button`].
    pub fn new(content: V) -> Self {
        Self {
            content: Pod::new(content),
            padding: None,
            fancy: None,
            transition: None,
            color: None,
            border_radius: None,
            border_width: None,
            border_color: None,
        }
    }
}

impl<V> Stylable for Button<V> {
    type Style = ButtonStyle;

    fn style(&self, style: &Self::Style) -> Self::Style {
        ButtonStyle {
            padding: self.padding.unwrap_or(style.padding),
            fancy: self.fancy.unwrap_or(style.fancy),
            transition: self.transition.unwrap_or(style.transition),
            color: self.color.unwrap_or(style.color),
            border_radius: self.border_radius.unwrap_or(style.border_radius),
            border_width: self.border_width.unwrap_or(style.border_width),
            border_color: self.border_color.unwrap_or(style.border_color),
        }
    }
}

#[doc(hidden)]
pub struct ButtonState {
    pub hovered: f32,
    pub active: f32,
    pub style: ButtonStyle,
}

impl<T, V: View<T>> View<T> for Button<V> {
    type State = (ButtonState, PodState<T, V>);

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        cx.set_focusable(true);

        let state = ButtonState {
            hovered: 0.0,
            active: 0.0,
            style: self.style(cx.style()),
        };

        (state, self.content.build(cx, data))
    }

    fn rebuild(
        &mut self,
        (state, content): &mut Self::State,
        cx: &mut RebuildCx,
        data: &mut T,
        old: &Self,
    ) {
        self.rebuild_style(cx, &mut state.style);
        self.content.rebuild(content, cx, data, &old.content);
    }

    fn event(
        &mut self,
        (state, content): &mut Self::State,
        cx: &mut EventCx,
        data: &mut T,
        event: &Event,
    ) -> bool {
        if cx.focused_changed() {
            cx.draw();
        }

        let handled = self.content.event(content, cx, data, event);

        if cx.hovered_changed() || cx.active_changed() {
            cx.animate();
        }

        if let Event::Animate(dt) = event {
            let hover = (state.style.transition).step(&mut state.hovered, cx.is_hovered(), *dt);
            let active = (state.style.transition).step(&mut state.active, cx.is_active(), *dt);

            if hover || active {
                cx.animate();
                cx.draw();
            }
        }

        handled
    }

    fn layout(
        &mut self,
        (state, content): &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
        let content_space = space.shrink(state.style.padding.size());
        let content_size = self.content.layout(content, cx, data, content_space);

        content.translate(state.style.padding.offset());

        space.fit(content_size + state.style.padding.size())
    }

    fn draw(&mut self, (state, content): &mut Self::State, cx: &mut DrawCx, data: &mut T) {
        cx.hoverable(|cx| {
            let dark = state.style.color.darken(0.05);
            let dim = state.style.color.darken(0.025);
            let bright = state.style.color.lighten(0.05);

            let hovered = state.style.transition.get(state.hovered);
            let active = state.style.transition.get(state.active);

            let face = state.style.color.mix(bright, hovered).mix(dim, active);

            if cx.is_focused() {
                let info = cx.theme().info;

                cx.quad(
                    cx.rect().expand(2.0),
                    Color::TRANSPARENT,
                    state.style.border_radius.expand(2.0),
                    BorderWidth::all(2.0),
                    info,
                );
            }

            if state.style.fancy == 0.0 {
                cx.quad(
                    cx.rect(),
                    face,
                    state.style.border_radius,
                    state.style.border_width,
                    state.style.border_color,
                );

                self.content.draw(content, cx, data);
                return;
            }

            let base = dim.mix(dark, 1.0 - active);

            cx.quad(
                cx.rect(),
                base,
                state.style.border_radius,
                BorderWidth::ZERO,
                Color::TRANSPARENT,
            );

            let float = Vector::NEG_Y * (1.0 - active) * state.style.fancy;

            cx.translated(float, |cx| {
                cx.quad(
                    cx.rect(),
                    face,
                    state.style.border_radius,
                    state.style.border_width,
                    state.style.border_color,
                );

                self.content.draw(content, cx, data);
            });
        });
    }
}
