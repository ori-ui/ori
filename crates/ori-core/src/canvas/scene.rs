use crate::{Affine, Primitive, Rect};

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
