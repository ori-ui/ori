use ori_core::{
    canvas::{Color, Scene},
    layout::Size,
};
use wgpu::{
    CommandEncoder, CommandEncoderDescriptor, CompositeAlphaMode, Device, LoadOp, Operations,
    PresentMode, RenderPass, RenderPassColorAttachment, RenderPassDescriptor, StoreOp, Surface,
    SurfaceConfiguration, SurfaceError, TextureDimension, TextureFormat, TextureUsages,
    TextureView,
};

use crate::{MeshRender, TextureCache, WgpuContext, WgpuError, WgpuRenderInstance};

/// A renderer for a [`ori_core::canvas::Scene`].
#[derive(Debug)]
pub struct WgpuRender {
    surface: Surface,
    config: SurfaceConfiguration,
    samples: u32,
    image: TextureCache,
    msaa: Option<TextureView>,
    mesh: MeshRender,
}

impl WgpuRender {
    /// Create a new renderer.
    pub fn new(
        instance: &WgpuRenderInstance,
        surface: Surface,
        samples: u32,
        width: u32,
        height: u32,
    ) -> Result<Self, WgpuError> {
        let device = &instance.device;

        let config = surface.get_default_config(&instance.adapter, width, height);
        let mut config = config.ok_or(WgpuError::SurfaceIncompatible)?;

        match config.format {
            TextureFormat::Bgra8UnormSrgb => {
                config.format = TextureFormat::Bgra8Unorm;
            }
            TextureFormat::Rgba8UnormSrgb => {
                config.format = TextureFormat::Rgba8Unorm;
            }
            TextureFormat::Bgra8Unorm | TextureFormat::Rgba8Unorm => {}
            _ => {
                return Err(WgpuError::SurfaceIncompatible);
            }
        }

        config.alpha_mode = CompositeAlphaMode::Auto;
        config.present_mode = PresentMode::AutoVsync;
        surface.configure(device, &config);

        let cache = TextureCache::new(device);

        let msaa = if samples > 1 {
            Some(Self::create_msaa(
                device,
                config.format,
                samples,
                width,
                height,
            ))
        } else {
            None
        };

        let mesh = MeshRender::new(device, config.format, samples, cache.bind_group_layout());

        Ok(Self {
            surface,
            config,
            samples,
            image: cache,
            msaa,
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

    /// Clean up unused resources.
    pub fn clean(&mut self) {
        self.image.clean();
    }

    fn size(&self) -> Size {
        Size::new(self.config.width as f32, self.config.height as f32)
    }

    fn resize(&mut self, context: &WgpuContext, width: u32, height: u32) {
        if self.config.width == width && self.config.height == height {
            return;
        }

        self.config.width = width;
        self.config.height = height;

        self.surface.configure(&context.device, &self.config);
        if let Some(ref mut msaa) = self.msaa {
            *msaa = Self::create_msaa(
                &context.device,
                self.config.format,
                self.samples,
                width,
                height,
            );
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
                        store: StoreOp::Store,
                    },
                })],
                ..Default::default()
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
                        store: StoreOp::Store,
                    },
                })],
                ..Default::default()
            }),
        }
    }

    /// Render the given [`ori_core::canvas::Scene`].
    pub fn render_scene(
        &mut self,
        context: &WgpuContext,
        scene: &Scene,
        clear_color: Color,
        width: u32,
        height: u32,
    ) {
        self.resize(context, width, height);
        let batches = scene.batches();

        let size = self.size();
        for batch in batches.iter() {
            (self.mesh).prepare_batch(context, &mut self.image, batch, size);
        }

        let mut encoder = (context.device).create_command_encoder(&CommandEncoderDescriptor {
            label: Some("ori_command_encoder"),
        });

        let target = match self.surface.get_current_texture() {
            Ok(target) => target,
            Err(SurfaceError::OutOfMemory) => panic!("Out of memory"),
            Err(err) => {
                eprintln!("Surface error: {:?}", err);
                return;
            }
        };

        let target_view = target.texture.create_view(&Default::default());

        let mut pass = self.begin_render_pass(&mut encoder, &target_view, clear_color);

        for batch in batches.iter() {
            self.mesh.render_batch(&mut pass, batch.index);
        }

        drop(pass);

        context.queue.submit(Some(encoder.finish()));

        target.present();
    }
}
