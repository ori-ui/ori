use crate::Context;

pub fn progress(fraction: f32) -> Progress {
    Progress::new(fraction)
}

pub struct Progress {
    fraction: f32,
    inverted: bool,
}

impl Progress {
    pub fn new(fraction: f32) -> Self {
        Self {
            fraction,
            inverted: false,
        }
    }

    pub fn inverted(mut self, inverted: bool) -> Self {
        self.inverted = inverted;
        self
    }
}

impl ori::ViewMarker for Progress {}
impl<T> ori::View<Context, T> for Progress {
    type Element = gtk4::ProgressBar;
    type State = ();

    fn build(&mut self, _cx: &mut Context, _data: &mut T) -> (Self::Element, Self::State) {
        let element = gtk4::ProgressBar::new();

        element.set_fraction(self.fraction as f64);
        element.set_inverted(self.inverted);

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
        if self.fraction != old.fraction {
            element.set_fraction(self.fraction as f64);
        }

        if self.inverted != old.inverted {
            element.set_inverted(self.inverted);
        }
    }

    fn teardown(&mut self, _element: Self::Element, _state: Self::State, _cx: &mut Context) {}

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
