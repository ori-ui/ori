use std::{
    fmt::Debug,
    ops::Deref,
    sync::{Arc, Mutex, MutexGuard},
};

use ash::{extensions::khr, vk};
use gpu_alloc::{GpuAllocator, MemoryBlock, Request};
use gpu_alloc_ash::AshMemoryDevice;

use crate::{AshError, AshInstance, AshSurface};

struct AshDeviceInner {
    physical: vk::PhysicalDevice,
    device: ash::Device,
    queue: vk::Queue,
    queue_family_index: u32,

    // allocator for the device
    allocator: Mutex<GpuAllocator<vk::DeviceMemory>>,

    // whether or not the device is owned by this struct
    // this is used to determine whether or not to drop the device
    // when this struct is dropped
    //
    // this is useful for when the device is created externally
    owned: bool,

    // keep a reference to the instance so it doesn't get dropped
    instance: AshInstance,
}

impl Drop for AshDeviceInner {
    fn drop(&mut self) {
        if self.owned {
            unsafe { self.device.destroy_device(None) };
        }
    }
}

#[derive(Clone)]
pub struct AshDevice {
    inner: Arc<AshDeviceInner>,
}

impl Debug for AshDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AshDevice")
            .field("physical", &self.inner.physical)
            .field("queue", &self.inner.queue)
            .field("queue_family", &self.inner.queue_family_index)
            .field("owned", &self.inner.owned)
            .finish()
    }
}

impl AshDevice {
    pub fn required_extensions() -> &'static [*const i8] {
        const EXT: &[*const i8] = &[khr::Swapchain::name().as_ptr()];
        EXT
    }

    unsafe fn get_queue_families(
        instance: &AshInstance,
        physical: vk::PhysicalDevice,
    ) -> Vec<vk::DeviceQueueCreateInfo> {
        let mut families = Vec::new();

        let queue_families = instance.get_physical_device_queue_family_properties(physical);
        for (index, queue_family) in queue_families.iter().enumerate() {
            if !queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                continue;
            }

            let create_info = vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(index as u32)
                .queue_priorities(&[1.0])
                .build();

            families.push(create_info);
        }

        families
    }

    unsafe fn select_physical_device(
        surface: &AshSurface,
        devices: &[vk::PhysicalDevice],
    ) -> Result<(vk::PhysicalDevice, u32), AshError> {
        for &physical in devices {
            let queue_families = Self::get_queue_families(&surface.instance(), physical);

            for (index, _) in queue_families.iter().enumerate() {
                let surface_supported = surface.get_physical_device_surface_support(
                    physical,
                    index as u32,
                    surface.surface(),
                )?;

                if surface_supported {
                    return Ok((physical, index as u32));
                }
            }
        }

        Err(AshError::DeviceNotFound)
    }

    /// Create a device from an existing [`AshInstance`] and [`ash::Device`].
    ///
    /// This will not destroy the device when this struct is dropped, so it must be
    /// destroyed manually.
    ///
    /// # Safety
    /// - The `instance` must be the same instance that was used to create the `device`.
    /// - The `device` must not be destroyed while this struct is alive.
    /// - The `physical` must be the physical device that was used to create the `device`.
    /// - The `queue_family` must be a valid queue family index for the `physical` device.
    /// - The `queue_family` must support graphics operations.
    pub unsafe fn external(
        instance: &AshInstance,
        physical: vk::PhysicalDevice,
        device: ash::Device,
        queue_family: u32,
        allocator: GpuAllocator<vk::DeviceMemory>,
    ) -> Self {
        let queue = device.get_device_queue(queue_family, 0);

        let inner = AshDeviceInner {
            physical,
            device,
            queue,
            queue_family_index: queue_family,
            allocator: Mutex::new(allocator),
            owned: false,
            instance: instance.clone(),
        };

        Self {
            inner: Arc::new(inner),
        }
    }

    /// Create a device from an existing [`AshInstance`].
    pub unsafe fn create(
        instance: &AshInstance,
        surface: &AshSurface,
        extensions: &[*const i8],
    ) -> Result<Self, AshError> {
        let devices = instance.enumerate_physical_devices()?;
        let (physical, queue_family) = Self::select_physical_device(surface, &devices)?;

        let queue_families = Self::get_queue_families(&instance, physical);

        let create_info = vk::DeviceCreateInfo {
            queue_create_info_count: queue_families.len() as u32,
            p_queue_create_infos: queue_families.as_ptr(),
            enabled_extension_count: extensions.len() as u32,
            pp_enabled_extension_names: extensions.as_ptr(),
            ..Default::default()
        };
        let device = instance.create_device(physical, &create_info, None)?;
        let queue = device.get_device_queue(queue_family, 0);

        let config = gpu_alloc::Config::i_am_prototyping();
        let props = gpu_alloc_ash::device_properties(&instance, vk::API_VERSION_1_3, physical)?;
        let allocator = GpuAllocator::new(config, props);

        let inner = AshDeviceInner {
            physical,
            device,
            queue,
            queue_family_index: 0,
            allocator: Mutex::new(allocator),
            owned: true,
            instance: instance.clone(),
        };

        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    /// Get a reference to the [`ash::Device`] that this struct wraps.
    pub fn device(&self) -> &ash::Device {
        &self.inner.device
    }

    /// Get the physical device that was used to create this device.
    pub fn physical(&self) -> vk::PhysicalDevice {
        self.inner.physical
    }

    /// Get a reference to the [`AshInstance`] that created this device.
    pub fn instance(&self) -> &AshInstance {
        &self.inner.instance
    }

    /// Get the queue that was created with this device.
    pub fn queue(&self) -> vk::Queue {
        self.inner.queue
    }

    /// Get the queue family index that was used to create this device.
    pub fn queue_family_index(&self) -> u32 {
        self.inner.queue_family_index
    }

    /// Get a reference to the [`GpuAllocator`] that was created with this device.
    pub fn allocator(&self) -> MutexGuard<'_, GpuAllocator<vk::DeviceMemory>> {
        self.inner.allocator.lock().unwrap()
    }

    /// Get a reference to the [`AshMemoryDevice`] that was created with this device.
    pub fn memory_device(&self) -> &AshMemoryDevice {
        AshMemoryDevice::wrap(self.device())
    }

    /// Allocate memory with the [`GpuAllocator`] that was created with this device.
    pub unsafe fn alloc(
        &self,
        request: Request,
    ) -> Result<MemoryBlock<vk::DeviceMemory>, AshError> {
        Ok(self.allocator().alloc(self.memory_device(), request)?)
    }

    /// Deallocate memory with the [`GpuAllocator`] that was created with this device.
    pub unsafe fn dealloc(&self, block: MemoryBlock<vk::DeviceMemory>) {
        self.allocator().dealloc(self.memory_device(), block);
    }
}

impl Deref for AshDevice {
    type Target = ash::Device;

    fn deref(&self) -> &Self::Target {
        &self.inner.device
    }
}
