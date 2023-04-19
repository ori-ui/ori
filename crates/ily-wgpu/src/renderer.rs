use std::cell::RefCell;

use ily_core::Vec2;
use ily_graphics::{
    Color, Frame, ImageData, ImageHandle, Mesh, Primitive, Quad, Rect, Renderer, TextHit,
    TextSection,
};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use wgpu::{
    util::{DeviceExt, StagingBelt},
    BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, CompositeAlphaMode, Device, Extent3d,
    FilterMode, Instance, LoadOp, Operations, Queue, RenderPass, RenderPassColorAttachment,
    RenderPassDescriptor, RequestAdapterOptions, SamplerBindingType, SamplerDescriptor,
    ShaderStages, Surface, SurfaceConfiguration, Texture, TextureDescriptor, TextureDimension,
    TextureFormat, TextureSampleType, TextureUsages, TextureViewDimension,
};
use wgpu_glyph::{
    ab_glyph::{Font, FontArc, ScaleFont},
    GlyphBrush, GlyphBrushBuilder, GlyphCruncher,
};

use crate::{Fonts, MeshPipeline, QuadPipeline, WgpuImage};

const TEXT_FONT: &[u8] = include_bytes!("../../../assets/NotoSans-Medium.ttf");
const ICON_FONT: &[u8] = include_bytes!("../../../assets/MaterialIcons-Regular.ttf");

pub struct WgpuRenderer {
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    surface: Surface,
    msaa_texture: Texture,
    image_bind_group_layout: BindGroupLayout,
    default_image: WgpuImage,
    mesh_pipeline: MeshPipeline,
    quad_pipeline: QuadPipeline,
    fonts: Fonts,
    glyph_brush: RefCell<GlyphBrush<()>>,
    staging_belt: StagingBelt,
}

impl WgpuRenderer {
    pub async unsafe fn new_async(
        window: &(impl HasRawWindowHandle + HasRawDisplayHandle),
        width: u32,
        height: u32,
    ) -> Self {
        let instance = Instance::new(Default::default());
        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: Default::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&Default::default(), None)
            .await
            .unwrap();

        let mut config = surface.get_default_config(&adapter, width, height).unwrap();
        config.format = TextureFormat::Bgra8Unorm;
        config.alpha_mode = CompositeAlphaMode::Auto;
        surface.configure(&device, &config);

        let msaa_texture = Self::create_msaa_texture(&device, config.format, width, height);

        let image_bind_group_layout = Self::create_image_bind_group_layout(&device);
        let default_image = Self::create_default_image(&device, &queue, &image_bind_group_layout);

        let mesh_pipeline = MeshPipeline::new(&device, &image_bind_group_layout, config.format);
        let quad_pipeline = QuadPipeline::new(&device, config.format);

        let mut fonts = Fonts::default();
        let text_font = FontArc::try_from_slice(TEXT_FONT).unwrap();
        let icon_font = FontArc::try_from_slice(ICON_FONT).unwrap();

        fonts.add_font("NotoSans");
        fonts.add_font("icon");

        let mut glyph_brush_builder = GlyphBrushBuilder::using_font(text_font);
        glyph_brush_builder.add_font(icon_font);

        let glyph_brush = glyph_brush_builder.build(&device, config.format);

        let staging_belt = StagingBelt::new(1024);

        Self {
            device,
            queue,
            config,
            surface,
            msaa_texture,
            mesh_pipeline,
            quad_pipeline,
            image_bind_group_layout,
            default_image,
            fonts,
            glyph_brush: RefCell::new(glyph_brush),
            staging_belt,
        }
    }

    pub unsafe fn new(
        window: &(impl HasRawWindowHandle + HasRawDisplayHandle),
        width: u32,
        height: u32,
    ) -> Self {
        pollster::block_on(Self::new_async(window, width, height))
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

    pub fn resize(&mut self, width: u32, height: u32) {
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

    fn prepare_mesh(&mut self, mesh: &Mesh, mesh_index: &mut usize) {
        (self.mesh_pipeline).prepare_mesh(&self.device, &self.queue, mesh, *mesh_index);
        *mesh_index += 1;
    }

    fn prepare_quad(&mut self, quad: &Quad, quad_index: &mut usize) {
        (self.quad_pipeline).prepare_quad(&self.device, &self.queue, quad, *quad_index);
        *quad_index += 1;
    }

    fn prepare_primitive(
        &mut self,
        primitive: &Primitive,
        mesh_index: &mut usize,
        quad_index: &mut usize,
    ) {
        match primitive {
            Primitive::Quad(quad) => self.prepare_quad(quad, quad_index),
            Primitive::Mesh(mesh) => self.prepare_mesh(mesh, mesh_index),
            _ => {}
        }
    }

    fn prepare_frame(&mut self, frame: &Frame) {
        let mut mesh_index = 0;
        let mut quad_index = 0;
        frame.visit_primitives(|primitive| {
            self.prepare_primitive(primitive, &mut mesh_index, &mut quad_index);
        });
    }

    fn render_quad<'a>(&'a self, pass: &mut RenderPass<'a>, index: &mut usize) {
        self.quad_pipeline.render(pass, *index);
        *index += 1;
    }

    fn render_mesh<'a>(&'a self, pass: &mut RenderPass<'a>, index: &mut usize) {
        self.mesh_pipeline.render(pass, &self.default_image, *index);
        *index += 1;
    }

    fn render_text<'a>(&self, text: &TextSection) {
        let section = self.fonts.convert_section(text);
        self.glyph_brush.borrow_mut().queue(section);
    }

    fn render_primitive<'a>(
        &'a self,
        pass: &mut RenderPass<'a>,
        primitive: &Primitive,
        mesh_index: &mut usize,
        quad_index: &mut usize,
    ) {
        match primitive {
            Primitive::Text(text) => self.render_text(text),
            Primitive::Quad(_) => self.render_quad(pass, quad_index),
            Primitive::Mesh(_) => self.render_mesh(pass, mesh_index),
            _ => {}
        }
    }

    pub fn render_frame(&mut self, frame: &Frame, clear_color: Color) {
        let width = self.config.width;
        let height = self.config.height;
        self.quad_pipeline.set_size(&self.queue, width, height);
        self.mesh_pipeline.set_size(&self.queue, width, height);

        // prepare the pipelines
        self.prepare_frame(frame);

        let target = self.surface.get_current_texture().unwrap();
        let view = target.texture.create_view(&Default::default());
        let msaa_view = self.msaa_texture.create_view(&Default::default());

        let mut encoder = self.device.create_command_encoder(&Default::default());

        // create render pass
        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Ily Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &msaa_view,
                resolve_target: Some(&view),
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

        // render primitives
        let mut mesh_index = 0;
        let mut quad_index = 0;
        frame.visit_primitives(|primitive| {
            self.render_primitive(&mut pass, primitive, &mut mesh_index, &mut quad_index);
        });

        drop(pass);

        self.glyph_brush
            .borrow_mut()
            .draw_queued(
                &self.device,
                &mut self.staging_belt,
                &mut encoder,
                &view,
                width,
                height,
            )
            .unwrap();

        // submit and present
        self.staging_belt.finish();
        self.queue.submit(Some(encoder.finish()));
        target.present();
        self.staging_belt.recall();
    }
}

impl Renderer for WgpuRenderer {
    fn scale(&self) -> f32 {
        1.0
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

    fn messure_text(&self, section: &TextSection) -> Option<Rect> {
        let section = self.fonts.convert_section(section);
        let mut glyph_brush = self.glyph_brush.borrow_mut();
        let bounds = glyph_brush.glyph_bounds(&section)?;

        Some(Rect {
            min: Vec2::new(bounds.min.x, bounds.min.y),
            max: Vec2::new(bounds.max.x, bounds.max.y),
        })
    }

    fn hit_text(&self, section: &TextSection, position: Vec2) -> Option<TextHit> {
        let mut glyph_brush = self.glyph_brush.borrow_mut();
        let font_id = if let Some(font) = &section.font {
            self.fonts.find_font(font)
        } else {
            wgpu_glyph::FontId::default()
        };

        let font = glyph_brush.fonts()[font_id.0].clone();
        let scaled = font.into_scaled(section.scale);
        let section = self.fonts.convert_section(section);

        let mut closest = None::<TextHit>;

        for glyph in glyph_brush.glyphs(section) {
            let wgpu_glyph::SectionGlyph {
                ref glyph,
                byte_index,
                ..
            } = *glyph;

            let min = Vec2::new(
                glyph.position.x - scaled.h_side_bearing(glyph.id),
                glyph.position.y - scaled.ascent(),
            );
            let size = Vec2::new(
                scaled.h_advance(glyph.id),
                scaled.ascent() - scaled.descent(),
            );

            let rect = Rect::min_size(min, size);
            let delta = position - rect.center();

            if rect.contains(position) {
                return Some(TextHit {
                    inside: true,
                    index: byte_index,
                    delta,
                });
            } else {
                if let Some(ref mut closest) = closest {
                    if delta.length_squared() < closest.delta.length_squared() {
                        *closest = TextHit {
                            inside: false,
                            index: byte_index,
                            delta,
                        };
                    }
                } else {
                    closest = Some(TextHit {
                        inside: false,
                        index: byte_index,
                        delta,
                    });
                }
            }
        }

        closest
    }
}
