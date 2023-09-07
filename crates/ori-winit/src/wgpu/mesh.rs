use std::{mem, sync::Arc};

use ori_core::{
    canvas::{Mesh, Vertex},
    layout::{Affine, Point, Rect, Size},
};
use wgpu::{
    include_wgsl, vertex_attr_array, BindGroup, BindGroupDescriptor, BindGroupEntry,
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BlendState,
    Buffer, BufferBindingType, BufferDescriptor, BufferUsages, ColorTargetState, ColorWrites,
    Device, FragmentState, IndexFormat, MultisampleState, PipelineLayoutDescriptor, Queue,
    RenderPass, RenderPipeline, RenderPipelineDescriptor, ShaderStages, TextureFormat,
    VertexBufferLayout, VertexState, VertexStepMode,
};

use super::{bytes_of, bytes_of_slice, CachedImage, ImageCache};

#[repr(C)]
#[derive(Clone, Copy)]
struct Uniforms {
    resolution: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct MeshVertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
    color: [f32; 4],
}

#[derive(Debug)]
struct Batch {
    index_count: usize,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    image: Option<Arc<CachedImage>>,
    clip: Rect,
}

impl Batch {
    fn new(device: &Device) -> Self {
        let vertex_buffer = Self::create_vertex_buffer(device, 512);
        let index_buffer = Self::create_index_buffer(device, 512);

        Self {
            index_count: 0,
            vertex_buffer,
            index_buffer,
            image: None,
            clip: Rect::ZERO,
        }
    }

    fn create_vertex_buffer(device: &Device, len: usize) -> Buffer {
        device.create_buffer(&BufferDescriptor {
            label: Some("ori_mesh_vertex_buffer"),
            size: mem::size_of::<Vertex>() as u64 * len as u64,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    fn create_index_buffer(device: &Device, len: usize) -> Buffer {
        device.create_buffer(&BufferDescriptor {
            label: Some("ori_mesh_index_buffer"),
            size: mem::size_of::<u32>() as u64 * len as u64,
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    fn resize_vertex_buffer(&mut self, device: &Device, len: usize) {
        let size = mem::size_of::<Vertex>() as u64 * len as u64;

        if self.vertex_buffer.size() < size {
            self.vertex_buffer = Self::create_vertex_buffer(device, len);
        }
    }

    fn resize_index_buffer(&mut self, device: &Device, len: usize) {
        let size = mem::size_of::<u32>() as u64 * len as u64;

        if self.index_buffer.size() < size {
            self.index_buffer = Self::create_index_buffer(device, len);
        }
    }
}

#[derive(Debug)]
pub struct MeshRender {
    batches: Vec<Batch>,
    pipeline: RenderPipeline,
    uniform_buffer: Buffer,
    uniform_bind_group: BindGroup,
}

impl MeshRender {
    pub fn new(
        device: &Device,
        format: TextureFormat,
        sample_count: u32,
        image_layout: &BindGroupLayout,
    ) -> Self {
        let shader = device.create_shader_module(include_wgsl!("shader/mesh.wgsl"));

        let uniform_buffer = Self::create_uniform_buffer(device);

        let uniform_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("ori_mesh_uniform_layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let uniform_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("ori_mesh_uniform_bind_group"),
            layout: &uniform_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("ori_quad_pipeline_layout"),
            bind_group_layouts: &[&uniform_layout, image_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("ori_quad_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vertex",
                buffers: &[
                    VertexBufferLayout {
                        array_stride: mem::size_of::<Vertex>() as u64,
                        step_mode: VertexStepMode::Vertex,
                        attributes: &vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Float32x4, 3 => Uint32],
                    },
                ],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fragment",
                targets: &[Some(ColorTargetState {
                    format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: MultisampleState {
                count: sample_count,
                ..Default::default()
            },
            multiview: None,
        });

        Self {
            batches: Vec::new(),
            pipeline,
            uniform_buffer,
            uniform_bind_group,
        }
    }

    fn create_uniform_buffer(device: &Device) -> Buffer {
        device.create_buffer(&BufferDescriptor {
            label: Some("ori_mesh_uniform_buffer"),
            size: mem::size_of::<Uniforms>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    fn write_uniforms(&self, queue: &Queue, resolution: Size) {
        let uniforms = Uniforms {
            resolution: resolution.into(),
        };

        unsafe {
            queue.write_buffer(&self.uniform_buffer, 0, bytes_of(&uniforms));
        }
    }

    fn resize_batches(&mut self, device: &Device, len: usize) {
        if self.batches.len() < len {
            self.batches.resize_with(len, || Batch::new(device));
        }
    }

    fn batch_index_count(batch: &[(&Mesh, Affine)]) -> usize {
        batch.iter().map(|(mesh, _)| mesh.indices.len()).sum()
    }

    fn batch_vertex_count(batch: &[(&Mesh, Affine)]) -> usize {
        batch.iter().map(|(mesh, _)| mesh.vertices.len()).sum()
    }

    fn batch_image(
        device: &Device,
        queue: &Queue,
        cache: &mut ImageCache,
        batch: &[(&Mesh, Affine)],
    ) -> Arc<CachedImage> {
        match batch[0].0.image {
            Some(ref image) => cache.get(device, queue, image),
            None => cache.fallback(device, queue),
        }
    }

    fn append_mesh_to_batch(
        vertices: &mut Vec<MeshVertex>,
        indices: &mut Vec<u32>,
        mesh: &Mesh,
        transform: Affine,
    ) {
        let vertex_offset = vertices.len() as u32;

        for vertex in &mesh.vertices {
            let position = transform * vertex.position;

            vertices.push(MeshVertex {
                position: position.into(),
                tex_coords: vertex.tex_coords.into(),
                color: vertex.color.into(),
            });
        }

        for index in &mesh.indices {
            indices.push(index + vertex_offset);
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn prepare_batch(
        &mut self,
        device: &Device,
        queue: &Queue,
        cache: &mut ImageCache,
        index: usize,
        meshes: &[(&Mesh, Affine)],
        clip: Rect,
        resolution: Size,
    ) {
        assert!(!meshes.is_empty());

        self.write_uniforms(queue, resolution);
        self.resize_batches(device, index + 1);

        let index_count = Self::batch_index_count(meshes);
        let vertex_count = Self::batch_vertex_count(meshes);

        let batch = &mut self.batches[index];
        batch.resize_index_buffer(device, index_count);
        batch.resize_vertex_buffer(device, vertex_count);
        batch.index_count = index_count;
        batch.image = Some(Self::batch_image(device, queue, cache, meshes));
        batch.clip = clip.clamp(Rect::min_size(Point::ZERO, resolution)).round();

        let mut vertices = Vec::with_capacity(vertex_count);
        let mut indices = Vec::with_capacity(index_count);

        for (mesh, transform) in meshes {
            Self::append_mesh_to_batch(&mut vertices, &mut indices, mesh, *transform);
        }

        unsafe {
            queue.write_buffer(&batch.vertex_buffer, 0, bytes_of_slice(&vertices));
            queue.write_buffer(&batch.index_buffer, 0, bytes_of_slice(&indices));
        }
    }

    pub fn render<'a>(&'a self, pass: &mut RenderPass<'a>, index: usize) {
        let batch = &self.batches[index];

        let image_bind_group = &batch.image.as_ref().unwrap().bind_group;

        pass.set_scissor_rect(
            batch.clip.min.x as u32,
            batch.clip.min.y as u32,
            batch.clip.width() as u32,
            batch.clip.height() as u32,
        );

        pass.set_pipeline(&self.pipeline);

        pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        pass.set_bind_group(1, image_bind_group, &[]);

        pass.set_vertex_buffer(0, batch.vertex_buffer.slice(..));
        pass.set_index_buffer(batch.index_buffer.slice(..), IndexFormat::Uint32);

        pass.draw_indexed(0..batch.index_count as u32, 0, 0..1);
    }
}
