use std::f32::consts::PI;

use ori::prelude::*;

#[derive(Default, Build)]
struct CustomView {}

impl View for CustomView {
    type State = ();

    fn build(&self) -> Self::State {}

    // set the style of the view, with a custom element name "custom-view"
    fn style(&self) -> Style {
        Style::new("custom-view")
    }

    // layout the view
    fn layout(
        &self,
        _state: &mut Self::State,
        _cx: &mut LayoutContext,
        space: AvailableSpace,
    ) -> Vec2 {
        // this view will take all the available space
        space.max
    }

    fn draw(&self, _state: &mut Self::State, cx: &mut DrawContext) {
        let curve = Curve::parametric(
            |t| {
                let t = t * 2.0;
                let x = t.sin();
                let y = t.cos();
                cx.rect().center() + Vec2::new(x, y) * 100.0
            },
            0.0,
            PI,
        );

        cx.draw(curve.fill(Color::BLUE));
    }
}

fn ui(_cx: Scope) -> impl View {
    view! {
        <CustomView />
    }
}

fn main() {
    App::new(ui).run();
}
