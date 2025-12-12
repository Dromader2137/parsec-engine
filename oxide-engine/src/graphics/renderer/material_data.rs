use std::sync::atomic::{AtomicU32, Ordering};

use crate::graphics::{
    renderer::{
        DefaultVertex, camera_data::CameraData, transform_data::TransformData,
    },
    vulkan::{
        VulkanError,
        command_buffer::VulkanCommandBuffer,
        descriptor_set::{
            VulkanDescriptorSetBinding, VulkanDescriptorSetLayout,
        },
        device::VulkanDevice,
        framebuffer::VulkanFramebuffer,
        graphics_pipeline::VulkanGraphicsPipeline,
        renderpass::VulkanRenderpass,
        shader::VulkanShaderModule,
    },
};

pub struct MaterialBase {
    id: u32,
    pipeline: VulkanGraphicsPipeline,
    #[allow(unused)]
    descriptor_set_layouts: Vec<VulkanDescriptorSetLayout>,
}

impl MaterialBase {
    const ID_COUNTER: AtomicU32 = AtomicU32::new(0);

    pub fn new(
        device: &VulkanDevice,
        renderpass: &VulkanRenderpass,
        framebuffers: Vec<&VulkanFramebuffer>,
        vertex_shader: &VulkanShaderModule,
        fragment_shader: &VulkanShaderModule,
        layout: Vec<Vec<VulkanDescriptorSetBinding>>,
    ) -> Result<MaterialBase, VulkanError> {
        let mut descriptor_set_layouts = Vec::new();

        for bindings in layout {
            descriptor_set_layouts
                .push(VulkanDescriptorSetLayout::new(&device, bindings)?);
        }

        let pipeline = VulkanGraphicsPipeline::new::<DefaultVertex>(
            device,
            renderpass,
            &framebuffers[0],
            vertex_shader,
            fragment_shader,
            &descriptor_set_layouts,
        )?;

        let id = Self::ID_COUNTER.load(Ordering::Acquire);
        Self::ID_COUNTER.store(id + 1, Ordering::Release);

        Ok(MaterialBase {
            id,
            pipeline,
            descriptor_set_layouts,
        })
    }

    pub fn id(&self) -> u32 { self.id }
}

pub enum MaterialDescriptorSets {
    ModelMatrixSet,
    ViewMatrixSet,
    ProjectionMatrixSet,
    UniformSet(u32),
}

pub struct MaterialData {
    material_base_id: u32,
    descriptor_sets: Vec<MaterialDescriptorSets>,
}

impl MaterialData {
    pub fn new(
        material_base: &MaterialBase,
        material_descriptor_sets: Vec<MaterialDescriptorSets>,
    ) -> Result<MaterialData, VulkanError> {
        Ok(MaterialData {
            material_base_id: material_base.id(),
            descriptor_sets: material_descriptor_sets,
        })
    }

    pub fn bind(
        &self,
        device: &VulkanDevice,
        material_base: &MaterialBase,
        command_buffer: &VulkanCommandBuffer,
        camera: &CameraData,
        camera_transform: &TransformData,
        transform: &TransformData,
    ) {
        command_buffer
            .bind_graphics_pipeline(device, &material_base.pipeline)
            .unwrap();
        for (set_index, set) in self.descriptor_sets.iter().enumerate() {
            let descriptor_set = match set {
                MaterialDescriptorSets::ViewMatrixSet => {
                    &camera_transform.look_at_set
                },
                MaterialDescriptorSets::ProjectionMatrixSet => {
                    &camera.projection_set
                },
                MaterialDescriptorSets::ModelMatrixSet => &transform.model_set,
                _ => unreachable!(),
            };
            command_buffer
                .bind_descriptor_set(
                    device,
                    descriptor_set,
                    &material_base.pipeline,
                    set_index as u32,
                )
                .unwrap();
        }
    }

    pub fn material_base_id(&self) -> u32 { self.material_base_id }
}
