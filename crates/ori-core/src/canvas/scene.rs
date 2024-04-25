use std::{cmp::Ordering, mem};

use crate::{
    layout::{Affine, Point, Rect},
    view::ViewId,
};

use super::{Mesh, Primitive};

/// A batch of meshes to draw.
#[derive(Clone, Debug, Default)]
pub struct Batch {
    /// The index of the batch.
    pub index: usize,
    /// The items to draw.
    pub mesh: Mesh,
    /// The clipping rectangle of the batch.
    ///
    /// Note that the clipping rectangle is in logical coordinates, and must be transformed to
    /// physical coordinates before being used by the rendering backend.
    pub clip: Rect,
}

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

        self.fragments.sort_by(cmp);
    }

    /// Get the fragment that intersect with the given point.
    pub fn view_at(&self, point: Point) -> Option<ViewId> {
        for fragment in self.fragments.iter().rev() {
            // discard fragments without a view
            let view = match fragment.view {
                Some(view) => view,
                None => continue,
            };

            // discard fragments that are clipped
            if !fragment.clip.contains(point) {
                continue;
            }

            let local = fragment.transform.inverse() * point;

            // if the primitive intersects with the point, return the view
            if fragment.primitive.intersects_point(local) {
                return Some(view);
            }
        }

        None
    }

    /// Get the batches in the scene.
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(self)))]
    pub fn batches(&self) -> Vec<Batch> {
        let mut batches = Vec::new();

        let mut mesh = Mesh::new();
        let mut clip = None;

        for fragment in self.fragments.iter() {
            let fragment_mesh = match fragment.primitive {
                Primitive::Trigger(_) => continue,
                Primitive::Quad(ref quad) => quad.compute_mesh(),
                Primitive::Mesh(ref mesh) => mesh.clone(),
            };

            let texture_compatible = fragment_mesh.texture == mesh.texture;
            let compatible = texture_compatible && Some(fragment.clip) == clip;

            if !compatible && !mesh.is_empty() {
                batches.push(Batch {
                    index: batches.len(),
                    mesh: mem::take(&mut mesh),
                    clip: clip.unwrap(),
                });
            }

            mesh.extend_transformed(&fragment_mesh, fragment.transform);

            mesh.texture = fragment_mesh.texture;

            clip = Some(fragment.clip);
        }

        if !mesh.is_empty() {
            batches.push(Batch {
                index: batches.len(),
                mesh,
                clip: clip.unwrap(),
            });
        }

        batches
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
    /// The view that the primitive is being drawn for.
    pub view: Option<ViewId>,
    /// Whether the fragment should be drawn pixel perfect.
    pub pixel_perfect: bool,
}
