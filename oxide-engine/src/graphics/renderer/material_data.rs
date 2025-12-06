use std::sync::atomic::{AtomicU32, Ordering};

use crate::graphics::{
    renderer::{
        DefaultVertex, camera_data::CameraData, transform_data::TransformData,
    },
    vulkan::{
        VulkanError,
        command_buffer::CommandBuffer,
        descriptor_set::{DescriptorSetBinding, DescriptorSetLayout},
        device::Device,
        framebuffer::Framebuffer,
        graphics_pipeline::GraphicsPipeline,
        renderpass::Renderpass,
        shader::ShaderModule,
    },
};

pub struct MaterialBase {
    id: u32,
    pipeline: GraphicsPipeline,
    #[allow(unused)]
    descriptor_set_layouts: Vec<DescriptorSetLayout>,
}

impl MaterialBase {
    const ID_COUNTER: AtomicU32 = AtomicU32::new(0);

    pub fn new(
        device: &Device,
        renderpass: &Renderpass,
        framebuffers: Vec<&Framebuffer>,
        vertex_shader: &ShaderModule,
        fragment_shader: &ShaderModule,
        layout: Vec<Vec<DescriptorSetBinding>>,
    ) -> Result<MaterialBase, VulkanError> {
        let mut descriptor_set_layouts = Vec::new();

        for bindings in layout {
            descriptor_set_layouts
                .push(DescriptorSetLayout::new(&device, bindings)?);
        }

        let pipeline = GraphicsPipeline::new::<DefaultVertex>(
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
        device: &Device,
        material_base: &MaterialBase,
        command_buffer: &CommandBuffer,
        camera: &CameraData,
        camera_transform: &TransformData,
        transform: &TransformData,
    ) {
        command_buffer.bind_graphics_pipeline(device, &material_base.pipeline);
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
            command_buffer.bind_descriptor_set(
                device,
                descriptor_set,
                &material_base.pipeline,
                set_index as u32,
            );
        }
    }

    pub fn material_base_id(&self) -> u32 { self.material_base_id }
}
