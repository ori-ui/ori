use std::mem;

use bytemuck::{bytes_of, cast_slice, Pod, Zeroable};
use ori_core::{
    canvas::Quad,
    layout::{Affine, Rect, Size},
};
use wgpu::{
    include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
    vertex_attr_array, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutEntry, BindingType, BlendState, Buffer, BufferBindingType, BufferDescriptor,
    BufferUsages, ColorTargetState, ColorWrites, Device, FragmentState, IndexFormat,
    MultisampleState, Queue, RenderPass, RenderPipeline, RenderPipelineDescriptor, ShaderStages,
    TextureFormat, VertexBufferLayout, VertexState,
};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Uniforms {
    resolution: [f32; 2],
    translation: [f32; 2],
    matrix: [f32; 4],
    min: [f32; 2],
    max: [f32; 2],
    color: [f32; 4],
    border_radius: [f32; 4],
    border_width: [f32; 4],
    border_color: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

#[derive(Debug)]
struct Instance {
    uniform_buffer: Buffer,
    vertex_buffer: Buffer,
    uniform_bind_group: BindGroup,
    clip: Rect,
}

impl Instance {
    fn new(device: &Device, uniform_layout: &BindGroupLayout) -> Self {
        let uniform_buffer = Self::create_uniform_buffer(device);
        let vertex_buffer = Self::create_vertex_buffer(device);
        let uniform_bind_group =
            Self::create_uniform_bind_group(device, uniform_layout, &uniform_buffer);

        Self {
            uniform_buffer,
            vertex_buffer,
            uniform_bind_group,
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

    fn create_vertex_buffer(device: &Device) -> Buffer {
        device.create_buffer(&BufferDescriptor {
            label: Some("ori_quad_vertex_buffer"),
            size: mem::size_of::<Vertex>() as u64 * 6,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
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

    fn vertices(quad: &Quad) -> [Vertex; 6] {
        [
            Vertex {
                position: quad.rect.top_left().into(),
                tex_coords: [0.0, 0.0],
            },
            Vertex {
                position: quad.rect.top_right().into(),
                tex_coords: [1.0, 0.0],
            },
            Vertex {
                position: quad.rect.bottom_right().into(),
                tex_coords: [1.0, 1.0],
            },
            Vertex {
                position: quad.rect.bottom_left().into(),
                tex_coords: [0.0, 1.0],
            },
            Vertex {
                position: quad.rect.bottom_right().into(),
                tex_coords: [1.0, 1.0],
            },
            Vertex {
                position: quad.rect.top_left().into(),
                tex_coords: [0.0, 0.0],
            },
        ]
    }

    fn write_uniform_buffer(
        &self,
        queue: &Queue,
        quad: &Quad,
        transform: Affine,
        resolution: Size,
    ) {
        let uniforms = Uniforms {
            resolution: resolution.into(),
            translation: transform.translation.into(),
            matrix: transform.matrix.to_cols_array(),
            min: quad.rect.min.into(),
            max: quad.rect.max.into(),
            color: quad.color.into(),
            border_radius: quad.border_radius.into(),
            border_width: quad.border_width.into(),
            border_color: quad.border_color.into(),
        };

        queue.write_buffer(&self.uniform_buffer, 0, bytes_of(&uniforms));
    }

    fn write_vertex_buffer(&self, queue: &Queue, quad: &Quad) {
        let vertices = Self::vertices(quad);
        queue.write_buffer(&self.vertex_buffer, 0, cast_slice(&vertices));
    }
}

#[derive(Debug)]
pub struct QuadRender {
    instances: Vec<Instance>,
    uniform_layout: BindGroupLayout,
    pipeline: RenderPipeline,
    index_buffer: Buffer,
}

impl QuadRender {
    pub fn new(device: &Device, format: TextureFormat) -> Self {
        let shader = device.create_shader_module(include_wgsl!("shader/quad.wgsl"));

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
            bind_group_layouts: &[&uniform_layout],
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
                    attributes: &vertex_attr_array![0 => Float32x2, 1 => Float32x2],
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
            index_buffer: Self::create_index_buffer(device),
        }
    }

    fn create_index_buffer(device: &Device) -> Buffer {
        let indices = [0u32, 1, 2, 2, 3, 0];

        device.create_buffer_init(&BufferInitDescriptor {
            label: Some("ori_quad_index_buffer"),
            contents: cast_slice(&indices),
            usage: BufferUsages::INDEX,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn prepare(
        &mut self,
        device: &Device,
        queue: &Queue,
        index: usize,
        quad: &Quad,
        transform: Affine,
        clip: Rect,
        resolution: Size,
    ) {
        if index >= self.instances.len() {
            let layout = &self.uniform_layout;
            (self.instances).resize_with(index + 1, || Instance::new(device, layout));
        }

        let instance = &mut self.instances[index];
        instance.write_vertex_buffer(queue, quad);
        instance.write_uniform_buffer(queue, quad, transform, resolution);
        instance.clip = clip;
    }

    pub fn render<'a>(&'a self, pass: &mut RenderPass<'a>, index: usize) {
        let instance = &self.instances[index];

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &instance.uniform_bind_group, &[]);
        pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);
        pass.set_vertex_buffer(0, instance.vertex_buffer.slice(..));
        pass.draw_indexed(0..6, 0, 0..1);
    }
}
