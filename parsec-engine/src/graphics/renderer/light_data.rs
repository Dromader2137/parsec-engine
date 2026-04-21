use crate::{
    graphics::{
        ActiveGraphicsBackend,
        buffer::{Buffer, BufferBuilder, BufferContent, BufferUsage},
        pipeline::{
            PipelineBindingType, PipelineResource,
            PipelineResourceBindingLayout, PipelineResourceLayout,
            PipelineResourceLayoutBuilder, PipelineShaderStage,
        },
    },
    math::{mat::Matrix4f, vec::Vec3f},
};

pub const MAX_LIGHT_COUNT: usize = 32;

#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct DirectionalLightData {
    world_to_light: Matrix4f,
    direction: Vec3f,
    color: Vec3f,
}

#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct RendererLightsData {
    light_count: u32,
    data: [DirectionalLightData; MAX_LIGHT_COUNT],
}

pub struct RendererLights {
    data: RendererLightsData,
    data_buffer: Buffer,
    resource_layout: PipelineResourceLayout,
    resource: PipelineResource,
}

impl RendererLights {
    pub fn new(backend: &mut ActiveGraphicsBackend) -> RendererLights {
        let data = RendererLightsData {
            light_count: 0,
            data: std::array::from_fn(|_| DirectionalLightData::default()),
        };

        let data_buffer = BufferBuilder::new()
            .usage(&[BufferUsage::Storage])
            .data(BufferContent::from_slice(&[data]))
            .build(backend)
            .unwrap();

        let resource_layout = PipelineResourceLayoutBuilder::new()
            .binding(PipelineResourceBindingLayout::new(
                PipelineBindingType::StorageBuffer,
                &[PipelineShaderStage::Vertex, PipelineShaderStage::Fragment],
            ))
            .build(backend)
            .unwrap();

        let resource = resource_layout.create_resource(backend).unwrap();

        RendererLights { data, data_buffer, resource_layout, resource }
    }
}
