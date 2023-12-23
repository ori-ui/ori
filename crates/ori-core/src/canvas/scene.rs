use std::{cmp::Ordering, mem};

use crate::{
    layout::{Affine, Point, Rect},
    view::ViewId,
};

use super::{Batch, BatchedMesh, BatchedQuad, MeshBatch, Primitive, QuadBatch};

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
    pub fn batches(&self) -> Vec<Batch> {
        let mut batches = Vec::new();

        let mut quad_image = None;
        let mut quad_clip = None;
        let mut batched_quads = Vec::new();
        let mut quad_count = 0;

        let mut mesh_image = None;
        let mut mesh_clip = None;
        let mut batched_meshes = Vec::new();
        let mut mesh_count = 0;

        for fragment in &self.fragments {
            match fragment.primitive {
                Primitive::Quad(ref quad) => {
                    if !batched_meshes.is_empty() {
                        batches.push(Batch::Mesh(MeshBatch {
                            index: mesh_count,
                            meshes: mem::take(&mut batched_meshes),
                            texture: mem::take(&mut mesh_image),
                            clip: mesh_clip.unwrap(),
                        }));

                        mesh_count += 1;
                    }

                    let image = quad.background.texture.clone();
                    let compatible = quad_clip == Some(fragment.clip) && quad_image == image;
                    if !compatible && !batched_quads.is_empty() {
                        batches.push(Batch::Quad(QuadBatch {
                            index: quad_count,
                            quads: mem::take(&mut batched_quads),
                            texture: mem::take(&mut quad_image),
                            clip: quad_clip.unwrap(),
                        }));

                        quad_count += 1;
                    }

                    quad_image = image;
                    quad_clip = Some(fragment.clip);
                    batched_quads.push(BatchedQuad {
                        quad: quad.clone(),
                        transform: fragment.transform,
                    });
                }
                Primitive::Mesh(ref mesh) => {
                    if !batched_quads.is_empty() {
                        batches.push(Batch::Quad(QuadBatch {
                            index: quad_count,
                            quads: mem::take(&mut batched_quads),
                            texture: mem::take(&mut quad_image),
                            clip: quad_clip.unwrap(),
                        }));

                        quad_count += 1;
                    }

                    let image = mesh.texture.clone();
                    let compatible = mesh_clip == Some(fragment.clip) && mesh_image == image;
                    if !compatible && !batched_meshes.is_empty() {
                        batches.push(Batch::Mesh(MeshBatch {
                            index: mesh_count,
                            meshes: mem::take(&mut batched_meshes),
                            texture: mem::take(&mut mesh_image),
                            clip: mesh_clip.unwrap(),
                        }));

                        mesh_count += 1;
                    }

                    mesh_image = image;
                    mesh_clip = Some(fragment.clip);
                    batched_meshes.push(BatchedMesh {
                        mesh: mesh.clone(),
                        transform: fragment.transform,
                    });
                }
                Primitive::Trigger(_) => {}
            }
        }

        if !batched_quads.is_empty() {
            batches.push(Batch::Quad(QuadBatch {
                index: quad_count,
                quads: mem::take(&mut batched_quads),
                texture: mem::take(&mut quad_image),
                clip: quad_clip.unwrap(),
            }));
        }

        if !batched_meshes.is_empty() {
            batches.push(Batch::Mesh(MeshBatch {
                index: mesh_count,
                meshes: mem::take(&mut batched_meshes),
                texture: mem::take(&mut mesh_image),
                clip: mesh_clip.unwrap(),
            }));
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
}
