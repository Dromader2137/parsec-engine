use crate::{
    graphics::{
        ActiveGraphicsBackend,
        command_list::{Command, CommandList},
        pipeline::{
            Pipeline, PipelineBuilder, PipelineOptions,
            PipelineResourceBindingLayout, PipelineResourceHandle,
            PipelineResourceLayout, PipelineResourceLayoutBuilder,
        },
        renderer::{
            camera_data::CameraData, light_data::RendererLights,
            shadow::RendererShadows, transform_data::TransformData,
        },
        renderpass::RenderpassHandle,
        shader::ShaderHandle,
    },
    utils::{IdType, identifiable::Identifiable},
};

pub struct MaterialBase {
    material_base_id: IdType,
    pipeline: Pipeline,
    resource_layouts: Vec<PipelineResourceLayout>,
}

crate::create_counter! {ID_COUNTER}
impl MaterialBase {
    pub fn new(
        backend: &mut ActiveGraphicsBackend,
        vertex_shader: ShaderHandle,
        fragment_shader: ShaderHandle,
        renderpass: RenderpassHandle,
        binding_layouts: Vec<Vec<PipelineResourceBindingLayout>>,
        pipeline_options: PipelineOptions,
    ) -> MaterialBase {
        let resource_layouts = binding_layouts
            .iter()
            .map(|binding_layout| {
                PipelineResourceLayoutBuilder::new()
                    .bindings(binding_layout)
                    .build(backend)
                    .unwrap()
            })
            .collect::<Vec<PipelineResourceLayout>>();

        let layout_handles: Vec<_> =
            resource_layouts.iter().map(|l| l.handle()).collect();
        let pipeline = PipelineBuilder::new()
            .renderpass(renderpass)
            .vertex_shader(vertex_shader)
            .fragment_shader(fragment_shader)
            .resource_layouts(&layout_handles)
            .cull_mode(pipeline_options.culling_mode)
            .build(backend)
            .unwrap();

        MaterialBase {
            material_base_id: ID_COUNTER.next(),
            pipeline,
            resource_layouts,
        }
    }

    pub fn id(&self) -> u32 { self.material_base_id }

    pub fn resource_layouts(&self) -> &[PipelineResourceLayout] {
        &self.resource_layouts
    }
}

impl Identifiable for MaterialBase {
    fn id(&self) -> IdType { self.id() }
}

pub enum MaterialPipelineBinding {
    Model,
    View,
    Projection,
    ShadowMap,
    Light,
    Generic(PipelineResourceHandle),
}

pub struct MaterialData {
    material_id: IdType,
    material_base_id: IdType,
    resources: Vec<MaterialPipelineBinding>,
}

crate::create_counter! {ID_COUNTER_MAT}
impl MaterialData {
    pub fn new(
        material_base: &MaterialBase,
        material_resrouces: Vec<MaterialPipelineBinding>,
    ) -> MaterialData {
        MaterialData {
            material_id: ID_COUNTER_MAT.next(),
            material_base_id: material_base.id(),
            resources: material_resrouces,
        }
    }

    pub fn bind(
        &self,
        command_list: &mut CommandList,
        material_base: &MaterialBase,
        camera: &CameraData,
        camera_transform: &TransformData,
        transform: &TransformData,
        lights: &RendererLights,
        shadows: &RendererShadows,
    ) {
        command_list.cmd(Command::BindGraphicsPipeline(
            material_base.pipeline.handle(),
        ));
        for (set_index, binding) in self.resources.iter().enumerate() {
            let pipeline_binding = match binding {
                MaterialPipelineBinding::View => {
                    camera_transform.look_at_resource.handle()
                },
                MaterialPipelineBinding::Projection => {
                    camera.projection_resource.handle()
                },
                MaterialPipelineBinding::Model => {
                    transform.model_resource.handle()
                },
                MaterialPipelineBinding::Light => lights.resource.handle(),
                MaterialPipelineBinding::ShadowMap => {
                    shadows.texture_resource.handle()
                },
                MaterialPipelineBinding::Generic(bind) => *bind,
            };
            command_list.cmd(Command::BindPipelineBinding(
                pipeline_binding,
                set_index as u32,
            ));
        }
    }

    pub fn material_base_id(&self) -> u32 { self.material_base_id }
}

impl Identifiable for MaterialData {
    fn id(&self) -> IdType { self.material_id }
}
