use ash::vk::{self, Handle};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::{event_loop::EventLoop, window::Window};

pub(crate) struct VulkanContext {
    entry:        ash::Entry,
    instance:     ash::Instance,
    physical:     vk::PhysicalDevice,
    device:       ash::Device,
    family_index: u32,
    queue:        vk::Queue,
}

impl Drop for VulkanContext {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
    }
}

impl VulkanContext {
    pub(crate) unsafe fn new<T>(event_loop: &EventLoop<T>) -> Self {
        let entry = unsafe { ash::Entry::load().unwrap() };

        let display_handle = event_loop.display_handle().unwrap();
        let raw_display = display_handle.as_raw();

        let app_info = vk::ApplicationInfo::default()
            .engine_name(c"skia")
            .api_version(vk::make_api_version(0, 1, 4, 0));

        let extensions = ash_window::enumerate_required_extensions(raw_display).unwrap();
        let layers = [c"VK_LAYER_KHRONOS_validation".as_ptr()];

        let mut instance_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(extensions);

        if cfg!(debug_assertions) {
            let is_validation_supported = unsafe {
                entry
                    .enumerate_instance_layer_properties()
                    .is_ok_and(|properties| {
                        properties
                            .iter()
                            .any(|p| p.layer_name_as_c_str() == Ok(c"VK_LAYER_KHRONOS_validation"))
                    })
            };

            if is_validation_supported {
                instance_info = instance_info.enabled_layer_names(&layers);
            }
        }

        let instance = unsafe { entry.create_instance(&instance_info, None).unwrap() };
        let physical = unsafe { instance.enumerate_physical_devices().unwrap()[0] };

        let family_index = unsafe {
            instance
                .get_physical_device_queue_family_properties(physical)
                .iter()
                .position(|q| q.queue_flags.contains(vk::QueueFlags::GRAPHICS))
                .unwrap()
        };

        let queue_info = [vk::DeviceQueueCreateInfo::default()
            .queue_family_index(family_index as u32)
            .queue_priorities(&[1.0])];

        let device_extensions = [vk::KHR_SWAPCHAIN_NAME.as_ptr()];

        let device_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_info)
            .enabled_extension_names(&device_extensions);

        let device = unsafe {
            instance
                .create_device(physical, &device_info, None)
                .unwrap()
        };
        let queue = unsafe { device.get_device_queue(family_index as u32, 0) };

        Self {
            entry,
            instance,
            physical,
            device,
            family_index: family_index as u32,
            queue,
        }
    }
}

pub(crate) struct VulkanWindow {
    entry:           ash::Entry,
    instance:        ash::Instance,
    device:          ash::Device,
    queue:           vk::Queue,
    skia_context:    skia_safe::gpu::DirectContext,
    surface:         vk::SurfaceKHR,
    capabilities:    vk::SurfaceCapabilitiesKHR,
    format:          vk::Format,
    swapchain:       vk::SwapchainKHR,
    pre_transform:   vk::SurfaceTransformFlagsKHR,
    composite_alpha: vk::CompositeAlphaFlagsKHR,
    command_pool:    vk::CommandPool,
    command_buffer:  vk::CommandBuffer,
    image_available: Vec<vk::Semaphore>,
    render_finished: Vec<vk::Semaphore>,
    in_flight:       vk::Fence,
    images:          Vec<vk::Image>,
    surfaces:        Vec<Option<skia_safe::Surface>>,
    current_frame:   u32,
    width:           u32,
    height:          u32,
}

impl Drop for VulkanWindow {
    fn drop(&mut self) {
        unsafe {
            self.surfaces.clear();

            self.skia_context.flush_and_submit();
            self.skia_context.submit(None);

            let _ = self.device.device_wait_idle();
            let _ = self.device.queue_wait_idle(self.queue);

            let device = ash::khr::swapchain::Device::new(&self.instance, &self.device);
            device.destroy_swapchain(self.swapchain, None);

            for &image_available in &self.image_available {
                self.device.destroy_semaphore(image_available, None);
            }

            for &render_finished in &self.render_finished {
                self.device.destroy_semaphore(render_finished, None);
            }

            self.device.destroy_fence(self.in_flight, None);
            self.device.destroy_command_pool(self.command_pool, None);

            let instance = ash::khr::surface::Instance::new(&self.entry, &self.instance);

            instance.destroy_surface(self.surface, None);
        }
    }
}

impl VulkanWindow {
    pub(crate) unsafe fn new(context: &mut VulkanContext, window: &Window) -> Self {
        let skia_context = unsafe {
            let get_proc = |gpo| match gpo {
                skia_safe::gpu::vk::GetProcOf::Instance(instance, name) => {
                    let instance = vk::Instance::from_raw(instance as _);
                    context
                        .entry
                        .get_instance_proc_addr(instance, name)
                        .unwrap() as _
                }

                skia_safe::gpu::vk::GetProcOf::Device(device, name) => {
                    let device = vk::Device::from_raw(device as _);
                    (context.instance.fp_v1_0().get_device_proc_addr)(device, name).unwrap() as _
                }
            };

            skia_safe::gpu::direct_contexts::make_vulkan(
                &skia_safe::gpu::vk::BackendContext::new(
                    context.instance.handle().as_raw() as _,
                    context.physical.as_raw() as _,
                    context.device.handle().as_raw() as _,
                    (
                        context.queue.as_raw() as _,
                        context.family_index as usize,
                    ),
                    &get_proc,
                ),
                None,
            )
            .unwrap()
        };

        let surface = unsafe {
            ash_window::create_surface(
                &context.entry,
                &context.instance,
                window.display_handle().unwrap().as_raw(),
                window.window_handle().unwrap().as_raw(),
                None,
            )
            .unwrap()
        };

        let instance = ash::khr::surface::Instance::new(&context.entry, &context.instance);
        let capabilities = unsafe {
            instance
                .get_physical_device_surface_capabilities(context.physical, surface)
                .unwrap()
        };

        let format = unsafe {
            instance
                .get_physical_device_surface_formats(context.physical, surface)
                .unwrap()
                .into_iter()
                .next()
                .unwrap()
                .format
        };

        let composite_alpha = if capabilities
            .supported_composite_alpha
            .contains(vk::CompositeAlphaFlagsKHR::PRE_MULTIPLIED)
        {
            vk::CompositeAlphaFlagsKHR::PRE_MULTIPLIED
        } else {
            vk::CompositeAlphaFlagsKHR::OPAQUE
        };

        let size = window.inner_size();
        let device = ash::khr::swapchain::Device::new(&context.instance, &context.device);
        let swapchain_info = vk::SwapchainCreateInfoKHR::default()
            .surface(surface)
            .min_image_count(capabilities.min_image_count.max(2))
            .image_array_layers(1)
            .image_format(format)
            .image_color_space(vk::ColorSpaceKHR::SRGB_NONLINEAR)
            .image_extent(vk::Extent2D {
                width:  size.width,
                height: size.height,
            })
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(capabilities.current_transform)
            .composite_alpha(composite_alpha)
            .present_mode(vk::PresentModeKHR::FIFO);

        let swapchain = unsafe { device.create_swapchain(&swapchain_info, None).unwrap() };

        let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);
        let in_flight = unsafe { context.device.create_fence(&fence_info, None).unwrap() };

        let pool_info = vk::CommandPoolCreateInfo::default()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

        let command_pool = unsafe {
            context
                .device
                .create_command_pool(&pool_info, None)
                .unwrap()
        };

        let buffer_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .command_buffer_count(1)
            .level(vk::CommandBufferLevel::PRIMARY);

        let command_buffer = unsafe {
            context
                .device
                .allocate_command_buffers(&buffer_info)
                .unwrap()
                .into_iter()
                .next()
                .unwrap()
        };

        let mut this = Self {
            entry: context.entry.clone(),
            instance: context.instance.clone(),
            device: context.device.clone(),
            queue: context.queue,
            skia_context,
            surface,
            capabilities,
            format,
            swapchain,
            pre_transform: capabilities.current_transform,
            composite_alpha,
            command_pool,
            command_buffer,
            image_available: Vec::new(),
            render_finished: Vec::new(),
            in_flight,
            images: Vec::new(),
            surfaces: Vec::new(),
            current_frame: 0,
            width: size.width,
            height: size.height,
        };

        unsafe { this.recreate_swapchain(size.width, size.height) };

        this
    }

    pub(crate) unsafe fn recreate_swapchain(&mut self, width: u32, height: u32) {
        self.surfaces.clear();

        unsafe { self.device.device_wait_idle().unwrap() };

        let device = ash::khr::swapchain::Device::new(&self.instance, &self.device);

        unsafe {
            device.destroy_swapchain(self.swapchain, None);
        }

        let swapchain_info = vk::SwapchainCreateInfoKHR::default()
            .surface(self.surface)
            .min_image_count(self.capabilities.min_image_count.max(2))
            .image_array_layers(1)
            .image_format(self.format)
            .image_color_space(vk::ColorSpaceKHR::SRGB_NONLINEAR)
            .image_extent(vk::Extent2D { width, height })
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(self.pre_transform)
            .composite_alpha(self.composite_alpha)
            .present_mode(vk::PresentModeKHR::FIFO);

        self.swapchain = unsafe { device.create_swapchain(&swapchain_info, None).unwrap() };
        self.images = unsafe { device.get_swapchain_images(self.swapchain).unwrap() };
        self.surfaces = vec![None; self.images.len()];
        self.width = width;
        self.height = height;

        while self.image_available.len() < self.images.len() {
            let image_available = unsafe {
                self.device
                    .create_semaphore(&Default::default(), None)
                    .unwrap()
            };

            self.image_available.push(image_available);
        }

        while self.render_finished.len() < self.images.len() {
            let render_finished = unsafe {
                self.device
                    .create_semaphore(&Default::default(), None)
                    .unwrap()
            };

            self.render_finished.push(render_finished);
        }
    }

    fn get_skia_surface(&mut self, index: u32) -> &mut skia_safe::Surface {
        if self.surfaces[index as usize].is_some() {
            return self.surfaces[index as usize].as_mut().unwrap();
        }

        let (format, color_type) = match self.format {
            vk::Format::B8G8R8A8_UNORM => (
                skia_safe::gpu::vk::Format::B8G8R8A8_UNORM,
                skia_safe::ColorType::BGRA8888,
            ),
            vk::Format::R16G16B16A16_SFLOAT => (
                skia_safe::gpu::vk::Format::R16G16B16A16_SFLOAT,
                skia_safe::ColorType::RGBAF16,
            ),
            _ => todo!(),
        };

        let image_info = unsafe {
            skia_safe::gpu::vk::ImageInfo::new(
                self.images[index as usize].as_raw() as _,
                skia_safe::gpu::vk::Alloc::default(),
                skia_safe::gpu::vk::ImageTiling::OPTIMAL,
                skia_safe::gpu::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                format,
                1,
                None,
                None,
                None,
                None,
            )
        };

        let render_target = skia_safe::gpu::backend_render_targets::make_vk(
            (self.width as i32, self.height as i32),
            &image_info,
        );

        let surface = skia_safe::gpu::surfaces::wrap_backend_render_target(
            &mut self.skia_context,
            &render_target,
            skia_safe::gpu::SurfaceOrigin::TopLeft,
            color_type,
            None,
            None,
        )
        .unwrap();

        self.surfaces[index as usize] = Some(surface);
        self.surfaces[index as usize].as_mut().unwrap()
    }

    pub(crate) unsafe fn draw<T>(&mut self, f: impl FnOnce(&skia_safe::Canvas) -> T) -> T {
        unsafe {
            self.device
                .wait_for_fences(&[self.in_flight], true, u64::MAX)
                .unwrap();

            self.device.reset_fences(&[self.in_flight]).unwrap();
        }

        let device = ash::khr::swapchain::Device::new(&self.instance, &self.device);

        let (index, suboptimal) = unsafe {
            device
                .acquire_next_image(
                    self.swapchain,
                    u64::MAX,
                    self.image_available[self.current_frame as usize],
                    vk::Fence::null(),
                )
                .unwrap()
        };

        if suboptimal {
            println!("suboptimal");
        }

        unsafe {
            self.transition_before(
                self.image_available[self.current_frame as usize],
                index,
            )
        };

        let surface = self.get_skia_surface(index);
        let canvas = surface.canvas();

        let output = f(canvas);

        self.skia_context.flush_and_submit();

        unsafe {
            self.transition_after(
                self.render_finished[self.current_frame as usize],
                index,
            )
        };

        let swapchains = [self.swapchain];
        let indices = [index];
        let semaphores = [self.render_finished[self.current_frame as usize]];
        let present_info = vk::PresentInfoKHR::default()
            .swapchains(&swapchains)
            .image_indices(&indices)
            .wait_semaphores(&semaphores);

        unsafe {
            device.queue_present(self.queue, &present_info).unwrap();
        };

        self.current_frame = (self.current_frame + 1) % self.images.len() as u32;

        output
    }

    unsafe fn transition_before(&self, semaphore: vk::Semaphore, image_index: u32) {
        let begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe {
            self.device
                .begin_command_buffer(self.command_buffer, &begin_info)
                .unwrap()
        };

        let barrier = vk::ImageMemoryBarrier::default()
            .image(self.images[image_index as usize])
            .src_access_mask(vk::AccessFlags::empty())
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
            .old_layout(vk::ImageLayout::UNDEFINED)
            .new_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask:      vk::ImageAspectFlags::COLOR,
                base_mip_level:   0,
                level_count:      1,
                base_array_layer: 0,
                layer_count:      1,
            });

        unsafe {
            self.device.cmd_pipeline_barrier(
                self.command_buffer,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[barrier],
            );
        }

        unsafe {
            self.device.end_command_buffer(self.command_buffer).unwrap();
        }

        let buffer = [self.command_buffer];
        let semaphores = [semaphore];
        let submit = vk::SubmitInfo::default()
            .command_buffers(&buffer)
            .wait_semaphores(&semaphores)
            .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT]);

        unsafe {
            self.device
                .queue_submit(self.queue, &[submit], self.in_flight)
                .unwrap();

            self.device
                .wait_for_fences(&[self.in_flight], true, u64::MAX)
                .unwrap();

            self.device.reset_fences(&[self.in_flight]).unwrap();
        }
    }

    unsafe fn transition_after(&self, semaphore: vk::Semaphore, image_index: u32) {
        let begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe {
            self.device
                .begin_command_buffer(self.command_buffer, &begin_info)
                .unwrap()
        };

        let barrier = vk::ImageMemoryBarrier::default()
            .image(self.images[image_index as usize])
            .src_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
            .dst_access_mask(vk::AccessFlags::empty())
            .old_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .new_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask:      vk::ImageAspectFlags::COLOR,
                base_mip_level:   0,
                level_count:      1,
                base_array_layer: 0,
                layer_count:      1,
            });

        unsafe {
            self.device.cmd_pipeline_barrier(
                self.command_buffer,
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[barrier],
            );

            self.device.end_command_buffer(self.command_buffer).unwrap();
        }

        let buffer = [self.command_buffer];
        let semaphores = [semaphore];
        let submit = vk::SubmitInfo::default()
            .command_buffers(&buffer)
            .signal_semaphores(&semaphores);

        unsafe {
            self.device
                .queue_submit(self.queue, &[submit], self.in_flight)
                .unwrap();
        }
    }
}
