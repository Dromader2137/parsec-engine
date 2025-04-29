use super::context::VulkanError;

pub struct GraphicsPipeline {
    graphics_pipeline: ash::vk::Pipeline
}

#[derive(Debug)]
pub enum GraphicsPipelineError {
}

impl From<GraphicsPipelineError> for VulkanError {
    fn from(value: GraphicsPipelineError) -> Self {
        VulkanError::GrphicsPipelineError(value)
    }
}

impl GraphicsPipeline {
    pub fn get_pipeline_raw(&self) -> &ash::vk::Pipeline {
        &self.graphics_pipeline
    }
}
