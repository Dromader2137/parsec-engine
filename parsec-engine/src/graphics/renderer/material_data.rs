use crate::{
    graphics::{
        backend::GraphicsBackend,
        command_list::CommandList,
        pipeline::{
            Pipeline, PipelineBinding, PipelineBindingLayout,
            PipelineSubbindingLayout,
        },
        renderer::{camera_data::CameraData, transform_data::TransformData},
        renderpass::Renderpass,
        shader::Shader,
    },
    utils::id_counter::IdCounter,
};

pub struct MaterialBase {
    id: u32,
    pipeline: Pipeline,
    #[allow(unused)]
    binding_layouts: Vec<PipelineBindingLayout>,
}

static ID_COUNTER: once_cell::sync::Lazy<IdCounter> =
    once_cell::sync::Lazy::new(|| IdCounter::new(0));
impl MaterialBase {
    pub fn new(
        backend: &mut impl GraphicsBackend,
        vertex_shader: Shader,
        fragment_shader: Shader,
        renderpass: Renderpass,
        binding_layouts: Vec<Vec<PipelineSubbindingLayout>>,
    ) -> MaterialBase {
        let binding_layouts = binding_layouts
            .iter()
            .map(|binding_layout| {
                backend
                    .create_pipeline_binding_layout(&binding_layout)
                    .unwrap()
            })
            .collect::<Vec<_>>();

        let pipeline = backend
            .create_pipeline(
                vertex_shader,
                fragment_shader,
                renderpass,
                None,
                &binding_layouts,
            )
            .unwrap();

        MaterialBase {
            id: ID_COUNTER.next(),
            pipeline,
            binding_layouts,
        }
    }

    pub fn id(&self) -> u32 { self.id }
}

pub enum MaterialPipelineBinding {
    ModelMatrix,
    ViewMatrix,
    InverseViewMatrix,
    ProjectionMatrix,
    InverseProjectionMatrix,
    Generic(PipelineBinding),
}

pub struct MaterialData {
    material_base_id: u32,
    descriptor_sets: Vec<MaterialPipelineBinding>,
}

impl MaterialData {
    pub fn new(
        material_base: &MaterialBase,
        material_descriptor_sets: Vec<MaterialPipelineBinding>,
    ) -> MaterialData {
        MaterialData {
            material_base_id: material_base.id(),
            descriptor_sets: material_descriptor_sets,
        }
    }

    pub fn bind(
        &self,
        backend: &mut impl GraphicsBackend,
        command_list: CommandList,
        material_base: &MaterialBase,
        camera: &CameraData,
        camera_transform: &TransformData,
        transform: &TransformData,
    ) {
        backend
            .command_bind_pipeline(command_list, material_base.pipeline)
            .unwrap();
        for (set_index, binding) in self.descriptor_sets.iter().enumerate() {
            let pipeline_binding = match binding {
                MaterialPipelineBinding::ViewMatrix => {
                    camera_transform.look_at_binding
                },
                MaterialPipelineBinding::ProjectionMatrix => {
                    camera.projection_binding
                },
                MaterialPipelineBinding::ModelMatrix => transform.model_binding,
                MaterialPipelineBinding::Generic(bind) => *bind,
                _ => todo!()
            };
            backend
                .command_bind_pipeline_binding(
                    command_list,
                    material_base.pipeline,
                    pipeline_binding,
                    set_index as u32,
                )
                .unwrap();
        }
    }

    pub fn material_base_id(&self) -> u32 { self.material_base_id }
}
