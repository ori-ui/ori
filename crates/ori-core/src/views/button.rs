use ori_macro::{example, Build};

use crate::{
    canvas::{BorderRadius, BorderWidth, Color},
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    layout::{Padding, Size, Space, Vector},
    rebuild::Rebuild,
    style::{Stylable, Styled, Theme},
    transition::Transition,
    view::{Pod, PodState, View},
};

/// Create a new [`Button`].
pub fn button<V>(view: V) -> Button<V> {
    Button::new(view)
}

/// A button.
///
/// Can be styled using the [`ButtonStyle`].
#[example(name = "button", width = 400, height = 300)]
#[derive(Stylable, Build, Rebuild)]
pub struct Button<V> {
    /// The content.
    #[build(ignore)]
    pub content: Pod<V>,

    /// The padding.
    #[rebuild(layout)]
    #[style(default = Padding::all(8.0))]
    pub padding: Styled<Padding>,

    /// The distance of the fancy effect.
    #[rebuild(draw)]
    #[style(default = 0.0)]
    pub fancy: Styled<f32>,

    /// The transition of the button.
    #[rebuild(draw)]
    #[style(default = Transition::ease(0.1))]
    pub transition: Styled<Transition>,

    /// The color of the button.
    #[rebuild(draw)]
    #[style(default -> Theme::SURFACE_HIGHER or Color::WHITE)]
    pub color: Styled<Color>,

    /// The border radius.
    #[rebuild(draw)]
    #[style(default = BorderRadius::all(4.0))]
    pub border_radius: Styled<BorderRadius>,

    /// The border width.
    #[rebuild(draw)]
    #[style(default)]
    pub border_width: Styled<BorderWidth>,

    /// The border color.
    #[rebuild(draw)]
    #[style(default -> Theme::OUTLINE or Color::BLACK)]
    pub border_color: Styled<Color>,
}

impl<V> Button<V> {
    /// Create a new [`Button`].
    pub fn new(content: V) -> Self {
        Self {
            content: Pod::new(content),
            padding: Styled::style("button.padding"),
            fancy: Styled::style("button.fancy"),
            transition: Styled::style("button.transition"),
            color: Styled::style("button.color"),
            border_radius: Styled::style("button.border-radius"),
            border_width: Styled::style("button.border-width"),
            border_color: Styled::style("button.border-color"),
        }
    }
}

#[doc(hidden)]
pub struct ButtonState<V> {
    pub hovered: f32,
    pub active: f32,
    pub style: ButtonStyle<V>,
}

impl<T, V: View<T>> View<T> for Button<V> {
    type State = (ButtonState<V>, PodState<T, V>);

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        cx.set_class("button");
        cx.set_focusable(true);

        let state = ButtonState {
            hovered: 0.0,
            active: 0.0,
            style: self.style(cx.styles()),
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
        Rebuild::rebuild(self, cx, old);
        state.style.rebuild(self, cx);

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
                cx.quad(
                    cx.rect().expand(2.0),
                    Color::TRANSPARENT,
                    state.style.border_radius.expand(2.0),
                    BorderWidth::all(2.0),
                    cx.styles().get_or(Color::BLUE, &Theme::INFO),
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
