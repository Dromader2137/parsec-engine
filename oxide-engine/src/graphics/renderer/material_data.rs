use std::sync::Arc;

use crate::graphics::{
    renderer::{
        DefaultVertex, camera_data::CameraData, transform_data::TransformData,
    },
    vulkan::{
        VulkanError,
        command_buffer::CommandBuffer,
        descriptor_set::{DescriptorSetBinding, DescriptorSetLayout},
        framebuffer::Framebuffer,
        graphics_pipeline::GraphicsPipeline,
        shader::ShaderModule,
    },
};

pub struct MaterialBase {
    pipeline: Arc<GraphicsPipeline>,
    #[allow(unused)]
    descriptor_set_layouts: Vec<Arc<DescriptorSetLayout>>,
}

impl MaterialBase {
    pub fn new(
        framebuffers: Vec<Arc<Framebuffer>>,
        vertex_shader: Arc<ShaderModule>,
        fragment_shader: Arc<ShaderModule>,
        layout: Vec<Vec<DescriptorSetBinding>>,
    ) -> Result<Arc<MaterialBase>, VulkanError> {
        let mut descriptor_set_layouts = Vec::new();

        for bindings in layout {
            descriptor_set_layouts.push(DescriptorSetLayout::new(
                framebuffers[0].renderpass.device.clone(),
                bindings,
            )?);
        }

        let pipeline = GraphicsPipeline::new::<DefaultVertex>(
            framebuffers[0].clone(),
            vertex_shader,
            fragment_shader,
            descriptor_set_layouts.clone(),
        )?;

        Ok(Arc::new(MaterialBase {
            pipeline,
            descriptor_set_layouts,
        }))
    }
}

pub enum MaterialDescriptorSets {
    ModelMatrixSet,
    ViewMatrixSet,
    ProjectionMatrixSet,
    UniformSet(u32),
}

pub struct MaterialData {
    material_base: Arc<MaterialBase>,
    descriptor_sets: Vec<MaterialDescriptorSets>,
}

impl MaterialData {
    pub fn new(
        material_base: Arc<MaterialBase>,
        material_descriptor_sets: Vec<MaterialDescriptorSets>,
    ) -> Result<MaterialData, VulkanError> {
        Ok(MaterialData {
            material_base,
            descriptor_sets: material_descriptor_sets,
        })
    }

    pub fn bind(
        &self,
        command_buffer: Arc<CommandBuffer>,
        camera: &CameraData,
        camera_transform: &TransformData,
        transform: &TransformData,
    ) {
        command_buffer
            .bind_graphics_pipeline(self.material_base.pipeline.clone());
        for (set_index, set) in self.descriptor_sets.iter().enumerate() {
            let descriptor_set = match set {
                MaterialDescriptorSets::ViewMatrixSet => {
                    camera_transform.look_at_set.clone()
                },
                MaterialDescriptorSets::ProjectionMatrixSet => {
                    camera.projection_set.clone()
                },
                MaterialDescriptorSets::ModelMatrixSet => {
                    transform.model_set.clone()
                },
                _ => unreachable!(),
            };
            command_buffer.bind_descriptor_set(
                descriptor_set.clone(),
                self.material_base.pipeline.clone(),
                set_index as u32,
            );
        }
    }
}
