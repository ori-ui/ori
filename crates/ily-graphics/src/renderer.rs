use std::any::{Any, TypeId};

use glam::Vec2;

use crate::{ImageData, ImageHandle, Rect, TextHit, TextSection};

pub trait Renderer: Any {
    fn window_size(&self) -> Vec2;
    fn create_image(&self, data: &ImageData) -> ImageHandle;
    fn messure_text(&self, section: &TextSection) -> Option<Rect>;
    fn hit_text(&self, section: &TextSection, position: Vec2) -> Option<TextHit>;

    fn scale(&self) -> f32 {
        1.0
    }
}

impl dyn Renderer {
    pub fn downcast_ref<T: Renderer>(&self) -> Option<&T> {
        // SAFETY: This obeys the safety rules of `Any::downcast_ref`.
        if TypeId::of::<T>() == Any::type_id(&*self) {
            unsafe { Some(&*(self as *const dyn Renderer as *const T)) }
        } else {
            None
        }
    }

    pub fn downcast_mut<T: Renderer>(&mut self) -> Option<&mut T> {
        // SAFETY: This obeys the safety rules of `Any::downcast_mut`.
        if TypeId::of::<T>() == Any::type_id(&*self) {
            unsafe { Some(&mut *(self as *mut dyn Renderer as *mut T)) }
        } else {
            None
        }
    }
}
