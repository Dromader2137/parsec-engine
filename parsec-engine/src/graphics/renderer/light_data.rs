use crate::{
    graphics::{
        buffer::Buffer,
        pipeline::{PipelineResource, PipelineResourceLayout},
        renderer::texture_atlas::TextureAtlasRegion,
    },
    math::vec::Vec3f,
};

pub const LIGHT_COUNT: usize = 32;

pub struct RendererLights {
    
}

pub struct RendererLightsData {
    lights: Vec<DirectionalLightData>,
    direction_storage_buffer: Buffer,
    direction_resource_layout: PipelineResourceLayout,
    direction_resource: PipelineResource,
}

pub struct DirectionalLightData {
    shadowmap_region: TextureAtlasRegion,
    direction: Vec3f,
    direction_buffer: Buffer,
    direction_layout: PipelineResourceLayout,
    direction_resource: PipelineResource,
}
