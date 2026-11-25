use crate::{
    assets::{core::mesh::Mesh, library::AssetHandle}, ecs::world::component::Component, renderer::material_data::MaterialData, utils::{IdType, identifiable::Identifiable},
};

#[derive(Debug, Component)]
pub struct MeshRenderer {
    pub mesh: AssetHandle<Mesh>,
    pub material_id: IdType,
}

impl MeshRenderer {
    pub fn new(mesh: AssetHandle<Mesh>, material: &MaterialData) -> MeshRenderer {
        MeshRenderer { mesh, material_id: material.id() }
    }
}
