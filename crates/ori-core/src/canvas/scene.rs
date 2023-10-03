use std::cmp::Ordering;

use crate::layout::{Affine, Rect};

use super::Primitive;

/// A collection of fragments to be rendered.
#[derive(Clone, Debug)]
pub struct Scene {
    fragments: Vec<Fragment>,
}

impl Scene {
    /// Create a new empty scene.
    pub const fn new() -> Self {
        Self {
            fragments: Vec::new(),
        }
    }

    /// Clear the scene.
    pub fn clear(&mut self) {
        self.fragments.clear();
    }

    /// Push a fragment to the scene.
    pub fn push(&mut self, fragment: Fragment) {
        self.fragments.push(fragment);
    }

    /// Get the fragments in the scene.
    pub fn fragments(&self) -> &[Fragment] {
        &self.fragments
    }

    /// Get a mutable reference to the fragments in the scene.
    pub fn fragments_mut(&mut self) -> &mut [Fragment] {
        &mut self.fragments
    }

    /// Sort the fragments in the scene by depth.
    pub fn sort(&mut self) {
        fn cmp(a: &Fragment, b: &Fragment) -> Ordering {
            a.depth.partial_cmp(&b.depth).unwrap_or(Ordering::Equal)
        }

        self.fragments.sort_by(cmp)
    }
}

/// A item to be rendered.
#[derive(Clone, Debug)]
pub struct Fragment {
    /// The primitive to be rendered.
    pub primitive: Primitive,
    /// The transform to apply to the primitive.
    pub transform: Affine,
    /// The depth of the primitive.
    pub depth: f32,
    /// The clip rectangle of the primitive.
    pub clip: Rect,
}
