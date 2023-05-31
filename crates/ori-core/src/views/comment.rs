use ori_style::Style;

use crate::View;

/// A comment view.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Comment {
    /// The content of the comment.
    pub comment: &'static str,
}

impl Comment {
    /// Create a new comment.
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
