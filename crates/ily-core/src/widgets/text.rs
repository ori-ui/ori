use glam::Vec2;

use crate::{BoxConstraints, View};

#[allow(dead_code)]
pub struct Text {
    text: String,
}

impl Text {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

impl View for Text {
    fn layout(&self, _bc: BoxConstraints) -> Vec2 {
        Vec2::new(200.0, self.text.len() as f32 * 20.0)
    }
}
