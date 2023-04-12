use std::cell::RefCell;

use ily_graphics::{Frame, Mesh, Primitive, Quad, TextLayout, TextSection};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use wgpu::{
    util::StagingBelt, Device, Extent3d, Instance, LoadOp, Operations, Queue, RenderPass,
    RenderPassColorAttachment, RenderPassDescriptor, RequestAdapterOptions, Surface,
    SurfaceConfiguration, Texture, TextureDescriptor, TextureDimension, TextureFormat,
    TextureUsages,
};
use wgpu_glyph::{ab_glyph::FontArc, GlyphBrush, GlyphBrushBuilder};

use crate::{Fonts, MeshPipeline, QuadPipeline};

pub struct Renderer {
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    surface: Surface,
    msaa_texture: Texture,
    mesh_pipeline: MeshPipeline,
    quad_pipeline: QuadPipeline,
    fonts: Fonts,
    glyph_brush: RefCell<GlyphBrush<()>>,
    staging_belt: StagingBelt,
}

impl Renderer {
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
        surface.configure(&device, &config);

        let msaa_texture = Self::create_msaa_texture(&device, config.format, width, height);

        let mesh_pipeline = MeshPipeline::new(&device, config.format);
        let quad_pipeline = QuadPipeline::new(&device, config.format);

        let fonts = Fonts::default();
        let font = FontArc::try_from_slice(include_bytes!("../../../assets/NotoSans-Medium.ttf"));
        let glyph_brush =
            GlyphBrushBuilder::using_font(font.unwrap()).build(&device, config.format);

        let staging_belt = StagingBelt::new(1024);

        Self {
            device,
            queue,
            config,
            surface,
            msaa_texture,
            mesh_pipeline,
            quad_pipeline,
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

    pub fn text_layout(&mut self) -> impl TextLayout + '_ {
        crate::text::TextLayout {
            fonts: &self.fonts,
            glyph: &self.glyph_brush,
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

    pub fn prepare_mesh(&mut self, mesh: &Mesh, mesh_index: &mut usize) {
        (self.mesh_pipeline).prepare_mesh(&self.device, &self.queue, mesh, *mesh_index);
        *mesh_index += 1;
    }

    pub fn prepare_quad(&mut self, quad: &Quad, quad_index: &mut usize) {
        (self.quad_pipeline).prepare_quad(&self.device, &self.queue, quad, *quad_index);
        *quad_index += 1;
    }

    pub fn prepare_primitive(
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

    pub fn prepare_frame(&mut self, frame: &Frame) {
        let mut mesh_index = 0;
        let mut quad_index = 0;
        frame.visit_primitives(|primitive| {
            self.prepare_primitive(primitive, &mut mesh_index, &mut quad_index);
        });
    }

    pub fn render_quad<'a>(&'a self, pass: &mut RenderPass<'a>, index: &mut usize) {
        self.quad_pipeline.render(pass, *index);
        *index += 1;
    }

    pub fn render_mesh<'a>(&'a self, pass: &mut RenderPass<'a>, index: &mut usize) {
        self.mesh_pipeline.render(pass, *index);
        *index += 1;
    }

    pub fn render_text<'a>(&self, text: &TextSection) {
        let section = self.fonts.convert_section(text);
        self.glyph_brush.borrow_mut().queue(section);
    }

    pub fn render_primitive<'a>(
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

    pub fn render_frame(&mut self, frame: &Frame) {
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
                    load: LoadOp::Clear(wgpu::Color::WHITE),
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
