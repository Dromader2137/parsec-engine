use crate::{
    graphics::{
        backend::GraphicsBackend,
        command_list::CommandList,
        pipeline::{
            Pipeline, PipelineBinding, PipelineBindingLayout, PipelineOptions, PipelineSubbindingLayout
        },
        renderer::{camera_data::CameraData, transform_data::TransformData},
        renderpass::Renderpass,
        shader::Shader,
    },
    utils::{identifiable::Identifiable, IdType},
};

pub struct MaterialBase {
    material_base_id: IdType,
    pipeline: Pipeline,
    #[allow(unused)]
    binding_layouts: Vec<PipelineBindingLayout>,
}

crate::create_counter!{ID_COUNTER}
impl MaterialBase {
    pub fn new(
        backend: &mut impl GraphicsBackend,
        vertex_shader: Shader,
        fragment_shader: Shader,
        renderpass: Renderpass,
        binding_layouts: Vec<Vec<PipelineSubbindingLayout>>,
        pipeline_options: PipelineOptions
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
                &binding_layouts,
                pipeline_options
            )
            .unwrap();

        MaterialBase {
            material_base_id: ID_COUNTER.next(),
            pipeline,
            binding_layouts,
        }
    }

    pub fn id(&self) -> u32 { self.material_base_id }
}

impl Identifiable for MaterialBase {
    fn id(&self) -> IdType {
        self.id()
    }
}

pub enum MaterialPipelineBinding {
    Model,
    View,
    Projection,
    ShadowMap,
    Light,
    Generic(PipelineBinding),
}

pub struct MaterialData {
    material_id: IdType,
    material_base_id: IdType,
    descriptor_sets: Vec<MaterialPipelineBinding>,
}

crate::create_counter!{ID_COUNTER_MAT}
impl MaterialData {
    pub fn new(
        material_base: &MaterialBase,
        material_descriptor_sets: Vec<MaterialPipelineBinding>,
    ) -> MaterialData {
        MaterialData {
            material_id: ID_COUNTER_MAT.next(),
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
        light_binding: PipelineBinding,
        shadowmap_binding: PipelineBinding
    ) {
        backend
            .command_bind_pipeline(command_list, material_base.pipeline)
            .unwrap();
        for (set_index, binding) in self.descriptor_sets.iter().enumerate() {
            let pipeline_binding = match binding {
                MaterialPipelineBinding::View => {
                    camera_transform.look_at_binding
                },
                MaterialPipelineBinding::Projection => {
                    camera.projection_binding
                },
                MaterialPipelineBinding::Model => transform.model_binding,
                MaterialPipelineBinding::Light => light_binding,
                MaterialPipelineBinding::ShadowMap => {
                    shadowmap_binding
                }
                MaterialPipelineBinding::Generic(bind) => *bind,
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

impl Identifiable for MaterialData {
    fn id(&self) -> IdType {
        self.material_id
    }
}
