use std::fmt::Display;

use ash::vk;

#[derive(Debug)]
pub enum AshError {
    #[cfg(not(target_os = "macos"))]
    LoadError(ash::LoadingError),
    #[cfg(target_os = "macos")]
    LoadError(ash_molten::LoadingError),
    VkResult(vk::Result),
    AllocationError(gpu_alloc::AllocationError),
    DeviceNotFound,
}

#[cfg(not(target_os = "macos"))]
impl From<ash::LoadingError> for AshError {
    fn from(e: ash::LoadingError) -> Self {
        Self::LoadError(e)
    }
}

#[cfg(target_os = "macos")]
impl From<ash_molten::LoadingError> for AshError {
    fn from(e: ash_molten::LoadingError) -> Self {
        Self::LoadError(e)
    }
}

impl From<vk::Result> for AshError {
    fn from(e: vk::Result) -> Self {
        Self::VkResult(e)
    }
}

impl From<gpu_alloc::AllocationError> for AshError {
    fn from(e: gpu_alloc::AllocationError) -> Self {
        Self::AllocationError(e)
    }
}

impl Display for AshError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(not(target_os = "macos"))]
            Self::LoadError(e) => write!(f, "Failed to load ash: {}", e),
            #[cfg(target_os = "macos")]
            Self::LoadError(e) => write!(f, "Failed to load ash: {}", e),
            Self::VkResult(e) => write!(f, "VkResult: {}", e),
            Self::AllocationError(e) => write!(f, "Failed to allocate memory: {}", e),
            Self::DeviceNotFound => write!(f, "Failed to find device"),
        }
    }
}
