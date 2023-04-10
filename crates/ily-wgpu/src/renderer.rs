use ily_graphics::{Frame, Mesh, Primitive};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use wgpu::{
    Device, Instance, Queue, RenderPass, RenderPassColorAttachment, RenderPassDescriptor,
    RequestAdapterOptions, Surface, SurfaceConfiguration,
};

use crate::MeshPipeline;

pub struct Renderer {
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    surface: Surface,
    mesh_pipeline: MeshPipeline,
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

        let config = surface.get_default_config(&adapter, width, height).unwrap();
        surface.configure(&device, &config);

        let mesh_pipeline = MeshPipeline::new(&device);

        Self {
            device,
            queue,
            config,
            surface,
            mesh_pipeline,
        }
    }

    pub unsafe fn new(
        window: &(impl HasRawWindowHandle + HasRawDisplayHandle),
        width: u32,
        height: u32,
    ) -> Self {
        pollster::block_on(Self::new_async(window, width, height))
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn prepare_mesh(&mut self, mesh: &Mesh, mesh_index: &mut usize) {
        (self.mesh_pipeline).prepare_mesh(&self.device, &self.queue, mesh, *mesh_index);
        *mesh_index += 1;
    }

    pub fn prepare_primitive(&mut self, primitive: &Primitive, mesh_index: &mut usize) {
        match primitive {
            Primitive::Quad(quad) => self.prepare_mesh(&quad.mesh(), mesh_index),
            Primitive::Mesh(mesh) => self.prepare_mesh(mesh, mesh_index),
            _ => {}
        }
    }

    pub fn prepare_frame(&mut self, frame: &Frame) {
        let mut mesh_index = 0;
        frame.visit_primitives(|primitive| {
            self.prepare_primitive(primitive, &mut mesh_index);
        });
    }

    pub fn render_mesh<'a>(&'a self, pass: &mut RenderPass<'a>, index: &mut usize) {
        self.mesh_pipeline.render(pass, *index);
        *index += 1;
    }

    pub fn render_primitive<'a>(
        &'a self,
        pass: &mut RenderPass<'a>,
        primitive: &Primitive,
        mesh_index: &mut usize,
    ) {
        match primitive {
            Primitive::Quad(_) => self.render_mesh(pass, mesh_index),
            Primitive::Mesh(_) => self.render_mesh(pass, mesh_index),
            _ => {}
        }
    }

    pub fn render_frame(&mut self, frame: &Frame) {
        let width = self.config.width;
        let height = self.config.height;
        self.mesh_pipeline.set_size(&self.queue, width, height);

        // prepare the pipelines
        self.prepare_frame(frame);

        let target = self.surface.get_current_texture().unwrap();
        let view = target.texture.create_view(&Default::default());

        let mut encoder = self.device.create_command_encoder(&Default::default());

        // create render pass
        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Ily Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: Default::default(),
            })],
            depth_stencil_attachment: None,
        });

        // render primitives
        let mut mesh_index = 0;
        frame.visit_primitives(|primitive| {
            self.render_primitive(&mut pass, primitive, &mut mesh_index);
        });

        drop(pass);

        // submit and present
        self.queue.submit(Some(encoder.finish()));
        target.present();
    }
}
