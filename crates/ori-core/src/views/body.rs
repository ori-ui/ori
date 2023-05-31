use glam::Vec2;
use ori_macro::Build;
use ori_reactive::Event;

use crate::{
    AvailableSpace, Children, DrawContext, EventContext, FlexLayout, LayoutContext, Style, View,
};

#[derive(Default, Build)]
pub struct Body {
    #[children]
    pub children: Children,
}

impl Body {
    pub fn new() -> Self {
        Self::default()
    }
}

impl View for Body {
    type State = ();

    fn build(&self) -> Self::State {}

    fn style(&self) -> Style {
        Style::new("body")
    }

    fn event(&self, _: &mut Self::State, cx: &mut EventContext, event: &Event) {
        self.children.event(cx, event);
    }

    fn layout(&self, _: &mut Self::State, cx: &mut LayoutContext, space: AvailableSpace) -> Vec2 {
        let layout = FlexLayout::from_style(cx);
        self.children.flex_layout(cx, space, layout)
    }

    fn draw(&self, _: &mut Self::State, cx: &mut DrawContext) {
        cx.draw_quad();

        cx.draw_layer(|cx| {
            self.children.draw(cx);
        });
    }
}
