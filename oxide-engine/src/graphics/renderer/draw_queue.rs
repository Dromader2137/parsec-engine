pub struct MeshAndMaterial {
    pub mesh_id: u32,
    pub material_id: u32
}

pub enum Draw {
    MeshAndMaterial(MeshAndMaterial)
}
