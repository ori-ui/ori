use std::{mem, num::NonZeroU64};

use bytemuck::{Pod, Zeroable};
use ily_core::Vec2;
use ily_graphics::{Color, Quad, Rect};
use wgpu::{
    include_wgsl,
    util::{BufferInitDescriptor, DeviceExt, StagingBelt},
    vertex_attr_array, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BlendState, Buffer,
    BufferBindingType, BufferDescriptor, BufferUsages, ColorTargetState, ColorWrites,
    CommandEncoder, Device, FragmentState, IndexFormat, LoadOp, MultisampleState, Operations,
    PipelineLayoutDescriptor, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, ShaderStages, TextureFormat, TextureView, VertexBufferLayout,
    VertexStepMode,
};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
struct QuadUniforms {
    resolution: Vec2,
    depth: f32,
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

pub struct QuadPipeline {
    pub bind_group_layout: BindGroupLayout,
    pub uniform_buffer: Buffer,
    pub uniform_bind_group: BindGroup,
    pub pipeline: RenderPipeline,
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
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

        let uniform_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Quad Uniform Bind Group"),
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
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

        let vertex_buffer = Self::create_vertex_buffer(device);
        let index_buffer = Self::create_index_buffer(device);

        Self {
            bind_group_layout,
            uniform_buffer,
            pipeline,
            uniform_bind_group,
            vertex_buffer,
            index_buffer,
        }
    }

    fn create_vertex_buffer(device: &Device) -> Buffer {
        device.create_buffer(&BufferDescriptor {
            label: Some("Quad Vertex Buffer"),
            size: mem::size_of::<QuadVertex>() as u64 * 4,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    fn create_index_buffer(device: &Device) -> Buffer {
        let indices = [0, 1, 2, 2, 3, 0];

        device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Quad Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: BufferUsages::INDEX,
        })
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

    fn write_uniform_buffer(
        &self,
        device: &Device,
        encoder: &mut CommandEncoder,
        staging_belt: &mut StagingBelt,
        width: u32,
        height: u32,
        depth: f32,
    ) {
        let uniforms = QuadUniforms {
            resolution: Vec2::new(width as f32, height as f32),
            depth,
        };

        let bytes = bytemuck::bytes_of(&uniforms);

        let mut buffer = staging_belt.write_buffer(
            encoder,
            &self.uniform_buffer,
            0,
            NonZeroU64::new(bytes.len() as u64).unwrap(),
            device,
        );
        buffer.copy_from_slice(bytes);
    }

    fn write_vertex_buffer(
        &self,
        device: &Device,
        encoder: &mut CommandEncoder,
        staging_belt: &mut StagingBelt,
        quad: &Quad,
    ) {
        let vertices = Self::quad_mesh(quad);
        let bytes = bytemuck::cast_slice(&vertices);

        let mut buffer = staging_belt.write_buffer(
            encoder,
            &self.vertex_buffer,
            0,
            NonZeroU64::new(bytes.len() as u64).unwrap(),
            device,
        );
        buffer.copy_from_slice(bytes);
    }

    pub fn render(
        &self,
        device: &Device,
        encoder: &mut CommandEncoder,
        staging_belt: &mut StagingBelt,
        view: &TextureView,
        msaa: &TextureView,
        width: u32,
        height: u32,
        quad: &Quad,
        depth: f32,
        clip: Option<Rect>,
    ) {
        self.write_uniform_buffer(device, encoder, staging_belt, width, height, depth);
        self.write_vertex_buffer(device, encoder, staging_belt, quad);

        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Quad Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &msaa,
                resolve_target: Some(view),
                ops: Operations {
                    load: LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        if let Some(clip) = clip {
            pass.set_scissor_rect(
                clip.min.x as u32,
                clip.min.y as u32,
                clip.width() as u32,
                clip.height() as u32,
            );
        }

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);
        pass.draw_indexed(0..6, 0, 0..1);
    }
}
