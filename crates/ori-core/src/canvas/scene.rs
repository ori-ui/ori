use crate::{Affine, Primitive, Rect};

#[derive(Clone, Debug)]
pub struct Scene {
    fragments: Vec<Fragment>,
}

impl Scene {
    pub const fn new() -> Self {
        Self {
            fragments: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.fragments.clear();
    }

    pub fn push(&mut self, fragment: Fragment) {
        self.fragments.push(fragment);
    }

    pub fn fragments(&self) -> &[Fragment] {
        &self.fragments
    }

    pub fn fragments_mut(&mut self) -> &mut [Fragment] {
        &mut self.fragments
    }
}

#[derive(Clone, Debug)]
pub struct Fragment {
    pub primitive: Primitive,
    pub transform: Affine,
    pub depth: f32,
    pub clip: Rect,
}
