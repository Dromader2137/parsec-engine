#[derive(Debug)]
pub struct MeshAndMaterial {
    pub mesh: u32,
    pub material: u32,
    pub camera: u32,
    pub camera_transform: u32,
    pub transform: u32,
}

#[derive(Debug)]
pub enum Draw {
    MeshAndMaterial(MeshAndMaterial),
}
