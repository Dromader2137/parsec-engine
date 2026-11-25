use parsec_engine_math::vec::{Vec2f, Vec3f};

use crate::{
    assets::Asset,
    create_counter,
    ecs::resources::Resources,
    graphics::{ActiveGraphicsBackend, buffer::BufferError, pipeline::DefaultVertex},
    renderer::mesh_data::MeshData,
    utils::{
        IdType,
        identifiable::{IdStore, Identifiable},
    },
};

pub mod obj;

#[derive(Debug, Default)]
pub struct InputMesh {
    pub positions: Vec<Vec3f>,
    pub normals: Vec<Vec3f>,
    pub uvs: Vec<Vec2f>,
    pub indices: Vec<u32>,
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct CookedMesh {
    positions: Vec<Vec3f>,
    normals: Vec<Vec3f>,
    uvs: Vec<Vec2f>,
    indices: Vec<u32>,
}

impl CookedMesh {
    pub fn new(
        positions: Vec<Vec3f>,
        normals: Vec<Vec3f>,
        uvs: Vec<Vec2f>,
        indices: Vec<u32>,
    ) -> Self {
        Self { positions, normals, uvs, indices }
    }

    pub fn from_input(input_mesh: InputMesh) -> Self {
        Self::new(input_mesh.positions, input_mesh.normals, input_mesh.uvs, input_mesh.indices)
    }
}

#[derive(Debug)]
pub struct Mesh {
    mesh_id: IdType,
    pub vertices: Vec<DefaultVertex>,
    pub indices: Vec<u32>,
    pub data_id: Option<u32>,
}

create_counter! {ID_COUNTER}
impl Mesh {
    pub fn new(vertices: Vec<DefaultVertex>, indices: Vec<u32>) -> Mesh {
        Mesh { mesh_id: ID_COUNTER.next(), vertices, indices, data_id: None }
    }

    fn from_cooked(cooked: CookedMesh) -> Self {
        let vertices = cooked
            .positions
            .iter()
            .zip(cooked.uvs.iter())
            .zip(cooked.normals.iter())
            .map(|((pos, uv), norm)| DefaultVertex::new(*pos, *norm, *uv))
            .collect();
        Self::new(vertices, cooked.indices)
    }
}

impl Identifiable for Mesh {
    //piwo
    fn id(&self) -> IdType { self.mesh_id }
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum MeshCookError {}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum MeshLoadError {
    #[error("graphics backend not found in resources")]
    GraphicsBackendNotFound,
    #[error(transparent)]
    Buffer(#[from] BufferError),
}

impl Asset for Mesh {
    type Input = InputMesh;
    type Cooked = CookedMesh;
    type CookError = MeshCookError;
    type LoadError = MeshLoadError;

    const ASSET_TYPE: &'static str = "mesh";

    fn cook(input_data: Self::Input) -> Result<Self::Cooked, Self::CookError> {
        Ok(CookedMesh::from_input(input_data))
    }

    fn load(cooked: Self::Cooked, resources: &mut Resources) -> Result<Self, Self::LoadError> {
        let mut mesh = Mesh::from_cooked(cooked);
        let mesh_data = {
            let mut backend = resources
                .get_mut::<ActiveGraphicsBackend>()
                .ok_or(MeshLoadError::GraphicsBackendNotFound)?;
            MeshData::new(&mut backend, &mesh.vertices, &mesh.indices)?
        };
        let mut mesh_data_store =
            resources.get_add::<IdStore<MeshData<DefaultVertex>>>(IdStore::new());
        let id = mesh_data_store.push(mesh_data);
        mesh.data_id = Some(id);
        Ok(mesh)
    }
}
