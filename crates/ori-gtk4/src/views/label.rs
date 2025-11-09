use crate::Context;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Wrap {
    Word,
    Char,
    WordChar,
}

impl From<Wrap> for gtk4::pango::WrapMode {
    fn from(wrap: Wrap) -> Self {
        match wrap {
            Wrap::Word => gtk4::pango::WrapMode::Word,
            Wrap::Char => gtk4::pango::WrapMode::Char,
            Wrap::WordChar => gtk4::pango::WrapMode::WordChar,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Ellipsize {
    Start,
    Middle,
    End,
}

impl From<Ellipsize> for gtk4::pango::EllipsizeMode {
    fn from(ellipsize: Ellipsize) -> Self {
        match ellipsize {
            Ellipsize::Start => gtk4::pango::EllipsizeMode::Start,
            Ellipsize::Middle => gtk4::pango::EllipsizeMode::Middle,
            Ellipsize::End => gtk4::pango::EllipsizeMode::End,
        }
    }
}

pub fn label(text: impl ToString) -> Label {
    Label::new(text)
}

#[must_use]
pub struct Label {
    pub text: String,
    pub wrap: Option<Wrap>,
    pub ellipsize: Option<Ellipsize>,
}

impl Label {
    pub fn new(text: impl ToString) -> Self {
        Self {
            text: text.to_string(),
            wrap: None,
            ellipsize: None,
        }
    }

    pub fn wrap(mut self, wrap: impl Into<Option<Wrap>>) -> Self {
        self.wrap = wrap.into();
        self
    }

    pub fn ellipsize(
        mut self,
        ellipsize: impl Into<Option<Ellipsize>>,
    ) -> Self {
        self.ellipsize = ellipsize.into();
        self
    }
}

impl<T> ori::View<Context, T> for Label {
    type Element = gtk4::Label;
    type State = ();

    fn build(
        &mut self,
        _cx: &mut Context,
        _data: &mut T,
    ) -> (Self::Element, Self::State) {
        let element = gtk4::Label::new(Some(&self.text));

        match self.wrap {
            Some(wrap) => {
                element.set_wrap(true);
                element.set_wrap_mode(wrap.into());
            }

            None => element.set_wrap(false),
        }

        element.set_ellipsize(self.ellipsize.map_or(
            gtk4::pango::EllipsizeMode::None,
            |ellipsize| ellipsize.into(),
        ));

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
        if self.text != old.text {
            element.set_text(&self.text);
        }

        if self.wrap != old.wrap {
            match self.wrap {
                Some(wrap) => {
                    element.set_wrap(true);
                    element.set_wrap_mode(wrap.into());
                }

                None => element.set_wrap(false),
            }
        }

        if self.ellipsize != old.ellipsize {
            element.set_ellipsize(self.ellipsize.map_or(
                gtk4::pango::EllipsizeMode::None,
                |ellipsize| ellipsize.into(),
            ));
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
