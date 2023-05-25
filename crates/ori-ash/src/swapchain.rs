use std::sync::{Arc, Mutex};

use ash::{extensions::khr, vk};

use crate::{AshDevice, AshError, AshSurface};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum PresentMode {
    Immediate,
    Mailbox,
    #[default]
    Fifo,
    FifoRelaxed,
}

impl PresentMode {
    pub fn to_vk(&self) -> vk::PresentModeKHR {
        match self {
            Self::Immediate => vk::PresentModeKHR::IMMEDIATE,
            Self::Mailbox => vk::PresentModeKHR::MAILBOX,
            Self::Fifo => vk::PresentModeKHR::FIFO,
            Self::FifoRelaxed => vk::PresentModeKHR::FIFO_RELAXED,
        }
    }

    pub fn find_best(&self, present_modes: &[vk::PresentModeKHR]) -> vk::PresentModeKHR {
        let desired = self.to_vk();
        let found = present_modes.iter().copied().find(|&mode| mode == desired);
        found.unwrap_or(vk::PresentModeKHR::FIFO)
    }
}

struct AshSwaphchainInner {
    loader: khr::Swapchain,
    swapchain: vk::SwapchainKHR,
    desired_image_count: u32,
    surface_format: vk::SurfaceFormatKHR,
    extent: vk::Extent2D,
    present_mode: vk::PresentModeKHR,
    pre_transform: vk::SurfaceTransformFlagsKHR,

    present_images: Vec<vk::Image>,
    present_image_views: Vec<vk::ImageView>,

    surface: AshSurface,
    device: AshDevice,
}

impl Drop for AshSwaphchainInner {
    fn drop(&mut self) {
        unsafe {
            self.loader.destroy_swapchain(self.swapchain, None);

            for &image_view in &self.present_image_views {
                self.device.destroy_image_view(image_view, None);
            }
        }
    }
}

#[derive(Clone)]
pub struct AshSwapchain {
    inner: Arc<Mutex<AshSwaphchainInner>>,
}

impl AshSwapchain {
    unsafe fn create_image_views(
        device: &AshDevice,
        surface_format: vk::SurfaceFormatKHR,
        images: &[vk::Image],
    ) -> Result<Vec<vk::ImageView>, AshError> {
        let mut image_views = Vec::with_capacity(images.len());
        for &image in images {
            let create_info = vk::ImageViewCreateInfo {
                image,
                view_type: vk::ImageViewType::TYPE_2D,
                format: surface_format.format,
                components: vk::ComponentMapping {
                    r: vk::ComponentSwizzle::IDENTITY,
                    g: vk::ComponentSwizzle::IDENTITY,
                    b: vk::ComponentSwizzle::IDENTITY,
                    a: vk::ComponentSwizzle::IDENTITY,
                },
                subresource_range: vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                },
                ..Default::default()
            };

            let image_view = device.create_image_view(&create_info, None)?;
            image_views.push(image_view);
        }

        Ok(image_views)
    }

    pub unsafe fn new(
        surface: &AshSurface,
        device: &AshDevice,
        present_mode: PresentMode,
        width: u32,
        height: u32,
    ) -> Result<Self, AshError> {
        let surface_format = surface.supported_formats(device)?[0];
        let capabilities = surface.capabilities(device)?;

        let mut desired_image_count = capabilities.min_image_count + 1;
        if capabilities.max_image_count > 0 && desired_image_count > capabilities.max_image_count {
            desired_image_count = capabilities.max_image_count;
        }

        let surface_resolution = match capabilities.current_extent.width {
            std::u32::MAX => vk::Extent2D { width, height },
            _ => capabilities.current_extent,
        };

        let transform_flags = vk::SurfaceTransformFlagsKHR::IDENTITY;
        let pre_transform = if capabilities.supported_transforms.contains(transform_flags) {
            vk::SurfaceTransformFlagsKHR::IDENTITY
        } else {
            capabilities.current_transform
        };

        let present_modes = surface.present_modes(device)?;
        let present_mode = present_mode.find_best(&present_modes);

        let create_info = vk::SwapchainCreateInfoKHR {
            surface: surface.surface(),
            min_image_count: desired_image_count,
            image_format: surface_format.format,
            image_color_space: surface_format.color_space,
            image_extent: surface_resolution,
            image_array_layers: 1,
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
            image_sharing_mode: vk::SharingMode::EXCLUSIVE,
            pre_transform,
            composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
            present_mode,
            clipped: vk::TRUE,
            ..Default::default()
        };

        let loader = khr::Swapchain::new(surface.instance(), device);
        let swapchain = loader.create_swapchain(&create_info, None)?;

        let present_images = loader.get_swapchain_images(swapchain)?;
        let present_image_views =
            Self::create_image_views(device, surface_format, &present_images)?;

        let inner = AshSwaphchainInner {
            loader,
            swapchain,
            desired_image_count,
            surface_format,
            extent: surface_resolution,
            present_mode,
            pre_transform,

            present_images,
            present_image_views,

            surface: surface.clone(),
            device: device.clone(),
        };

        Ok(Self {
            inner: Arc::new(Mutex::new(inner)),
        })
    }

    pub unsafe fn recreate(
        &self,
        present_mode: PresentMode,
        width: u32,
        height: u32,
    ) -> Result<(), AshError> {
        let mut inner = self.inner.lock().unwrap();

        let capabilities = inner.surface.capabilities(&inner.device)?;
        let surface_resolution = match capabilities.current_extent.width {
            std::u32::MAX => vk::Extent2D { width, height },
            _ => capabilities.current_extent,
        };

        let present_modes = inner.surface.present_modes(&inner.device)?;
        let present_mode = present_mode.find_best(&present_modes);

        let create_info = vk::SwapchainCreateInfoKHR {
            surface: inner.surface.surface(),
            min_image_count: inner.desired_image_count,
            image_format: inner.surface_format.format,
            image_color_space: inner.surface_format.color_space,
            image_extent: surface_resolution,
            image_array_layers: 1,
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
            image_sharing_mode: vk::SharingMode::EXCLUSIVE,
            pre_transform: inner.pre_transform,
            composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
            present_mode,
            clipped: vk::TRUE,
            old_swapchain: inner.swapchain,
            ..Default::default()
        };

        let swapchain = inner.loader.create_swapchain(&create_info, None)?;
        inner.loader.destroy_swapchain(inner.swapchain, None);
        inner.swapchain = swapchain;
        inner.extent = surface_resolution;

        for &image_view in &inner.present_image_views {
            inner.device.destroy_image_view(image_view, None);
        }

        inner.present_images = inner.loader.get_swapchain_images(inner.swapchain)?;
        inner.present_image_views =
            Self::create_image_views(&inner.device, inner.surface_format, &inner.present_images)?;

        Ok(())
    }

    pub unsafe fn acquire_next_image(
        &self,
        timeout: u64,
        semaphore: vk::Semaphore,
        fence: vk::Fence,
    ) -> Result<(u32, bool), vk::Result> {
        (self.loader()).acquire_next_image(self.swapchain(), timeout, semaphore, fence)
    }

    pub fn loader(&self) -> khr::Swapchain {
        self.inner.lock().unwrap().loader.clone()
    }

    pub fn swapchain(&self) -> vk::SwapchainKHR {
        self.inner.lock().unwrap().swapchain
    }

    pub fn desired_image_count(&self) -> u32 {
        self.inner.lock().unwrap().desired_image_count
    }

    pub fn surface_format(&self) -> vk::SurfaceFormatKHR {
        self.inner.lock().unwrap().surface_format
    }

    pub fn extent(&self) -> vk::Extent2D {
        self.inner.lock().unwrap().extent
    }

    pub fn present_mode(&self) -> vk::PresentModeKHR {
        self.inner.lock().unwrap().present_mode
    }

    pub fn pre_transform(&self) -> vk::SurfaceTransformFlagsKHR {
        self.inner.lock().unwrap().pre_transform
    }

    pub fn present_images(&self) -> Vec<vk::Image> {
        self.inner.lock().unwrap().present_images.clone()
    }

    pub fn present_image_views(&self) -> Vec<vk::ImageView> {
        self.inner.lock().unwrap().present_image_views.clone()
    }

    pub fn surface(&self) -> AshSurface {
        self.inner.lock().unwrap().surface.clone()
    }

    pub fn device(&self) -> AshDevice {
        self.inner.lock().unwrap().device.clone()
    }
}
