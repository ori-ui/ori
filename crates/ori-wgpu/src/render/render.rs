use std::sync::Arc;

use ori_core::{Fragment, Primitive, Scene, Size};
use wgpu::{
    CommandEncoder, CommandEncoderDescriptor, CompositeAlphaMode, Device, LoadOp, Operations,
    PresentMode, Queue, RenderPass, RenderPassColorAttachment, RenderPassDescriptor, Surface,
    SurfaceConfiguration, SurfaceError, TextureDimension, TextureFormat, TextureUsages,
    TextureView,
};

use crate::RenderError;

use super::{ImageCache, MeshRender, QuadRender, RenderInstance};

#[derive(Debug)]
pub struct Render {
    device: Arc<Device>,
    queue: Arc<Queue>,
    surface: Surface,
    config: SurfaceConfiguration,
    image: ImageCache,
    msaa: TextureView,
    quad: QuadRender,
    mesh: MeshRender,
}

impl Render {
    pub fn new(
        instance: &RenderInstance,
        surface: Surface,
        width: u32,
        height: u32,
    ) -> Result<Self, RenderError> {
        let device = instance.device.clone();
        let queue = instance.queue.clone();

        let config = surface.get_default_config(&instance.adapter, width, height);
        let mut config = config.ok_or(RenderError::SurfaceIncompatible)?;
        config.format = TextureFormat::Bgra8Unorm;
        config.alpha_mode = CompositeAlphaMode::Auto;
        config.present_mode = PresentMode::AutoVsync;
        surface.configure(&device, &config);

        let cache = ImageCache::new(&device);

        let msaa = Self::create_msaa(&device, config.format, width, height);
        let quad = QuadRender::new(&device, config.format);
        let mesh = MeshRender::new(&device, config.format, cache.bind_group_layout());

        Ok(Self {
            device,
            queue,
            surface,
            config,
            image: cache,
            msaa,
            quad,
            mesh,
        })
    }

    fn create_msaa(device: &Device, format: TextureFormat, width: u32, height: u32) -> TextureView {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ori_msaa_texture"),
            size: wgpu::Extent3d {
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
        });

        texture.create_view(&wgpu::TextureViewDescriptor::default())
    }

    pub fn clean(&mut self) {
        self.image.clean();
    }

    fn size(&self) -> Size {
        Size::new(self.config.width as f32, self.config.height as f32)
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if self.config.width == width && self.config.height == height {
            return;
        }

        self.config.width = width;
        self.config.height = height;

        self.surface.configure(&self.device, &self.config);
        self.msaa = Self::create_msaa(&self.device, self.config.format, width, height);
    }

    fn prepare_fragments(&mut self, fragments: &[Fragment]) {
        let mut quad_index = 0;
        let mut mesh_index = 0;

        let resolution = self.size();

        for fragment in fragments {
            match fragment.primitive {
                Primitive::Quad(quad) => {
                    self.quad.prepare(
                        &self.device,
                        &self.queue,
                        quad_index,
                        &quad,
                        fragment.transform,
                        fragment.clip,
                        resolution,
                    );

                    quad_index += 1;
                }
                Primitive::Mesh(ref mesh) => {
                    self.mesh.prepare(
                        &self.device,
                        &self.queue,
                        &mut self.image,
                        mesh_index,
                        mesh,
                        fragment.transform,
                        fragment.clip,
                        resolution,
                    );

                    mesh_index += 1;
                }
            }
        }
    }

    fn render_fragments<'a>(&'a self, pass: &mut RenderPass<'a>, fragments: &[Fragment]) {
        let mut quad_index = 0;
        let mut mesh_index = 0;

        for fragment in fragments {
            match fragment.primitive {
                Primitive::Quad(_) => {
                    self.quad.render(pass, quad_index);

                    quad_index += 1;
                }
                Primitive::Mesh(ref mesh) => {
                    self.mesh.render(pass, mesh_index, mesh);

                    mesh_index += 1;
                }
            }
        }
    }

    fn begin_render_pass<'a>(
        &'a self,
        encoder: &'a mut CommandEncoder,
        target: &'a TextureView,
    ) -> RenderPass<'a> {
        encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("ori_render_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &self.msaa,
                resolve_target: Some(target),
                ops: Operations {
                    load: LoadOp::Clear(wgpu::Color::WHITE),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        })
    }

    pub fn render_scene(&mut self, scene: &mut Scene, width: u32, height: u32) {
        self.resize(width, height);

        let fragments = scene.fragments_mut();
        fragments.sort_by(|a, b| a.depth.partial_cmp(&b.depth).unwrap());

        self.prepare_fragments(fragments);

        let mut encoder = (self.device).create_command_encoder(&CommandEncoderDescriptor {
            label: Some("ori_command_encoder"),
        });

        let target = match self.surface.get_current_texture() {
            Ok(target) => target,
            Err(SurfaceError::OutOfMemory) => panic!("Out of memory"),
            Err(_) => return,
        };

        let target_view = target.texture.create_view(&Default::default());

        {
            let mut pass = self.begin_render_pass(&mut encoder, &target_view);
            self.render_fragments(&mut pass, fragments);
        }

        self.queue.submit(Some(encoder.finish()));

        target.present();
    }
}
