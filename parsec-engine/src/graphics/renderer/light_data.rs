use crate::{
    ecs::{system::system, world::query::Query},
    graphics::{
        ActiveGraphicsBackend,
        buffer::{Buffer, BufferBuilder, BufferContent, BufferUsage},
        pipeline::{
            PipelineBindingType, PipelineResource,
            PipelineResourceBindingLayout, PipelineResourceLayout,
            PipelineResourceLayoutBuilder, PipelineShaderStage,
        },
        renderer::components::{light::Light, transform::Transform},
    },
    math::{
        mat::Matrix4f,
        vec::{Vec2f, Vec3f, Vec4f},
    },
    resources::Resource,
};

pub const MAX_LIGHT_COUNT: usize = 32;

#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct DirectionalLightData {
    world_to_light: Matrix4f,
    atlas_clip: Vec4f,
    direction: Vec4f,
    color: Vec4f,
}

#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct RendererLightsData {
    pub light_count: u32,
    _pad: [u32; 3],
    data: [DirectionalLightData; MAX_LIGHT_COUNT],
}

pub struct RendererLights {
    pub data: RendererLightsData,
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
        light_color: Vec3f,
        light_size: f32,
    ) {
        let light_idx = self.data.light_count as usize;
        let y = light_idx / 8;
        let x = light_idx % 8;
        let shadow_texture_from =
            Vec2f::new(x as f32, y as f32) / Vec2f::new(8.0, 4.0);
        let shadow_texture_to =
            Vec2f::new((x + 1) as f32, (y + 1) as f32) / Vec2f::new(8.0, 4.0);
        let world_to_light =
            Matrix4f::orthographic(0.0, 100.0, light_size, light_size)
                * Matrix4f::look_at(light_pos, light_dir, light_up);

        self.data.light_count += 1;
        self.data.data[light_idx].world_to_light = world_to_light;
        self.data.data[light_idx].direction =
            Vec4f::new(light_dir.x, light_dir.y, light_dir.z, 0.0);
        self.data.data[light_idx].color = light_color.into();
        self.data.data[light_idx].atlas_clip = Vec4f::new(
            shadow_texture_from.x,
            shadow_texture_from.y,
            shadow_texture_to.x,
            shadow_texture_to.y,
        );
        self.data_changed = true;
    }

    pub fn clear_data(&mut self) { self.data.light_count = 0; }

    pub fn update_buffer(&self, backend: &mut ActiveGraphicsBackend) {
        backend
            .update_buffer(
                self.data_buffer.handle(),
                BufferContent::from_slice(&[self.data]),
            )
            .unwrap();
    }

    pub fn destroy(self, backend: &mut ActiveGraphicsBackend) {
        self.resource.destroy(backend).unwrap();
        self.resource_layout.destroy(backend).unwrap();
        self.data_buffer.destroy(backend).unwrap();
    }
}

#[system]
fn update_light_data(
    mut backend: Resource<ActiveGraphicsBackend>,
    mut light_data: Resource<RendererLights>,
    mut lights: Query<(Light, Transform)>,
) {
    light_data.clear_data();
    for (_, (light, transfrom)) in lights.iter() {
        light_data.add_light_data(
            transfrom.position,
            light.direction,
            light.up,
            light.color,
            20.0,
        );
    }
    light_data.update_buffer(&mut backend);
}
