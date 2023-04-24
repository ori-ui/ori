use wgpu::{BindGroup, Sampler, Texture, TextureView};

#[derive(Debug)]
pub struct WgpuImage {
    pub texture: Texture,
    pub view: TextureView,
    pub sampler: Sampler,
    pub bind_group: BindGroup,
}
