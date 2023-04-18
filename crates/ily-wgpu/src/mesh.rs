use std::{mem, sync::Arc};

use bytemuck::{Pod, Zeroable};
use ily_core::Vec2;
use ily_graphics::{ImageHandle, Mesh, Vertex};
use wgpu::{
    include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
    vertex_attr_array, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BlendState, Buffer,
    BufferBindingType, BufferDescriptor, BufferUsages, ColorTargetState, ColorWrites, Device,
    FragmentState, IndexFormat, MultisampleState, PipelineLayoutDescriptor, Queue, RenderPass,
    RenderPipeline, RenderPipelineDescriptor, ShaderStages, TextureFormat, VertexBufferLayout,
    VertexStepMode,
};

use crate::WgpuImage;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
struct MeshUniforms {
    resolution: Vec2,
}

struct MeshInstance {
    bind_group: BindGroup,
    image: Option<Arc<WgpuImage>>,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    index_count: u32,
}

impl MeshInstance {
    pub fn new(device: &Device, pipeline: &MeshPipeline, mesh: &Mesh) -> Self {
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Mesh Bind Group"),
            layout: &pipeline.bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: pipeline.uniform_buffer.as_entire_binding(),
            }],
        });

        let vertex_buffer = Self::create_vertex_buffer(device, mesh.vertex_bytes());
        let index_buffer = Self::create_index_buffer(device, mesh.index_bytes());

        Self {
            bind_group,
            image: mesh.image.clone().and_then(ImageHandle::downcast_arc),
            vertex_buffer,
            index_buffer,
            index_count: mesh.indices.len() as u32,
        }
    }

    pub fn update(&mut self, device: &Device, queue: &Queue, mesh: &Mesh) {
        self.update_vertex_buffer(device, queue, mesh);
        self.update_index_buffer(device, queue, mesh);
        self.index_count = mesh.indices.len() as u32;
    }

    fn update_vertex_buffer(&mut self, device: &Device, queue: &Queue, mesh: &Mesh) {
        let data = mesh.vertex_bytes();

        if self.vertex_buffer.size() < data.len() as u64 {
            self.vertex_buffer = Self::create_vertex_buffer(device, data);
        } else {
            queue.write_buffer(&self.vertex_buffer, 0, data);
        }
    }

    fn update_index_buffer(&mut self, device: &Device, queue: &Queue, mesh: &Mesh) {
        let data = mesh.index_bytes();

        if self.index_buffer.size() < data.len() as u64 {
            self.index_buffer = Self::create_index_buffer(device, data);
        } else {
            queue.write_buffer(&self.index_buffer, 0, data);
        }
    }

    fn create_vertex_buffer(device: &Device, contents: &[u8]) -> Buffer {
        device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Mesh Vertex Buffer"),
            contents,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        })
    }

    fn create_index_buffer(device: &Device, contents: &[u8]) -> Buffer {
        device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Mesh Index Buffer"),
            contents,
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
        })
    }
}

pub struct MeshPipeline {
    pub bind_group_layout: BindGroupLayout,
    pub uniform_buffer: Buffer,
    pub pipeline: RenderPipeline,
    instances: Vec<MeshInstance>,
}

impl MeshPipeline {
    pub fn new(
        device: &Device,
        image_bind_group_layout: &BindGroupLayout,
        format: TextureFormat,
    ) -> Self {
        let shader = device.create_shader_module(include_wgsl!("mesh.wgsl"));

        let uniform_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Mesh Uniform Buffer"),
            size: mem::size_of::<MeshUniforms>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Mesh Bind Group Layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Mesh Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout, &image_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Mesh Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vertex",
                buffers: &[VertexBufferLayout {
                    array_stride: mem::size_of::<Vertex>() as u64,
                    step_mode: VertexStepMode::Vertex,
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
            multisample: MultisampleState {
                count: 4,
                ..Default::default()
            },
            depth_stencil: None,
            multiview: None,
        });

        Self {
            bind_group_layout,
            uniform_buffer,
            pipeline,
            instances: Vec::new(),
        }
    }

    pub fn prepare_mesh(&mut self, device: &Device, queue: &Queue, mesh: &Mesh, index: usize) {
        if let Some(instance) = self.instances.get_mut(index) {
            instance.update(device, queue, mesh);
        } else {
            let instance = MeshInstance::new(device, self, mesh);
            self.instances.push(instance);
        }
    }

    pub fn set_size(&self, queue: &Queue, width: u32, height: u32) {
        let uniforms = MeshUniforms {
            resolution: Vec2::new(width as f32, height as f32),
        };

        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
    }

    pub fn render<'a>(
        &'a self,
        pass: &mut RenderPass<'a>,
        default_image: &'a WgpuImage,
        index: usize,
    ) {
        let instance = &self.instances[index];
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &instance.bind_group, &[]);

        if let Some(image) = &instance.image {
            pass.set_bind_group(1, &image.bind_group, &[]);
        } else {
            pass.set_bind_group(1, &default_image.bind_group, &[]);
        }

        pass.set_vertex_buffer(0, instance.vertex_buffer.slice(..));
        pass.set_index_buffer(instance.index_buffer.slice(..), IndexFormat::Uint32);
        pass.draw_indexed(0..instance.index_count, 0, 0..1);
    }
}
