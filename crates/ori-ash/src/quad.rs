use ash::vk;

use crate::{AshDevice, AshError};

pub struct QuadPipeline {
    pub layout: vk::PipelineLayout,
    pub pipeline: vk::Pipeline,
    pub device: AshDevice,
}

impl QuadPipeline {
    pub unsafe fn new(device: &AshDevice) -> Result<Self, AshError> {
        todo!()
    }
}

impl Drop for QuadPipeline {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_pipeline(self.pipeline, None);
            self.device.destroy_pipeline_layout(self.layout, None);
        }
    }
}
