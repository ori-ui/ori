use ash::vk;
use ori_graphics::{
    Color, Frame, Glyph, ImageData, ImageHandle, Line, Renderer, TextSection, Vec2,
};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

use crate::{AshDevice, AshError, AshInstance, AshSurface, AshSwapchain, PresentMode};

pub struct AshRenderer {
    instance: AshInstance,
    device: AshDevice,
    surface: AshSurface,
    swapchain: AshSwapchain,
    render_pass: vk::RenderPass,
    framebuffers: Vec<vk::Framebuffer>,
    command_pool: vk::CommandPool,
    prepare_buffer: vk::CommandBuffer,
    draw_buffer: vk::CommandBuffer,
    prepare_reuse_fence: vk::Fence,
    draw_reuse_fence: vk::Fence,
    prepare_semaphore: vk::Semaphore,
    render_semaphore: vk::Semaphore,
    present_semaphore: vk::Semaphore,
    window_size: Vec2,
}

impl AshRenderer {
    unsafe fn create_render_pass(
        device: &AshDevice,
        swapchain: &AshSwapchain,
    ) -> Result<vk::RenderPass, AshError> {
        let attachments = &[vk::AttachmentDescription {
            format: swapchain.surface_format().format,
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::STORE,
            final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
            ..Default::default()
        }];

        let color_attachment_refs = &[vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        }];

        let subpass = vk::SubpassDescription::builder()
            .color_attachments(color_attachment_refs)
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .build();

        let dependency = vk::SubpassDependency {
            src_subpass: vk::SUBPASS_EXTERNAL,
            src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_READ
                | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            ..Default::default()
        };

        let create_info = vk::RenderPassCreateInfo::builder()
            .attachments(attachments)
            .subpasses(&[subpass])
            .dependencies(&[dependency])
            .build();

        Ok(device.create_render_pass(&create_info, None)?)
    }

    pub unsafe fn new(
        window: &(impl HasRawDisplayHandle + HasRawWindowHandle),
        width: u32,
        height: u32,
    ) -> Result<Self, AshError> {
        let instance = AshInstance::new(&window)?;

        let display_handle = window.raw_display_handle();
        let window_handle = window.raw_window_handle();
        let surface = AshSurface::new(&instance, display_handle, window_handle)?;

        let device = AshDevice::create(&instance, &surface, AshDevice::required_extensions())?;

        let present_mode = PresentMode::Immediate;
        let swapchain = AshSwapchain::new(&surface, &device, present_mode, width, height)?;

        let render_pass = Self::create_render_pass(&device, &swapchain)?;

        let mut framebuffers = Vec::new();
        for image_view in swapchain.present_image_views() {
            let attachments = &[image_view];

            let create_info = vk::FramebufferCreateInfo::builder()
                .render_pass(render_pass)
                .attachments(attachments)
                .width(width)
                .height(height)
                .layers(1);

            framebuffers.push(device.create_framebuffer(&create_info, None)?);
        }

        let create_info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(device.queue_family_index());
        let command_pool = device.create_command_pool(&create_info, None)?;

        let alloc_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(command_pool)
            .command_buffer_count(2)
            .level(vk::CommandBufferLevel::PRIMARY);

        let buffers = device.allocate_command_buffers(&alloc_info)?;
        let prepare_buffer = buffers[0];
        let draw_buffer = buffers[1];

        let create_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);
        let prepare_reuse_fence = device.create_fence(&create_info, None)?;
        let draw_reuse_fence = device.create_fence(&create_info, None)?;

        let create_info = vk::SemaphoreCreateInfo::builder();
        let prepare_semaphore = device.create_semaphore(&create_info, None)?;
        let render_semaphore = device.create_semaphore(&create_info, None)?;
        let present_semaphore = device.create_semaphore(&create_info, None)?;

        Ok(Self {
            instance,
            device,
            surface,
            swapchain,

            render_pass,
            framebuffers,

            command_pool,

            prepare_buffer,
            draw_buffer,

            prepare_reuse_fence,
            draw_reuse_fence,

            prepare_semaphore,
            render_semaphore,
            present_semaphore,

            window_size: Vec2::new(width as f32, height as f32),
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.window_size = Vec2::new(width as f32, height as f32);
        let present_mode = PresentMode::Immediate;
        let result = unsafe { self.swapchain.recreate(present_mode, width, height) };

        if let Err(err) = result {
            tracing::error!("Failed to recreate swapchain: {}", err);
            return;
        }

        // recreate framebuffers
        for framebuffer in self.framebuffers.drain(..) {
            unsafe { self.device.destroy_framebuffer(framebuffer, None) };
        }

        for image_view in self.swapchain.present_image_views() {
            let attachments = &[image_view];

            let create_info = vk::FramebufferCreateInfo::builder()
                .render_pass(self.render_pass)
                .attachments(attachments)
                .width(width)
                .height(height)
                .layers(1);

            let frame_buffer = unsafe { self.device.create_framebuffer(&create_info, None) };

            match frame_buffer {
                Ok(frame_buffer) => self.framebuffers.push(frame_buffer),
                Err(err) => tracing::error!("Failed to create framebuffer: {}", err),
            }
        }
    }

    pub fn instance(&self) -> &AshInstance {
        &self.instance
    }

    pub fn device(&self) -> &AshDevice {
        &self.device
    }

    unsafe fn prepare_frame(&mut self, frame: &Frame) -> Result<(), AshError> {
        (self.device).wait_for_fences(&[self.prepare_reuse_fence], true, u64::MAX)?;
        (self.device).reset_fences(&[self.prepare_reuse_fence])?;

        self.device.reset_command_buffer(
            self.prepare_buffer,
            vk::CommandBufferResetFlags::RELEASE_RESOURCES,
        )?;

        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        (self.device).begin_command_buffer(self.prepare_buffer, &begin_info)?;

        self.device.end_command_buffer(self.prepare_buffer)?;

        let submit_info = vk::SubmitInfo::builder()
            .command_buffers(&[self.prepare_buffer])
            .build();

        (self.device).queue_submit(
            self.device.queue(),
            &[submit_info],
            self.prepare_reuse_fence,
        )?;

        Ok(())
    }

    pub unsafe fn render_frame(
        &mut self,
        frame: &Frame,
        clear_color: Color,
    ) -> Result<(), AshError> {
        self.prepare_frame(frame)?;

        (self.device).wait_for_fences(&[self.draw_reuse_fence], true, u64::MAX)?;
        (self.device).reset_fences(&[self.draw_reuse_fence])?;

        self.device.reset_command_buffer(
            self.draw_buffer,
            vk::CommandBufferResetFlags::RELEASE_RESOURCES,
        )?;

        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        (self.device).begin_command_buffer(self.draw_buffer, &begin_info)?;

        let (image_index, _) = self.swapchain.acquire_next_image(
            u64::MAX,
            self.present_semaphore,
            vk::Fence::null(),
        )?;

        let clear_values = [vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [
                    clear_color.r as f32,
                    clear_color.g as f32,
                    clear_color.b as f32,
                    clear_color.a as f32,
                ],
            },
        }];

        let begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.render_pass)
            .framebuffer(self.framebuffers[image_index as usize])
            .clear_values(&clear_values)
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: vk::Extent2D {
                    width: self.window_size.x as u32,
                    height: self.window_size.y as u32,
                },
            });

        (self.device).cmd_begin_render_pass(
            self.draw_buffer,
            &begin_info,
            vk::SubpassContents::INLINE,
        );
        (self.device).cmd_end_render_pass(self.draw_buffer);

        (self.device).end_command_buffer(self.draw_buffer)?;

        let wait_semaphores = [self.present_semaphore];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = [self.draw_buffer];
        let signal_semaphores = [self.render_semaphore];

        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores)
            .build();

        (self.device).queue_submit(self.device.queue(), &[submit_info], self.draw_reuse_fence)?;

        self.present_frame(image_index)?;

        Ok(())
    }

    unsafe fn present_frame(&mut self, image_index: u32) -> Result<(), AshError> {
        let wait_semaphores = [self.render_semaphore];
        let swapchains = [self.swapchain.swapchain()];
        let image_indices = [image_index];

        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&wait_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        (self.swapchain.loader()).queue_present(self.device.queue(), &present_info)?;

        Ok(())
    }
}

impl Drop for AshRenderer {
    fn drop(&mut self) {
        unsafe {
            self.device.device_wait_idle().unwrap();

            for framebuffer in self.framebuffers.drain(..) {
                self.device.destroy_framebuffer(framebuffer, None);
            }

            self.device.destroy_fence(self.prepare_reuse_fence, None);
            self.device.destroy_fence(self.draw_reuse_fence, None);

            self.device.destroy_render_pass(self.render_pass, None);

            let buffers = [self.prepare_buffer, self.draw_buffer];
            (self.device).free_command_buffers(self.command_pool, &buffers);

            self.device.destroy_semaphore(self.prepare_semaphore, None);
            self.device.destroy_semaphore(self.render_semaphore, None);
            self.device.destroy_semaphore(self.present_semaphore, None);

            self.device.destroy_command_pool(self.command_pool, None);
        }
    }
}

impl Renderer for AshRenderer {
    fn window_size(&self) -> Vec2 {
        self.window_size
    }

    fn create_image(&self, data: &ImageData) -> ImageHandle {
        println!(
            "Creating image with size: {}x{}",
            data.width(),
            data.height()
        );

        ImageHandle::new((), data.width(), data.height())
    }

    fn text_glyphs(&self, section: &TextSection) -> Vec<Glyph> {
        Vec::new()
    }

    fn text_lines(&self, section: &TextSection) -> Vec<Line> {
        Vec::new()
    }
}
