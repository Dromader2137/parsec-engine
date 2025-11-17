use crate::{
    ecs::world::component::Component,
    graphics::renderer::{DefaultVertex, mesh_data::create_mesh_data},
    resources::ResourceCollection,
};

#[derive(Debug, Component)]
pub struct MeshRenderer {
    pub vertices: [DefaultVertex; 4],
    pub indices: [u32; 6],
    pub data_id: u32,
}

impl MeshRenderer {
    pub fn new(
        resources: &ResourceCollection,
        vertices: [DefaultVertex; 4],
        indices: [u32; 6],
    ) -> MeshRenderer {
        let data_id = create_mesh_data(resources, vertices.to_vec(), indices.to_vec()).unwrap();
        MeshRenderer {
            vertices,
            indices,
            data_id,
        }
    }
}
