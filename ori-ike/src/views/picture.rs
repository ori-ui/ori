use ike::{
    BuildCx, Color,
    widgets::{Fit, Picturable},
};

use crate::Context;

pub fn picture(fit: Fit, content: impl Into<Picturable>) -> Picture {
    Picture::new(fit, content)
}

pub struct Picture {
    contents: Picturable,
    fit:      Fit,
    color:    Option<Color>,
}

impl Picture {
    pub fn new(fit: Fit, content: impl Into<Picturable>) -> Self {
        Self {
            contents: content.into(),
            fit,
            color: None,
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
}

impl ori::ViewMarker for Picture {}
impl<T> ori::View<Context, T> for Picture {
    type Element = ike::WidgetId<ike::widgets::Picture>;
    type State = ();

    fn build(&mut self, cx: &mut Context, _data: &mut T) -> (Self::Element, Self::State) {
        let mut widget = ike::widgets::Picture::new(cx, self.contents.clone());
        ike::widgets::Picture::set_fit(&mut widget, self.fit);
        ike::widgets::Picture::set_color(&mut widget, self.color);

        (widget.id(), ())
    }

    fn rebuild(
        &mut self,
        element: &mut Self::Element,
        _state: &mut Self::State,
        cx: &mut Context,
        _data: &mut T,
        old: &mut Self,
    ) {
        let mut widget = cx.get_mut(*element);

        if self.contents != old.contents {
            ike::widgets::Picture::set_contents(&mut widget, self.contents.clone());
        }

        if self.fit != old.fit {
            ike::widgets::Picture::set_fit(&mut widget, self.fit);
        }

        if self.color != old.color {
            ike::widgets::Picture::set_color(&mut widget, self.color);
        }
    }

    fn teardown(
        &mut self,
        element: Self::Element,
        _state: Self::State,
        cx: &mut Context,
        _data: &mut T,
    ) {
        cx.remove(element);
    }

    fn event(
        &mut self,
        _element: &mut Self::Element,
        _state: &mut Self::State,
        _cx: &mut Context,
        _data: &mut T,
        _event: &mut ori::Event,
    ) -> ori::Action {
        ori::Action::new()
    }
}
