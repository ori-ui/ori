use crate::View;

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
    fn element(&self) -> Option<&'static str> {
        Some("comment")
    }
}
