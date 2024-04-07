use std::{mem, sync::Arc};

use ori_core::{
    canvas::{Batch, Vertex},
    layout::{Rect, Size},
};
use wgpu::{
    include_wgsl, vertex_attr_array, BindGroup, BindGroupDescriptor, BindGroupEntry,
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BlendState,
    Buffer, BufferBindingType, BufferDescriptor, BufferUsages, ColorTargetState, ColorWrites,
    Device, FragmentState, IndexFormat, MultisampleState, PipelineLayoutDescriptor, Queue,
    RenderPass, RenderPipeline, RenderPipelineDescriptor, ShaderStages, TextureFormat,
    VertexBufferLayout, VertexState, VertexStepMode,
};

use crate::{CachedTexture, TextureCache, WgpuContext};

unsafe fn bytes_of<T>(data: &T) -> &[u8] {
    std::slice::from_raw_parts(data as *const _ as *const u8, std::mem::size_of::<T>())
}

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
struct PreparedBatch {
    index_count: usize,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    image: Option<Arc<CachedTexture>>,
    clip: Rect,
}

impl PreparedBatch {
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
    batches: Vec<PreparedBatch>,
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
            self.batches.resize_with(len, || PreparedBatch::new(device));
        }
    }

    fn batch_image(
        context: &WgpuContext,
        cache: &mut TextureCache,
        batch: &Batch,
    ) -> Arc<CachedTexture> {
        match batch.mesh.texture {
            Some(ref image) => cache.get(context, image),
            None => cache.fallback(context),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn prepare_batch(
        &mut self,
        context: &WgpuContext,
        cache: &mut TextureCache,
        batch: &Batch,
        resolution: Size,
    ) {
        self.write_uniforms(&context.queue, resolution);
        self.resize_batches(&context.device, batch.index + 1);

        let index_count = batch.mesh.indices.len();
        let vertex_count = batch.mesh.vertices.len();

        let prepared = &mut self.batches[batch.index];
        prepared.resize_index_buffer(&context.device, index_count);
        prepared.resize_vertex_buffer(&context.device, vertex_count);
        prepared.index_count = index_count;
        prepared.image = Some(Self::batch_image(context, cache, batch));
        prepared.clip = batch.clip.clamp(resolution).round();

        let vertex_bytes = batch.mesh.vertex_bytes();
        let index_bytes = batch.mesh.index_bytes();

        (context.queue).write_buffer(&prepared.vertex_buffer, 0, vertex_bytes);
        (context.queue).write_buffer(&prepared.index_buffer, 0, index_bytes);
    }

    pub fn render_batch<'a>(&'a self, pass: &mut RenderPass<'a>, index: usize, scale_factor: f32) {
        let batch = &self.batches[index];

        let image_bind_group = &batch.image.as_ref().unwrap().bind_group;

        let clip_x = batch.clip.min.x * scale_factor;
        let clip_y = batch.clip.min.y * scale_factor;
        let clip_width = batch.clip.width() * scale_factor;
        let clip_height = batch.clip.height() * scale_factor;

        pass.set_scissor_rect(
            clip_x.round() as u32,
            clip_y.round() as u32,
            clip_width.round() as u32,
            clip_height.round() as u32,
        );

        pass.set_pipeline(&self.pipeline);

        pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        pass.set_bind_group(1, image_bind_group, &[]);

        pass.set_vertex_buffer(0, batch.vertex_buffer.slice(..));
        pass.set_index_buffer(batch.index_buffer.slice(..), IndexFormat::Uint32);

        pass.draw_indexed(0..batch.index_count as u32, 0, 0..1);
    }
}
