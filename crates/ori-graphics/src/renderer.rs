use std::any::{Any, TypeId};

use glam::Vec2;

use crate::{Glyph, ImageData, ImageHandle, Line, Rect, TextHit, TextSection};

pub trait Renderer: Any {
    fn window_size(&self) -> Vec2;
    fn create_image(&self, data: &ImageData) -> ImageHandle;
    fn text_glyphs(&self, section: &TextSection) -> Vec<Glyph>;

    /// Calculates the bounding [`Rect`] of a [`TextSection`].
    fn messure_text(&self, section: &TextSection) -> Option<Rect> {
        let mut rect = None::<Rect>;

        for glyph in self.text_glyphs(section) {
            if let Some(ref mut rect) = rect {
                *rect = rect.union(glyph.rect);
            } else {
                rect = Some(glyph.rect);
            }
        }

        Some(Rect {
            min: rect?.min,
            max: rect?.max + 1.0,
        })
    }

    fn text_lines(&self, section: &TextSection) -> Vec<Line>;

    /// Calculates the [`TextHit`] of a [`TextSection`] at a given point.
    fn hit_text(&self, section: &TextSection, point: Vec2) -> Option<TextHit> {
        for line in self.text_lines(section) {
            if !(point.y > line.rect.min.y && point.y < line.rect.max.y) {
                continue;
            }

            for glyph in line.glyphs.iter() {
                if glyph.rect.contains(point) {
                    let delta = point - glyph.rect.center();

                    return Some(TextHit {
                        index: glyph.index,
                        inside: true,
                        delta,
                    });
                }
            }

            let (index, delta) = if let Some(glyph) = line.glyphs.last() {
                (glyph.index, point - glyph.rect.center())
            } else {
                (line.index, point - line.rect.center())
            };

            return Some(TextHit {
                index,
                inside: false,
                delta,
            });
        }

        None
    }

    /// Returns the scale of the [`Renderer`].
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
