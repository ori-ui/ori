use glam::Vec2;

use crate::{Affine, Size, Update};

#[derive(Clone, Debug)]
pub struct ViewState {
    pub(crate) hot: bool,
    pub(crate) active: bool,
    pub(crate) has_active: bool,
    pub(crate) update: Update,
    /* layout */
    pub(crate) flex: f32,
    pub(crate) size: Size,
    pub(crate) transform: Affine,
}

impl Default for ViewState {
    fn default() -> Self {
        Self {
            hot: false,
            active: false,
            has_active: false,
            update: Update::LAYOUT | Update::DRAW,
            flex: 0.0,
            size: Size::ZERO,
            transform: Affine::IDENTITY,
        }
    }
}

impl ViewState {
    pub(crate) fn propagate(&mut self, child: &mut Self) {
        self.has_active |= child.has_active;
        self.update |= child.update;
    }
}

impl ViewState {
    pub fn is_hot(&self) -> bool {
        self.hot
    }

    pub fn set_hot(&mut self, hot: bool) {
        self.hot = hot;
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn set_active(&mut self, active: bool) {
        self.active = active;
        self.has_active = active;
    }

    pub fn flex(&self) -> f32 {
        self.flex
    }

    pub fn set_flex(&mut self, flex: f32) {
        self.flex = flex;
    }

    pub fn size(&self) -> Size {
        self.size
    }

    pub fn transform(&self) -> Affine {
        self.transform
    }

    pub fn set_transform(&mut self, transform: Affine) {
        self.transform = transform;
    }

    pub fn translate(&mut self, translation: Vec2) {
        self.transform = Affine::translate(translation);
    }

    pub fn request_rebuild(&mut self) {
        self.update |= Update::TREE;
    }

    pub fn request_layout(&mut self) {
        self.update |= Update::LAYOUT;
    }

    pub fn request_draw(&mut self) {
        self.update |= Update::DRAW;
    }

    pub fn update(&self) -> Update {
        self.update
    }
}
