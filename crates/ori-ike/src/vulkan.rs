use std::slice;

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
        let validation_layers = [c"VK_LAYER_KHRONOS_validation"];
        let validation_layer_names = validation_layers
            .into_iter()
            .map(|n| n.as_ptr())
            .collect::<Vec<_>>();

        let mut instance_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(extensions);

        if cfg!(debug_assertions) {
            let is_validation_supported = unsafe {
                validation_layers.iter().all(|name| {
                    entry
                        .enumerate_instance_layer_properties()
                        .is_ok_and(|properties| {
                            properties
                                .iter()
                                .any(|p| p.layer_name_as_c_str() == Ok(*name))
                        })
                })
            };

            if is_validation_supported {
                instance_info = instance_info.enabled_layer_names(&validation_layer_names);
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
    entry:            ash::Entry,
    instance:         ash::Instance,
    device:           ash::Device,
    queue:            vk::Queue,
    skia_context:     skia_safe::gpu::DirectContext,
    surface:          vk::SurfaceKHR,
    capabilities:     vk::SurfaceCapabilitiesKHR,
    present_mode:     vk::PresentModeKHR,
    format:           vk::Format,
    swapchain:        vk::SwapchainKHR,
    pre_transform:    vk::SurfaceTransformFlagsKHR,
    composite_alpha:  vk::CompositeAlphaFlagsKHR,
    command_pool:     vk::CommandPool,
    command_buffers:  Vec<vk::CommandBuffer>,
    image_available:  Vec<vk::Semaphore>,
    render_finished:  Vec<vk::Semaphore>,
    in_flight:        Vec<vk::Fence>,
    swapchain_images: Vec<vk::Image>,
    skia_surfaces:    Vec<(skia_safe::Surface, vk::Image)>,
    current_frame:    u32,
    width:            u32,
    height:           u32,
}

impl Drop for VulkanWindow {
    fn drop(&mut self) {
        unsafe {
            let _ = self.device.device_wait_idle();

            let device = ash::khr::swapchain::Device::new(&self.instance, &self.device);
            device.destroy_swapchain(self.swapchain, None);

            for &image_available in &self.image_available {
                self.device.destroy_semaphore(image_available, None);
            }

            for &render_finished in &self.render_finished {
                self.device.destroy_semaphore(render_finished, None);
            }

            for &in_flight in &self.in_flight {
                self.device.destroy_fence(in_flight, None);
            }

            self.skia_surfaces.clear();
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

        let present_modes = unsafe {
            instance
                .get_physical_device_surface_present_modes(context.physical, surface)
                .unwrap()
        };

        let present_mode = if present_modes.contains(&vk::PresentModeKHR::MAILBOX) {
            vk::PresentModeKHR::MAILBOX
        } else if present_modes.contains(&vk::PresentModeKHR::IMMEDIATE) {
            vk::PresentModeKHR::IMMEDIATE
        } else {
            vk::PresentModeKHR::FIFO
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

        let pool_info = vk::CommandPoolCreateInfo::default()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

        let command_pool = unsafe {
            context
                .device
                .create_command_pool(&pool_info, None)
                .unwrap()
        };

        let size = window.inner_size();
        let mut this = Self {
            entry: context.entry.clone(),
            instance: context.instance.clone(),
            device: context.device.clone(),
            queue: context.queue,
            skia_context,
            surface,
            capabilities,
            present_mode,
            format,
            swapchain: vk::SwapchainKHR::null(),
            pre_transform: capabilities.current_transform,
            composite_alpha,
            command_pool,
            command_buffers: Vec::new(),
            image_available: Vec::new(),
            render_finished: Vec::new(),
            in_flight: Vec::new(),
            swapchain_images: Vec::new(),
            skia_surfaces: Vec::new(),
            current_frame: 0,
            width: size.width,
            height: size.height,
        };

        unsafe { this.recreate_swapchain(size.width, size.height) };

        this
    }

    pub(crate) unsafe fn recreate_swapchain(&mut self, width: u32, height: u32) {
        unsafe { self.device.device_wait_idle().unwrap() };

        self.skia_surfaces.clear();

        let device = ash::khr::swapchain::Device::new(&self.instance, &self.device);

        unsafe { device.destroy_swapchain(self.swapchain, None) };

        let swapchain_info = vk::SwapchainCreateInfoKHR::default()
            .surface(self.surface)
            .min_image_count(self.capabilities.min_image_count.max(2))
            .image_array_layers(1)
            .image_format(self.format)
            .image_color_space(vk::ColorSpaceKHR::SRGB_NONLINEAR)
            .image_extent(vk::Extent2D { width, height })
            .image_usage(vk::ImageUsageFlags::TRANSFER_DST)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(self.pre_transform)
            .composite_alpha(self.composite_alpha)
            .present_mode(self.present_mode);

        self.swapchain = unsafe { device.create_swapchain(&swapchain_info, None).unwrap() };
        self.swapchain_images = unsafe { device.get_swapchain_images(self.swapchain).unwrap() };
        self.width = width;
        self.height = height;

        if self.command_buffers.len() < self.swapchain_images.len() {
            let buffer_info = vk::CommandBufferAllocateInfo::default()
                .command_pool(self.command_pool)
                .command_buffer_count(self.swapchain_images.len() as u32)
                .level(vk::CommandBufferLevel::PRIMARY);

            self.command_buffers =
                unsafe { self.device.allocate_command_buffers(&buffer_info).unwrap() };
        }

        while self.image_available.len() < self.swapchain_images.len() {
            let image_available = unsafe {
                self.device
                    .create_semaphore(&Default::default(), None)
                    .unwrap()
            };

            self.image_available.push(image_available);
        }

        while self.render_finished.len() < self.swapchain_images.len() {
            let render_finished = unsafe {
                self.device
                    .create_semaphore(&Default::default(), None)
                    .unwrap()
            };

            self.render_finished.push(render_finished);
        }

        while self.in_flight.len() < self.swapchain_images.len() {
            let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);
            let in_flight = unsafe { self.device.create_fence(&fence_info, None).unwrap() };

            self.in_flight.push(in_flight);
        }

        while self.skia_surfaces.len() < self.swapchain_images.len() {
            let color_type = match self.format {
                vk::Format::R16G16B16A16_SFLOAT => skia_safe::ColorType::RGBAF16,
                vk::Format::B8G8R8A8_UNORM => skia_safe::ColorType::BGRA8888,
                _ => panic!(
                    "unsupported format: `{:?}`",
                    self.format
                ),
            };

            let image_info = skia_safe::ImageInfo::new(
                skia_safe::ISize::new(width as i32, height as i32),
                color_type,
                skia_safe::AlphaType::Premul,
                None,
            );

            let mut surface = skia_safe::gpu::surfaces::render_target(
                &mut self.skia_context,
                skia_safe::gpu::Budgeted::Yes,
                &image_info,
                Some(4),
                skia_safe::gpu::SurfaceOrigin::TopLeft,
                None,
                false,
                None,
            )
            .unwrap();

            let target = skia_safe::gpu::surfaces::get_backend_render_target(
                &mut surface,
                skia_safe::surface::BackendHandleAccess::FlushRead,
            )
            .unwrap();

            let image = vk::Image::from_raw(*target.vulkan_image_info().unwrap().image() as _);

            self.skia_surfaces.push((surface, image));
        }
    }

    pub(crate) unsafe fn draw<T>(
        &mut self,
        window: &Window,
        f: impl FnOnce(&skia_safe::Canvas) -> T,
    ) -> Option<T> {
        let command_buffer = self.command_buffers[self.current_frame as usize];
        let image_available = self.image_available[self.current_frame as usize];
        let in_flight = self.in_flight[self.current_frame as usize];

        unsafe {
            // wait for resources to be available
            self.device
                .wait_for_fences(&[in_flight], true, u64::MAX)
                .unwrap();

            self.device.reset_fences(&[in_flight]).unwrap();

            // acquire swapchain image
            let device = ash::khr::swapchain::Device::new(&self.instance, &self.device);

            let (image_index, suboptimal) = match device.acquire_next_image(
                self.swapchain,
                u64::MAX,
                image_available,
                vk::Fence::null(),
            ) {
                Ok(image) => image,
                Err(err) => {
                    println!("err: {err:?}");
                    return None;
                }
            };

            if suboptimal {
                println!("suboptimal");
            }

            let swapchain_image = self.swapchain_images[image_index as usize];
            let render_finished = self.render_finished[image_index as usize];

            // do rendering
            let (surface, skia_image) = &mut self.skia_surfaces[image_index as usize];
            let canvas = surface.canvas();

            let output = f(canvas);

            // IMPORTANT: resolve skia msaa
            skia_safe::gpu::surfaces::resolve_msaa(surface);
            self.skia_context.flush_and_submit();

            // record command buffer
            let begin_info = vk::CommandBufferBeginInfo::default()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

            self.device
                .begin_command_buffer(command_buffer, &begin_info)
                .unwrap();

            let range = vk::ImageSubresourceRange::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .layer_count(1)
                .level_count(1);

            let skia_to_transfer_src = vk::ImageMemoryBarrier::default()
                .image(*skia_image)
                .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .new_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
                .src_access_mask(vk::AccessFlags::NONE)
                .dst_access_mask(vk::AccessFlags::TRANSFER_READ)
                .subresource_range(range);

            let swapchain_to_transfer_dst = vk::ImageMemoryBarrier::default()
                .image(swapchain_image)
                .old_layout(vk::ImageLayout::UNDEFINED)
                .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .src_access_mask(vk::AccessFlags::TRANSFER_READ)
                .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                .subresource_range(range);

            self.device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[skia_to_transfer_src, swapchain_to_transfer_dst],
            );

            self.device.cmd_copy_image(
                command_buffer,
                *skia_image,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                swapchain_image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[vk::ImageCopy::default()
                    .src_offset(Default::default())
                    .src_subresource(
                        vk::ImageSubresourceLayers::default()
                            .aspect_mask(vk::ImageAspectFlags::COLOR)
                            .layer_count(1),
                    )
                    .dst_offset(Default::default())
                    .dst_subresource(
                        vk::ImageSubresourceLayers::default()
                            .aspect_mask(vk::ImageAspectFlags::COLOR)
                            .layer_count(1),
                    )
                    .extent(
                        vk::Extent3D::default()
                            .width(self.width)
                            .height(self.height)
                            .depth(1),
                    )],
            );

            let skia_to_transfer_dst = vk::ImageMemoryBarrier::default()
                .image(*skia_image)
                .old_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
                .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .src_access_mask(vk::AccessFlags::TRANSFER_READ)
                .dst_access_mask(vk::AccessFlags::TRANSFER_READ)
                .subresource_range(range);

            let swapchain_to_present = vk::ImageMemoryBarrier::default()
                .image(swapchain_image)
                .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .new_layout(vk::ImageLayout::PRESENT_SRC_KHR)
                .src_access_mask(vk::AccessFlags::TRANSFER_READ)
                .dst_access_mask(vk::AccessFlags::TRANSFER_READ)
                .subresource_range(range);

            self.device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[skia_to_transfer_dst, swapchain_to_present],
            );

            self.device.end_command_buffer(command_buffer).unwrap();

            let submit = vk::SubmitInfo::default()
                .wait_semaphores(slice::from_ref(&image_available))
                .wait_dst_stage_mask(&[vk::PipelineStageFlags::TRANSFER])
                .command_buffers(slice::from_ref(&command_buffer))
                .signal_semaphores(slice::from_ref(&render_finished));

            self.device
                .queue_submit(self.queue, &[submit], in_flight)
                .unwrap();

            let present_info = vk::PresentInfoKHR::default()
                .swapchains(slice::from_ref(&self.swapchain))
                .image_indices(slice::from_ref(&image_index))
                .wait_semaphores(slice::from_ref(&render_finished));

            window.pre_present_notify();
            device.queue_present(self.queue, &present_info).unwrap();

            self.current_frame = (self.current_frame + 1) % self.swapchain_images.len() as u32;

            Some(output)
        }
    }
}
