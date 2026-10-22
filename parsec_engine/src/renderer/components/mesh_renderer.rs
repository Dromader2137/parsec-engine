use crate::{assets::{core::mesh::Mesh, library::AssetHandle}, ecs::world::component::Component};

#[derive(Debug, Component)]
pub struct MeshRenderer {
    pub mesh: AssetHandle<Mesh>,
    pub material_id: u32,
}

impl MeshRenderer {
    pub fn new(mesh: AssetHandle<Mesh>, material_id: u32) -> MeshRenderer {
        MeshRenderer {
            mesh,
            material_id,
        }
    }
}
