#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(unused)]
pub enum VulkanPipelineStage {
    TopOfPipe,
    VertexInput,
    VertexShader,
    EarlyFragmentTests,
    FragmentShader,
    LateFragmentTests,
    BottomOfPipe,
    ColorAttachmentOutput,
    Transfer,
    Host,
}

impl VulkanPipelineStage {
    pub fn raw_pipeline_stage(&self) -> ash::vk::PipelineStageFlags {
        match self {
            VulkanPipelineStage::TopOfPipe => {
                ash::vk::PipelineStageFlags::TOP_OF_PIPE
            },
            VulkanPipelineStage::VertexInput => {
                ash::vk::PipelineStageFlags::VERTEX_INPUT
            },
            VulkanPipelineStage::VertexShader => {
                ash::vk::PipelineStageFlags::VERTEX_SHADER
            },
            VulkanPipelineStage::EarlyFragmentTests => {
                ash::vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS
            },
            VulkanPipelineStage::FragmentShader => {
                ash::vk::PipelineStageFlags::FRAGMENT_SHADER
            },
            VulkanPipelineStage::LateFragmentTests => {
                ash::vk::PipelineStageFlags::LATE_FRAGMENT_TESTS
            },
            VulkanPipelineStage::BottomOfPipe => {
                ash::vk::PipelineStageFlags::BOTTOM_OF_PIPE
            },
            VulkanPipelineStage::ColorAttachmentOutput => {
                ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
            },
            VulkanPipelineStage::Transfer => {
                ash::vk::PipelineStageFlags::TRANSFER
            },
            VulkanPipelineStage::Host => ash::vk::PipelineStageFlags::HOST,
        }
    }

    pub fn raw_combined_stage(stages: &[Self]) -> ash::vk::PipelineStageFlags {
        stages
            .iter()
            .fold(ash::vk::PipelineStageFlags::empty(), |acc, x| {
                acc | x.raw_pipeline_stage()
            })
    }
}
