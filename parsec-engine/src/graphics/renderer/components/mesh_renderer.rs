use crate::ecs::world::component::Component;

#[derive(Debug, Component)]
pub struct MeshRenderer {
    pub mesh_id: u32,
    pub material_id: u32,
}

impl MeshRenderer {
    pub fn new(mesh_id: u32, material_id: u32) -> MeshRenderer {
        MeshRenderer {
            mesh_id,
            material_id,
        }
    }
}
