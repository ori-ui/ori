use crate::{Style, View};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Comment {
    comment: &'static str,
}

impl Comment {
    pub fn new(comment: &'static str) -> Self {
        Self { comment }
    }
}

impl View for Comment {
    type State = ();

    fn build(&self) -> Self::State {}

    fn style(&self) -> Style {
        Style::new("comment")
    }
}
