use std::sync::Arc;

use ori_core::{
    canvas::{Color, Fragment, Mesh, Primitive, Quad, Scene},
    layout::{Affine, Rect, Size},
};
use wgpu::{
    CommandEncoder, CommandEncoderDescriptor, CompositeAlphaMode, Device, LoadOp, Operations,
    PresentMode, Queue, RenderPass, RenderPassColorAttachment, RenderPassDescriptor, Surface,
    SurfaceConfiguration, SurfaceError, TextureDimension, TextureFormat, TextureUsages,
    TextureView,
};

use crate::{log::warn_internal, RenderError};

use super::{MeshRender, QuadRender, TextureCache, WgpuRenderInstance};

#[derive(Clone, Debug)]
enum Batch {
    Quad(usize),
    Mesh(usize),
}

#[derive(Debug)]
pub struct WgpuRender {
    device: Arc<Device>,
    queue: Arc<Queue>,
    surface: Surface,
    config: SurfaceConfiguration,
    samples: u32,
    image: TextureCache,
    msaa: Option<TextureView>,
    quad: QuadRender,
    mesh: MeshRender,
}

impl WgpuRender {
    pub fn new(
        instance: &WgpuRenderInstance,
        surface: Surface,
        samples: u32,
        width: u32,
        height: u32,
    ) -> Result<Self, RenderError> {
        let device = instance.device.clone();
        let queue = instance.queue.clone();

        let config = surface.get_default_config(&instance.adapter, width, height);
        let mut config = config.ok_or(RenderError::SurfaceIncompatible)?;

        match config.format {
            TextureFormat::Bgra8UnormSrgb => {
                config.format = TextureFormat::Bgra8Unorm;
            }
            TextureFormat::Rgba8UnormSrgb => {
                config.format = TextureFormat::Rgba8Unorm;
            }
            TextureFormat::Bgra8Unorm | TextureFormat::Rgba8Unorm => {}
            _ => {
                return Err(RenderError::SurfaceIncompatible);
            }
        }

        config.alpha_mode = CompositeAlphaMode::Auto;
        config.present_mode = PresentMode::AutoVsync;
        surface.configure(&device, &config);

        let cache = TextureCache::new(&device);

        let msaa = if samples > 1 {
            Some(Self::create_msaa(
                &device,
                config.format,
                samples,
                width,
                height,
            ))
        } else {
            None
        };

        let quad = QuadRender::new(&device, config.format, samples, cache.bind_group_layout());
        let mesh = MeshRender::new(&device, config.format, samples, cache.bind_group_layout());

        Ok(Self {
            device,
            queue,
            surface,
            config,
            samples,
            image: cache,
            msaa,
            quad,
            mesh,
        })
    }

    fn create_msaa(
        device: &Device,
        format: TextureFormat,
        sample_count: u32,
        width: u32,
        height: u32,
    ) -> TextureView {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ori_msaa_texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count,
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
        if let Some(ref mut msaa) = self.msaa {
            *msaa = Self::create_msaa(
                &self.device,
                self.config.format,
                self.samples,
                width,
                height,
            );
        }
    }

    fn push_quad_batch(
        &mut self,
        batches: &mut Vec<Batch>,
        quad_clip: Option<Rect>,
        quad_batch: &mut Vec<(&Quad, Affine)>,
        quad_batch_count: &mut usize,
    ) {
        let resolution = self.size();

        self.quad.prepare_batch(
            &self.device,
            &self.queue,
            &mut self.image,
            *quad_batch_count,
            quad_batch,
            quad_clip.unwrap(),
            resolution,
        );

        batches.push(Batch::Quad(*quad_batch_count));

        *quad_batch = Vec::new();
        *quad_batch_count += quad_batch.len();
        *quad_batch_count += 1;
    }

    fn push_mesh_batch(
        &mut self,
        batches: &mut Vec<Batch>,
        mesh_clip: Option<Rect>,
        mesh_batch: &mut Vec<(&Mesh, Affine)>,
        mesh_batch_count: &mut usize,
    ) {
        let resolution = self.size();
        self.mesh.prepare_batch(
            &self.device,
            &self.queue,
            &mut self.image,
            *mesh_batch_count,
            mesh_batch,
            mesh_clip.unwrap(),
            resolution,
        );

        batches.push(Batch::Mesh(*mesh_batch_count));

        *mesh_batch = Vec::new();
        *mesh_batch_count += mesh_batch.len();
        *mesh_batch_count += 1;
    }

    fn prepare_fragments(&mut self, fragments: &[Fragment]) -> Vec<Batch> {
        let mut batches = Vec::new();

        let mut quad_image = None;
        let mut quad_clip = None;
        let mut quad_batch = Vec::new();
        let mut quad_batch_count = 0;

        let mut mesh_image = None;
        let mut mesh_clip = None;
        let mut mesh_batch = Vec::new();
        let mut mesh_batch_count = 0;

        for fragment in fragments {
            match fragment.primitive {
                Primitive::Trigger(_) => {}
                Primitive::Quad(ref quad) => {
                    if !mesh_batch.is_empty() {
                        self.push_mesh_batch(
                            &mut batches,
                            mesh_clip,
                            &mut mesh_batch,
                            &mut mesh_batch_count,
                        );
                    }

                    let image = quad.background.texture.clone();
                    let new_batch = quad_clip != Some(fragment.clip) || quad_image != image;
                    if new_batch && !quad_batch.is_empty() {
                        self.push_quad_batch(
                            &mut batches,
                            quad_clip,
                            &mut quad_batch,
                            &mut quad_batch_count,
                        );
                    }

                    quad_image = image;
                    quad_clip = Some(fragment.clip);
                    quad_batch.push((quad, fragment.transform));
                }
                Primitive::Mesh(ref mesh) => {
                    if !quad_batch.is_empty() {
                        self.push_quad_batch(
                            &mut batches,
                            quad_clip,
                            &mut quad_batch,
                            &mut quad_batch_count,
                        );
                    }

                    let new_batch = mesh_clip != Some(fragment.clip) || mesh_image != mesh.texture;
                    if new_batch && !mesh_batch.is_empty() {
                        self.push_mesh_batch(
                            &mut batches,
                            mesh_clip,
                            &mut mesh_batch,
                            &mut mesh_batch_count,
                        );
                    }

                    mesh_image = mesh.texture.clone();
                    mesh_clip = Some(fragment.clip);
                    mesh_batch.push((mesh, fragment.transform));
                }
            }
        }

        if !quad_batch.is_empty() {
            self.push_quad_batch(
                &mut batches,
                quad_clip,
                &mut quad_batch,
                &mut quad_batch_count,
            );
        }

        if !mesh_batch.is_empty() {
            self.push_mesh_batch(
                &mut batches,
                mesh_clip,
                &mut mesh_batch,
                &mut mesh_batch_count,
            );
        }

        batches
    }

    fn render_fragments<'a>(&'a self, pass: &mut RenderPass<'a>, batches: &[Batch]) {
        for batch in batches {
            match batch {
                Batch::Quad(index) => {
                    self.quad.render(pass, *index);
                }
                Batch::Mesh(index) => {
                    self.mesh.render(pass, *index);
                }
            }
        }
    }

    fn begin_render_pass<'a>(
        &'a self,
        encoder: &'a mut CommandEncoder,
        target: &'a TextureView,
        clear_color: Color,
    ) -> RenderPass<'a> {
        match self.msaa {
            Some(ref msaa) => encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("ori_render_pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: msaa,
                    resolve_target: Some(target),
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
            }),
            None => encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("ori_render_pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: target,
                    resolve_target: None,
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
            }),
        }
    }

    pub fn render_scene(&mut self, scene: &mut Scene, clear_color: Color, width: u32, height: u32) {
        self.resize(width, height);

        let fragments = scene.fragments_mut();
        fragments.sort_by(|a, b| a.depth.partial_cmp(&b.depth).unwrap());

        let batches = self.prepare_fragments(fragments);

        let mut encoder = (self.device).create_command_encoder(&CommandEncoderDescriptor {
            label: Some("ori_command_encoder"),
        });

        let target = match self.surface.get_current_texture() {
            Ok(target) => target,
            Err(SurfaceError::OutOfMemory) => panic!("Out of memory"),
            Err(err) => {
                warn_internal!("Surface error: {:?}", err);
                return;
            }
        };

        let target_view = target.texture.create_view(&Default::default());

        let mut pass = self.begin_render_pass(&mut encoder, &target_view, clear_color);
        self.render_fragments(&mut pass, &batches);
        drop(pass);

        self.queue.submit(Some(encoder.finish()));

        target.present();
    }
}
