use std::ops::{Deref, DerefMut};

use crate::{
    layout::Vector,
    text::{Fonts, TextBuffer},
    view::ViewState,
};

use super::BaseCx;

/// A context for laying out the view tree.
pub struct LayoutCx<'a, 'b> {
    pub(crate) base: &'a mut BaseCx<'b>,
    pub(crate) view_state: &'a mut ViewState,
}

impl<'a, 'b> Deref for LayoutCx<'a, 'b> {
    type Target = BaseCx<'b>;

    fn deref(&self) -> &Self::Target {
        self.base
    }
}

impl<'a, 'b> DerefMut for LayoutCx<'a, 'b> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.base
    }
}

impl<'a, 'b> LayoutCx<'a, 'b> {
    /// Create a new layout context.
    pub fn new(base: &'a mut BaseCx<'b>, view_state: &'a mut ViewState) -> Self {
        Self { base, view_state }
    }

    /// Create a child context.
    pub fn child(&mut self) -> LayoutCx<'_, 'b> {
        LayoutCx {
            base: self.base,
            view_state: self.view_state,
        }
    }

    /// Prepare text for drawing.
    pub fn prepare_text(&mut self, buffer: &TextBuffer, offset: Vector) {
        self.prepare_text_raw(buffer.raw(), offset);
    }

    /// Prepare text for drawing.
    pub fn prepare_text_raw(&mut self, buffer: &cosmic_text::Buffer, offset: Vector) {
        let scale = self.window().scale;
        let contexts = &mut *self.base.contexts;

        let fonts = contexts.get_or_default::<Fonts>();
        fonts.prepare_buffer(buffer, offset, scale);
    }
}
