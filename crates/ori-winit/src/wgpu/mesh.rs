use core::slice;
use std::{mem, sync::Arc};

use bytemuck::{bytes_of, cast_slice, Pod, Zeroable};
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

use super::{CachedImage, ImageCache};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Uniforms {
    resolution: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct MeshData {
    translation: [f32; 2],
    matrix: [f32; 4],
}

#[derive(Debug)]
struct Batch {
    index_count: usize,
    data_buffer: Buffer,
    vertex_buffer: Buffer,
    data_index_buffer: Buffer,
    index_buffer: Buffer,
    data_bind_group: BindGroup,
    image: Option<Arc<CachedImage>>,
    clip: Rect,
}

impl Batch {
    fn new(device: &Device, data_layout: &BindGroupLayout) -> Self {
        let data_buffer = Self::create_data_buffer(device, 128);
        let vertex_buffer = Self::create_vertex_buffer(device, 512);
        let data_index_buffer = Self::create_data_index_buffer(device, 512);
        let index_buffer = Self::create_index_buffer(device, 512);
        let data_bind_group = Self::create_data_bind_group(device, data_layout, &data_buffer);

        Self {
            index_count: 0,
            data_buffer,
            data_index_buffer,
            vertex_buffer,
            index_buffer,
            data_bind_group,
            image: None,
            clip: Rect::ZERO,
        }
    }

    fn create_data_buffer(device: &Device, len: usize) -> Buffer {
        device.create_buffer(&BufferDescriptor {
            label: Some("ori_mesh_data_buffer"),
            size: mem::size_of::<MeshData>() as u64 * len as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    fn create_vertex_buffer(device: &Device, len: usize) -> Buffer {
        device.create_buffer(&BufferDescriptor {
            label: Some("ori_mesh_vertex_buffer"),
            size: mem::size_of::<Vertex>() as u64 * len as u64,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    fn create_data_index_buffer(device: &Device, len: usize) -> Buffer {
        device.create_buffer(&BufferDescriptor {
            label: Some("ori_mesh_data_index_buffer"),
            size: mem::size_of::<u32>() as u64 * len as u64,
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

    fn create_data_bind_group(
        device: &Device,
        layout: &BindGroupLayout,
        data_buffer: &Buffer,
    ) -> BindGroup {
        device.create_bind_group(&BindGroupDescriptor {
            label: Some("ori_mesh_data_bind_group"),
            layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: data_buffer.as_entire_binding(),
            }],
        })
    }

    fn resize_data_buffer(&mut self, device: &Device, layout: &BindGroupLayout, len: usize) {
        let size = mem::size_of::<MeshData>() as u64 * len as u64;

        if self.data_buffer.size() < size {
            self.data_buffer = Self::create_data_buffer(device, len);
            self.data_bind_group = Self::create_data_bind_group(device, layout, &self.data_buffer);
        }
    }

    fn resize_vertex_buffer(&mut self, device: &Device, len: usize) {
        let size = mem::size_of::<Vertex>() as u64 * len as u64;

        if self.vertex_buffer.size() < size {
            self.vertex_buffer = Self::create_vertex_buffer(device, len);
            self.data_index_buffer = Self::create_data_index_buffer(device, len);
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
    data_layout: BindGroupLayout,
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

        let data_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("ori_mesh_data_layout"),
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
            bind_group_layouts: &[&uniform_layout, &data_layout, image_layout],
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
                        attributes: &vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Float32x4],
                    },
                    VertexBufferLayout {
                        array_stride: 4,
                        step_mode: VertexStepMode::Vertex,
                        attributes: &vertex_attr_array![3 => Uint32],
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
            data_layout,
            pipeline,
            uniform_buffer,
            uniform_bind_group,
        }
    }

    fn create_uniform_buffer(device: &Device) -> Buffer {
        device.create_buffer(&BufferDescriptor {
            label: Some("ori_mesh_uniform_buffer"),
            size: mem::size_of::<MeshData>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn prepare(
        &mut self,
        device: &Device,
        queue: &Queue,
        cache: &mut ImageCache,
        index: usize,
        meshes: &[(&Mesh, Affine)],
        clip: Rect,
        resolution: Size,
    ) {
        let uniforms = Uniforms {
            resolution: resolution.into(),
        };

        queue.write_buffer(&self.uniform_buffer, 0, bytes_of(&uniforms));

        if index >= self.batches.len() {
            let len = index + 1;
            let layout = &self.data_layout;
            self.batches.resize_with(len, || Batch::new(device, layout));
        }

        let index_count = meshes.iter().map(|(mesh, _)| mesh.indices.len()).sum();
        let vertex_count = meshes.iter().map(|(mesh, _)| mesh.vertices.len()).sum();

        let batch = &mut self.batches[index];
        batch.resize_data_buffer(device, &self.data_layout, meshes.len());
        batch.resize_index_buffer(device, index_count);
        batch.resize_vertex_buffer(device, vertex_count);
        batch.index_count = index_count;
        batch.clip = clip.clamp(Rect::min_size(Point::ZERO, resolution)).round();

        match meshes[0].0.image {
            Some(ref image) => batch.image = Some(cache.get(device, queue, image)),
            None => batch.image = Some(cache.fallback(device, queue)),
        }

        let mut datas = Vec::with_capacity(meshes.len());
        let mut vertices = Vec::with_capacity(vertex_count);
        let mut data_indices = Vec::with_capacity(vertex_count);
        let mut indices = Vec::with_capacity(index_count);

        for (i, (mesh, transform)) in meshes.iter().enumerate() {
            let data = MeshData {
                translation: transform.translation.into(),
                matrix: transform.matrix.into(),
            };

            let vertex_offset = vertices.len() as u32;

            datas.push(data);
            vertices.extend_from_slice(&mesh.vertices);
            data_indices.resize(data_indices.len() + mesh.vertices.len(), i as u32);

            for index in &mesh.indices {
                indices.push(index + vertex_offset);
            }
        }

        let vertex_bytes = unsafe {
            slice::from_raw_parts(
                vertices.as_ptr() as *const u8,
                vertices.len() * mem::size_of::<Vertex>(),
            )
        };

        queue.write_buffer(&batch.data_buffer, 0, cast_slice(&datas));
        queue.write_buffer(&batch.vertex_buffer, 0, vertex_bytes);
        queue.write_buffer(&batch.data_index_buffer, 0, cast_slice(&data_indices));
        queue.write_buffer(&batch.index_buffer, 0, cast_slice(&indices));
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
        pass.set_bind_group(1, &batch.data_bind_group, &[]);
        pass.set_bind_group(2, image_bind_group, &[]);

        pass.set_vertex_buffer(0, batch.vertex_buffer.slice(..));
        pass.set_vertex_buffer(1, batch.data_index_buffer.slice(..));

        pass.set_index_buffer(batch.index_buffer.slice(..), IndexFormat::Uint32);

        pass.draw_indexed(0..batch.index_count as u32, 0, 0..1);
    }
}
