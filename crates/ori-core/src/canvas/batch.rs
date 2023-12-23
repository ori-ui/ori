use crate::{
    image::Texture,
    layout::{Affine, Rect},
};

use super::{Mesh, Quad};

/// A batched quad.
#[derive(Clone, Debug, Default)]
pub struct BatchedQuad {
    /// The batched quad.
    pub quad: Quad,
    /// The affine transform.
    pub transform: Affine,
}

/// A batched mesh.
#[derive(Clone, Debug, Default)]
pub struct BatchedMesh {
    /// The batched mesh.
    pub mesh: Mesh,
    /// The affine transform.
    pub transform: Affine,
}

/// A batch of quads to draw.
#[derive(Clone, Debug, Default)]
pub struct QuadBatch {
    /// The index of the batch.
    pub index: usize,
    /// The items to draw.
    pub quads: Vec<BatchedQuad>,
    /// The texture of the batch.
    pub texture: Option<Texture>,
    /// The clipping rectangle of the batch.
    pub clip: Rect,
}

/// A batch of meshes to draw.
#[derive(Clone, Debug, Default)]
pub struct MeshBatch {
    /// The index of the batch.
    pub index: usize,
    /// The items to draw.
    pub meshes: Vec<BatchedMesh>,
    /// The texture of the batch.
    pub texture: Option<Texture>,
    /// The clipping rectangle of the batch.
    pub clip: Rect,
}

/// A batch of primitives to draw.
#[derive(Clone, Debug)]
pub enum Batch {
    /// A quad batch.
    Quad(QuadBatch),
    /// A mesh batch.
    Mesh(MeshBatch),
}
