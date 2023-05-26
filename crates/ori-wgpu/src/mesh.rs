use std::{mem, num::NonZeroU64};

use bytemuck::{Pod, Zeroable};
use ori_graphics::{math::Vec2, ImageHandle, Mesh, Rect, Vertex};
use wgpu::{
    include_wgsl, util::StagingBelt, vertex_attr_array, BindGroup, BindGroupDescriptor,
    BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
    BlendState, Buffer, BufferBindingType, BufferDescriptor, BufferUsages, ColorTargetState,
    ColorWrites, CommandEncoder, Device, FragmentState, IndexFormat, MultisampleState,
    PipelineLayoutDescriptor, RenderPass, RenderPipeline, RenderPipelineDescriptor, ShaderStages,
    TextureFormat, VertexBufferLayout, VertexStepMode,
};

use crate::WgpuImage;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
struct MeshUniforms {
    resolution: Vec2,
}

#[derive(Debug)]
struct Instance {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    index_count: u32,
    image: Option<ImageHandle>,
    clip: Rect,
    draw: bool,
}

impl Instance {
    fn new(device: &Device) -> Self {
        Self {
            vertex_buffer: MeshPipeline::create_vertex_buffer(device, 512),
            index_buffer: MeshPipeline::create_index_buffer(device, 512),
            index_count: 0,
            image: None,
            clip: Rect::default(),
            draw: false,
        }
    }

    fn recreate_vertex_buffer(&mut self, device: &Device, vertices: u64) {
        self.vertex_buffer = MeshPipeline::create_vertex_buffer(device, vertices);
    }

    fn recreate_index_buffer(&mut self, device: &Device, indices: u64) {
        self.index_buffer = MeshPipeline::create_index_buffer(device, indices);
    }

    fn write_vertex_buffer(
        &mut self,
        device: &Device,
        encoder: &mut CommandEncoder,
        staging_belt: &mut StagingBelt,
        vertices: &[Vertex],
    ) {
        let bytes = bytemuck::cast_slice(vertices);

        if bytes.is_empty() {
            return;
        }

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

        if bytes.is_empty() {
            return;
        }

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
}

#[derive(Default, Debug)]
struct Layer {
    instances: Vec<Instance>,
    instance_count: usize,
}

impl Layer {
    const fn new() -> Self {
        Self {
            instances: Vec::new(),
            instance_count: 0,
        }
    }
}

pub struct MeshPipeline {
    #[allow(dead_code)]
    bind_group_layout: BindGroupLayout,
    uniform_buffer: Buffer,
    uniform_bind_group: BindGroup,
    pipeline: RenderPipeline,
    layers: Vec<Layer>,
}

impl MeshPipeline {
    pub fn new(
        device: &Device,
        image_bind_group_layout: &BindGroupLayout,
        format: TextureFormat,
    ) -> Self {
        let shader = device.create_shader_module(include_wgsl!("shader/mesh.wgsl"));

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

        Self {
            bind_group_layout,
            uniform_buffer,
            uniform_bind_group,
            pipeline,
            layers: Vec::new(),
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
    ) {
        let uniforms = MeshUniforms {
            resolution: Vec2::new(width as f32, height as f32),
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

    pub fn prepare(
        &mut self,
        device: &Device,
        encoder: &mut CommandEncoder,
        staging_belt: &mut StagingBelt,
        width: u32,
        height: u32,
        layer: usize,
        meshes: &[(&Mesh, Option<Rect>)],
    ) {
        self.write_uniform_buffer(device, encoder, staging_belt, width, height);

        if layer >= self.layers.len() {
            self.layers.resize_with(layer + 1, Layer::new);
        }

        let layer = &mut self.layers[layer];
        layer.instance_count = meshes.len();

        if meshes.len() > layer.instances.len() {
            (layer.instances).resize_with(meshes.len(), || Instance::new(device));
        }

        let screen_rect = Rect::new(Vec2::ZERO, Vec2::new(width as f32, height as f32));

        for ((mesh, clip), instance) in meshes.into_iter().zip(&mut layer.instances) {
            if mesh.vertices.is_empty() || mesh.indices.is_empty() {
                instance.draw = false;
                continue;
            } else {
                instance.draw = true;
            }

            instance.clip = match clip {
                Some(clip) => clip.intersect(screen_rect),
                None => screen_rect,
            };

            instance.write_vertex_buffer(device, encoder, staging_belt, &mesh.vertices);
            instance.write_index_buffer(device, encoder, staging_belt, &mesh.indices);
            instance.image = mesh.image.clone();
            instance.index_count = mesh.indices.len() as u32;
        }
    }

    pub fn render<'a>(
        &'a self,
        pass: &mut RenderPass<'a>,
        layer: usize,
        default_image: &'a WgpuImage,
    ) {
        let layer = &self.layers[layer];

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.uniform_bind_group, &[]);

        for instance in &layer.instances[..layer.instance_count] {
            if !instance.draw {
                continue;
            }

            pass.set_scissor_rect(
                instance.clip.min.x as u32,
                instance.clip.min.y as u32,
                instance.clip.width().floor() as u32,
                instance.clip.height().floor() as u32,
            );

            let image = instance
                .image
                .as_ref()
                .and_then(|image| image.downcast_ref::<WgpuImage>());
            if let Some(image) = image {
                pass.set_bind_group(1, &image.bind_group, &[]);
            } else {
                pass.set_bind_group(1, &default_image.bind_group, &[]);
            }

            pass.set_vertex_buffer(0, instance.vertex_buffer.slice(..));
            pass.set_index_buffer(instance.index_buffer.slice(..), IndexFormat::Uint32);
            pass.draw_indexed(0..instance.index_count, 0, 0..1);
        }
    }
}
