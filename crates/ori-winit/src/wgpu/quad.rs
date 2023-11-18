use std::{mem, sync::Arc};

use ori_core::{
    canvas::Quad,
    layout::{Affine, Point, Rect, Size},
};
use wgpu::{
    include_wgsl, vertex_attr_array, BindGroup, BindGroupDescriptor, BindGroupEntry,
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BlendState,
    Buffer, BufferBindingType, BufferDescriptor, BufferUsages, ColorTargetState, ColorWrites,
    Device, FragmentState, MultisampleState, PipelineLayoutDescriptor, Queue, RenderPass,
    RenderPipeline, RenderPipelineDescriptor, ShaderStages, TextureFormat, VertexBufferLayout,
    VertexState,
};

use crate::WgpuContext;

use super::{
    bytes_of, bytes_of_slice,
    texture::{CachedTexture, TextureCache},
};

#[repr(C)]
#[derive(Clone, Copy)]
struct Uniforms {
    resolution: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct QuadData {
    translation: [f32; 2],
    matrix: [f32; 4],
    min: [f32; 2],
    max: [f32; 2],
    _padding: [u8; 8],
    color: [f32; 4],
    border_radius: [f32; 4],
    border_width: [f32; 4],
    border_color: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

#[derive(Debug)]
struct Batch {
    vertex_count: usize,
    data_buffer: Buffer,
    vertex_buffer: Buffer,
    data_bind_group: BindGroup,
    image: Option<Arc<CachedTexture>>,
    clip: Rect,
}

impl Batch {
    fn new(device: &Device, data_layout: &BindGroupLayout, cap: usize) -> Self {
        let data_buffer = Self::create_data_buffer(device, cap);
        let vertex_buffer = Self::create_vertex_buffer(device, cap);
        let data_bind_group = Self::create_data_bind_group(device, data_layout, &data_buffer);

        Self {
            vertex_count: 0,
            data_buffer,
            vertex_buffer,
            data_bind_group,
            image: None,
            clip: Rect::ZERO,
        }
    }

    fn create_data_buffer(device: &Device, cap: usize) -> Buffer {
        device.create_buffer(&BufferDescriptor {
            label: Some("ori_quad_data_buffer"),
            size: mem::size_of::<QuadData>() as u64 * cap as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    fn create_vertex_buffer(device: &Device, cap: usize) -> Buffer {
        device.create_buffer(&BufferDescriptor {
            label: Some("ori_quad_vertex_buffer"),
            size: mem::size_of::<Vertex>() as u64 * cap as u64 * 6,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    fn create_data_bind_group(
        device: &Device,
        layout: &BindGroupLayout,
        data_buffer: &Buffer,
    ) -> BindGroup {
        device.create_bind_group(&BindGroupDescriptor {
            label: Some("ori_quad_data_bind_group"),
            layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: data_buffer.as_entire_binding(),
            }],
        })
    }

    fn vertices(quad: &Quad) -> [Vertex; 6] {
        [
            Vertex {
                position: quad.rect.top_left().round().into(),
                tex_coords: [0.0, 0.0],
            },
            Vertex {
                position: quad.rect.top_right().round().into(),
                tex_coords: [1.0, 0.0],
            },
            Vertex {
                position: quad.rect.bottom_right().round().into(),
                tex_coords: [1.0, 1.0],
            },
            Vertex {
                position: quad.rect.bottom_left().round().into(),
                tex_coords: [0.0, 1.0],
            },
            Vertex {
                position: quad.rect.bottom_right().round().into(),
                tex_coords: [1.0, 1.0],
            },
            Vertex {
                position: quad.rect.top_left().round().into(),
                tex_coords: [0.0, 0.0],
            },
        ]
    }

    fn resize(&mut self, device: &Device, layout: &BindGroupLayout, len: usize) {
        let size = mem::size_of::<QuadData>() as u64 * len as u64;

        if self.data_buffer.size() < size {
            self.data_buffer = Self::create_data_buffer(device, len);
            self.vertex_buffer = Self::create_vertex_buffer(device, len);
            self.data_bind_group = Self::create_data_bind_group(device, layout, &self.data_buffer);
        }
    }
}

#[derive(Debug)]
pub struct QuadRender {
    batches: Vec<Batch>,
    data_layout: BindGroupLayout,
    pipeline: RenderPipeline,
    uniform_buffer: Buffer,
    uniform_bind_group: BindGroup,
}

impl QuadRender {
    pub fn new(
        device: &Device,
        format: TextureFormat,
        sample_count: u32,
        image_layout: &BindGroupLayout,
    ) -> Self {
        let shader = device.create_shader_module(include_wgsl!("shader/quad.wgsl"));

        let uniform_buffer = Self::create_uniform_buffer(device);

        let uniform_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
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

        let uniform_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("ori_quad_uniform_bind_group"),
            layout: &uniform_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let data_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("ori_quad_data_layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("ori_quad_pipeline_layout"),
            bind_group_layouts: &[&uniform_layout, image_layout, &data_layout],
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
                count: sample_count,
                ..Default::default()
            },
            multiview: None,
        });

        Self {
            batches: Vec::new(),
            data_layout,
            pipeline,
            uniform_buffer,
            uniform_bind_group,
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
            let layout = &self.data_layout;
            (self.batches).resize_with(len, || Batch::new(device, layout, 128));
        }
    }

    fn batch_image(
        context: &WgpuContext,
        cache: &mut TextureCache,
        batch: &[(&Quad, Affine)],
    ) -> Arc<CachedTexture> {
        match batch[0].0.background.texture {
            Some(ref image) => cache.get(context, image),
            None => cache.fallback(context),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn prepare_batch(
        &mut self,
        context: &WgpuContext,
        cache: &mut TextureCache,
        index: usize,
        quads: &[(&Quad, Affine)],
        clip: Rect,
        resolution: Size,
    ) {
        assert!(!quads.is_empty());

        self.write_uniforms(&context.queue, resolution);
        self.resize_batches(&context.device, index + 1);

        let batch = &mut self.batches[index];
        batch.resize(&context.device, &self.data_layout, quads.len());
        batch.vertex_count = quads.len() * 6;
        batch.image = Some(Self::batch_image(context, cache, quads));
        batch.clip = clip.clamp(Rect::min_size(Point::ZERO, resolution)).round();

        let mut datas = Vec::with_capacity(quads.len());
        let mut vertices = Vec::with_capacity(quads.len() * 6);

        for (quad, transform) in quads {
            let data = QuadData {
                translation: transform.translation.into(),
                matrix: transform.matrix.into(),
                min: quad.rect.min.into(),
                max: quad.rect.max.into(),
                _padding: [0; 8],
                color: quad.background.color.into(),
                border_radius: quad.border_radius.into(),
                border_width: quad.border_width.into(),
                border_color: quad.border_color.into(),
            };

            datas.push(data);
            vertices.extend(Batch::vertices(quad));
        }

        unsafe {
            let datas = bytes_of_slice(&datas);
            let vertices = bytes_of_slice(&vertices);

            (context.queue).write_buffer(&batch.data_buffer, 0, datas);
            (context.queue).write_buffer(&batch.vertex_buffer, 0, vertices);
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
        pass.set_bind_group(2, &batch.data_bind_group, &[]);

        pass.set_vertex_buffer(0, batch.vertex_buffer.slice(..));

        pass.draw(0..batch.vertex_count as u32, 0..1);
    }
}
