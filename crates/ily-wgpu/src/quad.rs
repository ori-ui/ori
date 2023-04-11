use std::mem;

use bytemuck::{Pod, Zeroable};
use ily_core::Vec2;
use ily_graphics::{Color, Quad};
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

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
struct QuadUniforms {
    resolution: Vec2,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
struct QuadVertex {
    position: Vec2,
    min: Vec2,
    max: Vec2,
    color: Color,
    border_color: Color,
    border_radius: [f32; 4],
    border_width: f32,
}

struct QuadInstance {
    bind_group: BindGroup,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
}

impl QuadInstance {
    pub fn new(device: &Device, pipeline: &QuadPipeline, quad: &Quad) -> Self {
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Quad Bind Group"),
            layout: &pipeline.bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: pipeline.uniform_buffer.as_entire_binding(),
            }],
        });

        let mesh = Self::quad_mesh(quad);
        let vertex_buffer = Self::create_vertex_buffer(device, bytemuck::cast_slice(&mesh));
        let index_buffer = Self::create_index_buffer(device);

        Self {
            bind_group,
            vertex_buffer,
            index_buffer,
        }
    }

    fn quad_mesh(quad: &Quad) -> [QuadVertex; 4] {
        [
            QuadVertex {
                position: quad.rect.top_left(),
                min: quad.rect.min,
                max: quad.rect.max,
                color: quad.background,
                border_color: quad.border_color,
                border_radius: quad.border_radius,
                border_width: quad.border_width,
            },
            QuadVertex {
                position: quad.rect.top_right(),
                min: quad.rect.min,
                max: quad.rect.max,
                color: quad.background,
                border_color: quad.border_color,
                border_radius: quad.border_radius,
                border_width: quad.border_width,
            },
            QuadVertex {
                position: quad.rect.bottom_right(),
                min: quad.rect.min,
                max: quad.rect.max,
                color: quad.background,
                border_color: quad.border_color,
                border_radius: quad.border_radius,
                border_width: quad.border_width,
            },
            QuadVertex {
                position: quad.rect.bottom_left(),
                min: quad.rect.min,
                max: quad.rect.max,
                color: quad.background,
                border_color: quad.border_color,
                border_radius: quad.border_radius,
                border_width: quad.border_width,
            },
        ]
    }

    pub fn update(&mut self, device: &Device, queue: &Queue, quad: &Quad) {
        self.update_vertex_buffer(device, queue, quad);
    }

    fn update_vertex_buffer(&mut self, device: &Device, queue: &Queue, quad: &Quad) {
        let mesh = Self::quad_mesh(quad);
        let data = bytemuck::cast_slice(&mesh);

        if self.vertex_buffer.size() < data.len() as u64 {
            self.vertex_buffer = Self::create_vertex_buffer(device, data);
        } else {
            queue.write_buffer(&self.vertex_buffer, 0, data);
        }
    }

    fn create_vertex_buffer(device: &Device, contents: &[u8]) -> Buffer {
        device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Quad Vertex Buffer"),
            contents,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        })
    }

    fn create_index_buffer(device: &Device) -> Buffer {
        device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Quad Index Buffer"),
            contents: bytemuck::cast_slice::<u32, _>(&[0, 1, 2, 2, 3, 0]),
            usage: BufferUsages::INDEX,
        })
    }
}

pub struct QuadPipeline {
    pub bind_group_layout: BindGroupLayout,
    pub uniform_buffer: Buffer,
    pub pipeline: RenderPipeline,
    instances: Vec<QuadInstance>,
}

impl QuadPipeline {
    pub fn new(device: &Device, format: TextureFormat) -> Self {
        let shader = device.create_shader_module(include_wgsl!("quad.wgsl"));

        let uniform_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Quad Uniform Buffer"),
            size: mem::size_of::<QuadUniforms>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Quad Bind Group Layout"),
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
            label: Some("Quad Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Quad Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vertex",
                buffers: &[VertexBufferLayout {
                    array_stride: mem::size_of::<QuadVertex>() as u64,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &vertex_attr_array![
                        0 => Float32x2,
                        1 => Float32x2,
                        2 => Float32x2,
                        3 => Float32x4,
                        4 => Float32x4,
                        5 => Float32x4,
                        6 => Float32,
                    ],
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

    pub fn prepare_quad(&mut self, device: &Device, queue: &Queue, quad: &Quad, index: usize) {
        if let Some(instance) = self.instances.get_mut(index) {
            instance.update(device, queue, quad);
        } else {
            let instance = QuadInstance::new(device, self, quad);
            self.instances.push(instance);
        }
    }

    pub fn set_size(&self, queue: &Queue, width: u32, height: u32) {
        let uniforms = QuadUniforms {
            resolution: Vec2::new(width as f32, height as f32),
        };

        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
    }

    pub fn render<'a>(&'a self, pass: &mut RenderPass<'a>, index: usize) {
        let instance = &self.instances[index];
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &instance.bind_group, &[]);
        pass.set_vertex_buffer(0, instance.vertex_buffer.slice(..));
        pass.set_index_buffer(instance.index_buffer.slice(..), IndexFormat::Uint32);
        pass.draw_indexed(0..6, 0, 0..1);
    }
}
