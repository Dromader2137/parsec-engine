use std::sync::Arc;

use crate::{
    graphics::{
        renderer::{DefaultVertex, camera_data::CameraData, transform_data::TransformData},
        vulkan::{
            VulkanError,
            command_buffer::CommandBuffer,
            descriptor_set::{DescriptorSet, DescriptorSetBinding, DescriptorSetLayout},
            device::Device,
            framebuffer::Framebuffer,
            graphics_pipeline::GraphicsPipeline,
            shader::ShaderModule,
        },
    },
    resources::{Rsc, RscMut},
    utils::id_vec::IdVec,
};

pub struct MaterialBase {
    pipeline: Arc<GraphicsPipeline>,
    #[allow(unused)]
    descriptor_set_layouts: Vec<Arc<DescriptorSetLayout>>,
}

pub struct MaterialData {
    base: Arc<MaterialBase>,
    descriptor_sets: Vec<MaterialDescriptorSets>,
}

pub enum MaterialDescriptorSets {
    ModelMatrixSet,
    ViewMatrixSet,
    ProjectionMatrixSet,
    UniformSet(u32),
}

impl MaterialData {
    pub fn bind(
        &self,
        command_buffer: Arc<CommandBuffer>,
        camera_id: u32,
        camera_transform_id: u32,
        transform_id: u32,
    ) {
        let descriptor_sets = Rsc::<IdVec<Arc<DescriptorSet>>>::get().unwrap();
        let transforms = Rsc::<IdVec<TransformData>>::get().unwrap();
        let cameras = Rsc::<IdVec<CameraData>>::get().unwrap();
        command_buffer.bind_graphics_pipeline(self.base.pipeline.clone());
        for (set_index, set) in self.descriptor_sets.iter().enumerate() {
            let descriptor_set_id = match set {
                MaterialDescriptorSets::ViewMatrixSet => {
                    let transform = transforms.get(camera_transform_id).unwrap();
                    transform.look_at_set_id
                },
                MaterialDescriptorSets::ProjectionMatrixSet => {
                    let camera = cameras.get(camera_id).unwrap();
                    camera.projection_set_id
                },
                MaterialDescriptorSets::ModelMatrixSet => {
                    let transform = transforms.get(transform_id).unwrap();
                    transform.model_set_id
                },
                MaterialDescriptorSets::UniformSet(set_id) => *set_id,
            };
            let descriptor_set = descriptor_sets.get(descriptor_set_id).unwrap();
            command_buffer.bind_descriptor_set(
                descriptor_set.clone(),
                self.base.pipeline.clone(),
                set_index as u32,
            );
        }
    }
}

pub fn create_material_base(
    vertex_id: u32,
    fragment_id: u32,
    layout: Vec<Vec<DescriptorSetBinding>>,
) -> Result<u32, VulkanError> {
    let material_base = {
        let device = Rsc::<Arc<Device>>::get().unwrap();
        let framebuffers = Rsc::<Vec<Arc<Framebuffer>>>::get().unwrap();
        let shader_modules = Rsc::<IdVec<Arc<ShaderModule>>>::get().unwrap();
        let mut descriptor_set_layouts = Vec::new();

        for bindings in layout {
            descriptor_set_layouts.push(DescriptorSetLayout::new(device.clone(), bindings)?);
        }

        let pipeline = GraphicsPipeline::new::<DefaultVertex>(
            framebuffers[0].clone(),
            shader_modules.get(vertex_id).unwrap().clone(),
            shader_modules.get(fragment_id).unwrap().clone(),
            descriptor_set_layouts.clone(),
        )?;

        Arc::new(MaterialBase {
            pipeline,
            descriptor_set_layouts,
        })
    };

    let mut material_bases = RscMut::<IdVec<Arc<MaterialBase>>>::get().unwrap();
    Ok(material_bases.push(material_base))
}

pub fn create_material(
    material_base_id: u32,
    material_descriptor_sets: Vec<MaterialDescriptorSets>,
) -> Result<u32, VulkanError> {
    let material_bases = Rsc::<IdVec<Arc<MaterialBase>>>::get().unwrap();
    let base = material_bases.get(material_base_id).unwrap().clone();
    let material_data = MaterialData {
        base,
        descriptor_sets: material_descriptor_sets,
    };

    let mut materials = RscMut::<IdVec<MaterialData>>::get().unwrap();
    Ok(materials.push(material_data))
}
