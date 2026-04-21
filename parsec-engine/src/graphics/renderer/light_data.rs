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
    math::{
        mat::Matrix4f,
        vec::{Vec2f, Vec3f, Vec4f},
    },
};

pub const MAX_LIGHT_COUNT: usize = 32;

#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct DirectionalLightData {
    world_to_light: Matrix4f,
    direction: Vec4f,
    color: Vec4f,
}

#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct RendererLightsData {
    light_count: u32,
    _pad: [u32; 3],
    data: [DirectionalLightData; MAX_LIGHT_COUNT],
}

pub struct RendererLights {
    data: RendererLightsData,
    data_buffer: Buffer,
    data_changed: bool,
    resource_layout: PipelineResourceLayout,
    pub resource: PipelineResource,
}

impl RendererLights {
    pub fn new(backend: &mut ActiveGraphicsBackend) -> RendererLights {
        let data = RendererLightsData {
            light_count: 0,
            _pad: [0; 3],
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
        resource
            .bind_buffer(backend, data_buffer.handle(), 0)
            .unwrap();

        RendererLights {
            data,
            data_buffer,
            data_changed: false,
            resource_layout,
            resource,
        }
    }

    pub fn add_light_data(
        &mut self,
        light_pos: Vec3f,
        light_dir: Vec3f,
        light_up: Vec3f,
        shadow_texture_from: Vec2f,
        shadow_texture_to: Vec2f,
    ) {
        let world_to_light =
            Matrix4f::subimage(shadow_texture_from, shadow_texture_to)
                * Matrix4f::orthographic(0.0, 100.0, 25.0, 25.0)
                * Matrix4f::look_at(light_pos, light_dir, light_up);

        let light_idx = self.data.light_count as usize;
        self.data.light_count += 1;
        self.data.data[light_idx].world_to_light = world_to_light;
        self.data.data[light_idx].direction =
            Vec4f::new(light_dir.x, light_dir.y, light_dir.z, 0.0);
        self.data.data[light_idx].color = Vec4f::ONE;
        self.data_changed = true;
    }

    pub fn update_buffer(&self, backend: &mut ActiveGraphicsBackend) {
        backend
            .update_buffer(
                self.data_buffer.handle(),
                BufferContent::from_slice(&[self.data]),
            )
            .unwrap();
    }
}
