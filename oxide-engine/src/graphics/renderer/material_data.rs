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
    resources::ResourceCollection,
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
        resources: &ResourceCollection,
        command_buffer: Arc<CommandBuffer>,
        camera_id: u32,
        camera_transform_id: u32,
        transform_id: u32,
    ) {
        let descriptor_sets = resources.get::<IdVec<Arc<DescriptorSet>>>().unwrap();
        let transforms = resources.get::<IdVec<TransformData>>().unwrap();
        let cameras = resources.get::<IdVec<CameraData>>().unwrap();
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
    resources: &mut ResourceCollection,
    vertex_id: u32,
    fragment_id: u32,
    layout: Vec<Vec<DescriptorSetBinding>>,
) -> Result<u32, VulkanError> {
    let material_base = {
        let device = resources.get::<Arc<Device>>().unwrap();
        let framebuffers = resources.get::<Vec<Arc<Framebuffer>>>().unwrap();
        let shader_modules = resources.get::<IdVec<Arc<ShaderModule>>>().unwrap();
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

    let mut material_bases = resources.get_mut::<IdVec<Arc<MaterialBase>>>().unwrap();
    Ok(material_bases.push(material_base))
}

pub fn create_material(
    resources: &mut ResourceCollection,
    material_base_id: u32,
    material_descriptor_sets: Vec<MaterialDescriptorSets>,
) -> Result<u32, VulkanError> {
    let material_bases = resources.get::<IdVec<Arc<MaterialBase>>>().unwrap();
    let base = material_bases.get(material_base_id).unwrap().clone();
    let material_data = MaterialData {
        base,
        descriptor_sets: material_descriptor_sets,
    };

    let mut materials = resources.get_mut::<IdVec<MaterialData>>().unwrap();
    Ok(materials.push(material_data))
}
