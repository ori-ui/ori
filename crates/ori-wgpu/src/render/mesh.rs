use std::{mem, sync::Arc};

use bytemuck::{bytes_of, Pod, Zeroable};
use ori_core::{
    canvas::{Mesh, Vertex},
    layout::{Affine, Rect, Size},
};
use wgpu::{
    include_wgsl, vertex_attr_array, BindGroup, BindGroupDescriptor, BindGroupEntry,
    BindGroupLayout, BindGroupLayoutEntry, BindingType, BlendState, Buffer, BufferBindingType,
    BufferDescriptor, BufferUsages, ColorTargetState, ColorWrites, Device, FragmentState,
    IndexFormat, MultisampleState, Queue, RenderPass, RenderPipeline, RenderPipelineDescriptor,
    ShaderStages, TextureFormat, VertexBufferLayout, VertexState,
};

use super::{CachedImage, ImageCache};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Uniforms {
    resolution: [f32; 2],
    translation: [f32; 2],
    matrix: [f32; 4],
}

#[derive(Debug)]
struct Instance {
    uniform_buffer: Buffer,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    uniform_bind_group: BindGroup,
    image: Option<Arc<CachedImage>>,
    clip: Rect,
}

impl Instance {
    fn new(device: &Device, uniform_layout: &BindGroupLayout) -> Self {
        let uniform_buffer = Self::create_uniform_buffer(device);
        let vertex_buffer = Self::create_vertex_buffer(device, 512);
        let index_buffer = Self::create_index_buffer(device, 512);
        let uniform_bind_group =
            Self::create_uniform_bind_group(device, uniform_layout, &uniform_buffer);

        Self {
            uniform_buffer,
            vertex_buffer,
            index_buffer,
            uniform_bind_group,
            image: None,
            clip: Rect::ZERO,
        }
    }

    fn create_uniform_buffer(device: &Device) -> Buffer {
        device.create_buffer(&BufferDescriptor {
            label: Some("ori_quad_uniform_buffer"),
            size: mem::size_of::<Uniforms>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    fn create_vertex_buffer(device: &Device, len: usize) -> Buffer {
        device.create_buffer(&BufferDescriptor {
            label: Some("ori_quad_vertex_buffer"),
            size: mem::size_of::<Vertex>() as u64 * len as u64,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    fn create_index_buffer(device: &Device, len: usize) -> Buffer {
        device.create_buffer(&BufferDescriptor {
            label: Some("ori_quad_index_buffer"),
            size: mem::size_of::<u32>() as u64 * len as u64,
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    fn create_uniform_bind_group(
        device: &Device,
        layout: &BindGroupLayout,
        buffer: &Buffer,
    ) -> BindGroup {
        device.create_bind_group(&BindGroupDescriptor {
            label: Some("ori_quad_uniform_bind_group"),
            layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        })
    }

    fn write_uniform_buffer(&self, queue: &Queue, transform: Affine, resolution: Size) {
        let uniforms = Uniforms {
            resolution: resolution.into(),
            translation: transform.translation.into(),
            matrix: transform.matrix.to_cols_array(),
        };

        queue.write_buffer(&self.uniform_buffer, 0, bytes_of(&uniforms));
    }

    fn write_vertex_buffer(&mut self, device: &Device, queue: &Queue, mesh: &Mesh) {
        if self.vertex_buffer.size() < mesh.vertex_bytes().len() as u64 {
            self.vertex_buffer = Self::create_vertex_buffer(device, mesh.vertices.len());
        }

        queue.write_buffer(&self.vertex_buffer, 0, mesh.vertex_bytes());
    }

    fn write_index_buffer(&mut self, device: &Device, queue: &Queue, mesh: &Mesh) {
        if self.index_buffer.size() < mesh.index_bytes().len() as u64 {
            self.index_buffer = Self::create_index_buffer(device, mesh.indices.len());
        }

        queue.write_buffer(&self.index_buffer, 0, mesh.index_bytes());
    }
}

#[derive(Debug)]
pub struct MeshRender {
    instances: Vec<Instance>,
    uniform_layout: BindGroupLayout,
    pipeline: RenderPipeline,
}

impl MeshRender {
    pub fn new(device: &Device, format: TextureFormat, image_layout: &BindGroupLayout) -> Self {
        let shader = device.create_shader_module(include_wgsl!("shader/mesh.wgsl"));

        let uniform_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ori_quad_uniform_layout"),
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

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
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
                buffers: &[VertexBufferLayout {
                    array_stride: mem::size_of::<Vertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Float32x4],
                }],
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
                count: 4,
                ..Default::default()
            },
            multiview: None,
        });

        Self {
            instances: Vec::new(),
            uniform_layout,
            pipeline,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn prepare(
        &mut self,
        device: &Device,
        queue: &Queue,
        cache: &mut ImageCache,
        index: usize,
        mesh: &Mesh,
        transform: Affine,
        clip: Rect,
        resolution: Size,
    ) {
        if index >= self.instances.len() {
            let layout = &self.uniform_layout;
            (self.instances).resize_with(index + 1, || Instance::new(device, layout));
        }

        let image = match mesh.image {
            Some(ref image) => cache.get(device, queue, image),
            None => cache.fallback(device, queue),
        };

        let instance = &mut self.instances[index];
        instance.write_vertex_buffer(device, queue, mesh);
        instance.write_index_buffer(device, queue, mesh);
        instance.write_uniform_buffer(queue, transform, resolution);
        instance.image = Some(image);
        instance.clip = clip;
    }

    pub fn render<'a>(&'a self, pass: &mut RenderPass<'a>, index: usize, mesh: &Mesh) {
        let instance = &self.instances[index];

        let image_bind_group = &instance.image.as_ref().unwrap().bind_group;

        pass.set_scissor_rect(
            instance.clip.min.x as u32,
            instance.clip.min.y as u32,
            instance.clip.width() as u32,
            instance.clip.height() as u32,
        );

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &instance.uniform_bind_group, &[]);
        pass.set_bind_group(1, image_bind_group, &[]);
        pass.set_index_buffer(instance.index_buffer.slice(..), IndexFormat::Uint32);
        pass.set_vertex_buffer(0, instance.vertex_buffer.slice(..));
        pass.draw_indexed(0..mesh.indices.len() as u32, 0, 0..1);
    }
}
