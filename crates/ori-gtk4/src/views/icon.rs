use crate::Context;

pub fn icon(name: impl Into<String>) -> Icon {
    Icon::new(name)
}

pub struct Icon {
    name: String,
}

impl Icon {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

impl ori::ViewMarker for Icon {}
impl<T> ori::View<Context, T> for Icon {
    type Element = gtk4::Image;
    type State = ();

    fn build(&mut self, _cx: &mut Context, _data: &mut T) -> (Self::Element, Self::State) {
        let element = gtk4::Image::from_icon_name(&self.name);

        (element, ())
    }

    fn rebuild(
        &mut self,
        element: &mut Self::Element,
        _state: &mut Self::State,
        _cx: &mut Context,
        _data: &mut T,
        old: &mut Self,
    ) {
        if self.name != old.name {
            element.set_icon_name(Some(&self.name));
        }
    }

    fn teardown(
        &mut self,
        _element: Self::Element,
        _state: Self::State,
        _cx: &mut Context,
        _data: &mut T,
    ) {
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
