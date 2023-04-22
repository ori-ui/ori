use std::{mem, num::NonZeroU64};

use bytemuck::{Pod, Zeroable};
use ily_core::Vec2;
use ily_graphics::{Mesh, Rect, Vertex};
use wgpu::{
    include_wgsl, util::StagingBelt, vertex_attr_array, BindGroup, BindGroupDescriptor,
    BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
    BlendState, Buffer, BufferBindingType, BufferDescriptor, BufferUsages, ColorTargetState,
    ColorWrites, CommandEncoder, Device, FragmentState, IndexFormat, LoadOp, MultisampleState,
    Operations, PipelineLayoutDescriptor, RenderPassColorAttachment, RenderPassDescriptor,
    RenderPipeline, RenderPipelineDescriptor, ShaderStages, TextureFormat, TextureView,
    VertexBufferLayout, VertexStepMode,
};

use crate::WgpuImage;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
struct MeshUniforms {
    resolution: Vec2,
    depth: f32,
}

pub struct MeshPipeline {
    pub bind_group_layout: BindGroupLayout,
    pub uniform_buffer: Buffer,
    pub uniform_bind_group: BindGroup,
    pub pipeline: RenderPipeline,
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
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

        let uniform_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Mesh Uniform Bind Group"),
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
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

        let vertex_buffer = Self::create_vertex_buffer(device, 512);
        let index_buffer = Self::create_index_buffer(device, 512);

        Self {
            bind_group_layout,
            uniform_buffer,
            uniform_bind_group,
            pipeline,
            vertex_buffer,
            index_buffer,
        }
    }

    fn create_vertex_buffer(device: &Device, vertices: u64) -> Buffer {
        device.create_buffer(&BufferDescriptor {
            label: Some("Mesh Vertex Buffer"),
            size: mem::size_of::<Vertex>() as u64 * vertices,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    fn create_index_buffer(device: &Device, indices: u64) -> Buffer {
        device.create_buffer(&BufferDescriptor {
            label: Some("Mesh Index Buffer"),
            size: mem::size_of::<u32>() as u64 * indices,
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
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
        let uniforms = MeshUniforms {
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

    fn recreate_vertex_buffer(&mut self, device: &Device, vertices: u64) {
        self.vertex_buffer = Self::create_vertex_buffer(device, vertices);
    }

    fn recreate_index_buffer(&mut self, device: &Device, indices: u64) {
        self.index_buffer = Self::create_index_buffer(device, indices);
    }

    fn write_vertex_buffer(
        &mut self,
        device: &Device,
        encoder: &mut CommandEncoder,
        staging_belt: &mut StagingBelt,
        vertices: &[Vertex],
    ) {
        let bytes = bytemuck::cast_slice(vertices);

        if self.vertex_buffer.size() < bytes.len() as u64 {
            self.recreate_vertex_buffer(device, vertices.len() as u64);
        }

        let mut buffer = staging_belt.write_buffer(
            encoder,
            &self.vertex_buffer,
            0,
            NonZeroU64::new(bytes.len() as u64).unwrap(),
            device,
        );

        buffer.copy_from_slice(bytes);
    }

    fn write_index_buffer(
        &mut self,
        device: &Device,
        encoder: &mut CommandEncoder,
        staging_belt: &mut StagingBelt,
        indices: &[u32],
    ) {
        let bytes = bytemuck::cast_slice(indices);

        if self.index_buffer.size() < bytes.len() as u64 {
            self.recreate_index_buffer(device, indices.len() as u64);
        }

        let mut buffer = staging_belt.write_buffer(
            encoder,
            &self.index_buffer,
            0,
            NonZeroU64::new(bytes.len() as u64).unwrap(),
            device,
        );

        buffer.copy_from_slice(bytes);
    }

    pub fn render(
        &mut self,
        device: &Device,
        encoder: &mut CommandEncoder,
        staging_belt: &mut StagingBelt,
        view: &TextureView,
        msaa: &TextureView,
        width: u32,
        height: u32,
        default_image: &WgpuImage,
        mesh: &Mesh,
        depth: f32,
        clip: Option<Rect>,
    ) {
        if mesh.vertices.is_empty() || mesh.indices.is_empty() {
            return;
        }

        self.write_uniform_buffer(device, encoder, staging_belt, width, height, depth);
        self.write_vertex_buffer(device, encoder, staging_belt, &mesh.vertices);
        self.write_index_buffer(device, encoder, staging_belt, &mesh.indices);

        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Mesh Render Pass"),
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

        if let Some(image) = &mesh.image {
            let image = image.downcast_ref::<WgpuImage>().unwrap();
            pass.set_bind_group(1, &image.bind_group, &[]);
        } else {
            pass.set_bind_group(1, &default_image.bind_group, &[]);
        }

        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);
        pass.draw_indexed(0..mesh.indices.len() as u32, 0, 0..1);
    }
}
