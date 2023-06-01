use std::sync::Arc;

use ori_graphics::{
    prelude::UVec2, Color, Frame, ImageData, ImageHandle, Primitive, PrimitiveKind, Renderer,
};
use wgpu::{
    util::{DeviceExt, StagingBelt},
    Adapter, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, CommandEncoder, CompositeAlphaMode, Device,
    Extent3d, FilterMode, ImageCopyTexture, ImageDataLayout, LoadOp, Operations, Origin3d,
    PresentMode, Queue, RenderPassColorAttachment, RenderPassDescriptor, SamplerBindingType,
    SamplerDescriptor, ShaderStages, Surface, SurfaceConfiguration, Texture, TextureAspect,
    TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType, TextureUsages,
    TextureView, TextureViewDimension,
};

use crate::{BlitPipeline, MeshPipeline, QuadPipeline, WgpuImage};

#[allow(dead_code)]
pub struct WgpuRenderer {
    device: Arc<Device>,
    queue: Arc<Queue>,
    config: SurfaceConfiguration,
    surface: Surface,
    msaa_texture: Texture,
    image_bind_group_layout: BindGroupLayout,
    default_image: WgpuImage,
    blit_pipeline: BlitPipeline,
    mesh_pipeline: MeshPipeline,
    quad_pipeline: QuadPipeline,
    staging_belt: StagingBelt,
}

impl WgpuRenderer {
    /// Creates a new renderer.
    pub fn new(
        adapter: &Adapter,
        device: Arc<Device>,
        queue: Arc<Queue>,
        surface: Surface,
        width: u32,
        height: u32,
    ) -> Self {
        let mut config = surface.get_default_config(adapter, width, height).unwrap();
        config.format = TextureFormat::Bgra8Unorm;
        config.alpha_mode = CompositeAlphaMode::Auto;
        config.usage |= TextureUsages::TEXTURE_BINDING;
        config.present_mode = PresentMode::AutoVsync;
        surface.configure(&device, &config);

        let msaa_texture = Self::create_msaa_texture(&device, config.format, width, height);

        let image_bind_group_layout = Self::create_image_bind_group_layout(&device);
        let default_image = Self::create_default_image(&device, &queue, &image_bind_group_layout);

        let blit_pipeline = BlitPipeline::new(&device, config.format);
        let mesh_pipeline = MeshPipeline::new(&device, &image_bind_group_layout, config.format);
        let quad_pipeline = QuadPipeline::new(&device, config.format);

        let staging_belt = StagingBelt::new(1024);

        Self {
            device,
            queue,
            config,
            surface,
            msaa_texture,
            blit_pipeline,
            mesh_pipeline,
            quad_pipeline,
            image_bind_group_layout,
            default_image,
            staging_belt,
        }
    }

    fn create_msaa_texture(
        device: &Device,
        format: TextureFormat,
        width: u32,
        height: u32,
    ) -> Texture {
        device.create_texture(&TextureDescriptor {
            label: Some("MSAA Texture"),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 4,
            dimension: TextureDimension::D2,
            format,
            usage: TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        })
    }

    fn create_image_bind_group_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Ily Image Bind Group Layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        })
    }

    fn create_image(
        device: &Device,
        queue: &Queue,
        layout: &BindGroupLayout,
        width: u32,
        height: u32,
        data: &[u8],
    ) -> WgpuImage {
        let texture = device.create_texture_with_data(
            queue,
            &TextureDescriptor {
                label: Some("Ily Texture"),
                size: Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8Unorm,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                view_formats: &[],
            },
            data,
        );
        let view = texture.create_view(&Default::default());
        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("Ily Image Sampler"),
            min_filter: FilterMode::Linear,
            mag_filter: FilterMode::Linear,
            ..Default::default()
        });
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Ily Bind Group"),
            layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
        });

        WgpuImage {
            texture,
            view,
            sampler,
            bind_group,
        }
    }

    fn create_default_image(device: &Device, queue: &Queue, layout: &BindGroupLayout) -> WgpuImage {
        let data = vec![255, 255, 255, 255];
        Self::create_image(device, queue, layout, 1, 1, &data)
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn queue(&self) -> &Queue {
        &self.queue
    }

    pub fn device_arc(&self) -> Arc<Device> {
        self.device.clone()
    }

    pub fn queue_arc(&self) -> Arc<Queue> {
        self.queue.clone()
    }

    #[allow(dead_code)]
    fn blit_texture(
        &mut self,
        encoder: &mut CommandEncoder,
        source: &TextureView,
        target: &TextureView,
    ) {
        (self.blit_pipeline).blit(&self.device, encoder, source, target);
    }

    /// Primitives must be sorted by their z-index.
    pub fn prepare(&mut self, encoder: &mut CommandEncoder, primitives: &[&Primitive]) -> usize {
        let mut z_index = f32::NEG_INFINITY;
        let mut layer = 0;
        let mut quads = Vec::new();
        let mut meshes = Vec::new();

        for primitive in primitives {
            if primitive.z_index != z_index {
                self.quad_pipeline.prepare(
                    &self.device,
                    encoder,
                    &mut self.staging_belt,
                    self.config.width,
                    self.config.height,
                    layer,
                    &quads,
                );

                self.mesh_pipeline.prepare(
                    &self.device,
                    encoder,
                    &mut self.staging_belt,
                    self.config.width,
                    self.config.height,
                    layer,
                    &meshes,
                );

                z_index = primitive.z_index;
                layer += 1;

                quads.clear();
                meshes.clear();
            }

            match primitive.kind {
                PrimitiveKind::Quad(ref quad) => quads.push((quad, primitive.clip)),
                PrimitiveKind::Mesh(ref mesh) => meshes.push((mesh, primitive.clip)),
            }
        }

        self.quad_pipeline.prepare(
            &self.device,
            encoder,
            &mut self.staging_belt,
            self.config.width,
            self.config.height,
            layer,
            &quads,
        );

        self.mesh_pipeline.prepare(
            &self.device,
            encoder,
            &mut self.staging_belt,
            self.config.width,
            self.config.height,
            layer,
            &meshes,
        );

        layer
    }

    fn render(
        &self,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        clear_color: Color,
        layers: usize,
    ) {
        let msaa_view = self.msaa_texture.create_view(&Default::default());

        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Ily Main Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &msaa_view,
                resolve_target: Some(view),
                ops: Operations {
                    load: LoadOp::Clear(wgpu::Color {
                        r: clear_color.r as f64,
                        g: clear_color.g as f64,
                        b: clear_color.b as f64,
                        a: clear_color.a as f64,
                    }),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        for layer in 0..=layers {
            (self.quad_pipeline).render(&mut pass, layer);
            (self.mesh_pipeline).render(&mut pass, layer, &self.default_image);
        }
    }
}

impl Renderer for WgpuRenderer {
    fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        self.msaa_texture = Self::create_msaa_texture(
            &self.device,
            self.config.format,
            self.config.width,
            self.config.height,
        );
    }

    fn create_image(&self, data: &ImageData) -> ImageHandle {
        let image = Self::create_image(
            &self.device,
            &self.queue,
            &self.image_bind_group_layout,
            data.width(),
            data.height(),
            data.pixels(),
        );

        ImageHandle::new(image, data.width(), data.height())
    }

    fn write_image(&self, handle: &ImageHandle, offset: UVec2, data: &ImageData) {
        let Some(image) = handle.downcast_ref::<WgpuImage>() else { return };

        self.queue.write_texture(
            ImageCopyTexture {
                texture: &image.texture,
                mip_level: 0,
                origin: Origin3d {
                    x: offset.x,
                    y: offset.y,
                    z: 0,
                },
                aspect: TextureAspect::All,
            },
            data.pixels(),
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(data.width() * 4),
                rows_per_image: None,
            },
            Extent3d {
                width: data.width(),
                height: data.height(),
                depth_or_array_layers: 1,
            },
        );
    }

    fn render_frame(&mut self, frame: &Frame, clear_color: Color) {
        let target = self.surface.get_current_texture().unwrap();
        let view = target.texture.create_view(&Default::default());

        let mut encoder = self.device.create_command_encoder(&Default::default());

        // create render pass
        let mut primitives: Vec<_> = frame.primitives().iter().collect();
        primitives.sort_by(|a, b| a.z_index.partial_cmp(&b.z_index).unwrap());

        let layers = self.prepare(&mut encoder, &primitives);
        self.render(&mut encoder, &view, clear_color, layers);

        // submit and present
        self.staging_belt.finish();
        self.queue.submit(Some(encoder.finish()));
        target.present();
        self.staging_belt.recall();
    }
}
