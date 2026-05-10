use parsec_engine_math::vec::{Vec2f, Vec3f};

use crate::{
    assets::{Asset, assets::mesh::obj::cook_obj},
    create_counter,
    ecs::world::World,
    error::OptionNoneErr,
    graphics::{ActiveGraphicsBackend, pipeline::DefaultVertex},
    renderer::mesh_data::MeshData,
    utils::{
        IdType,
        identifiable::{IdStore, Identifiable},
    },
};

pub mod obj;

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
        Self {
            positions,
            normals,
            uvs,
            indices,
        }
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
        Mesh {
            mesh_id: ID_COUNTER.next(),
            vertices,
            indices,
            data_id: None,
        }
    }
}

impl Identifiable for Mesh {
    fn id(&self) -> IdType { self.mesh_id }
}

impl From<CookedMesh> for Mesh {
    fn from(value: CookedMesh) -> Self {
        let vertices = value
            .positions
            .iter()
            .zip(value.uvs.iter())
            .zip(value.normals.iter())
            .map(|((pos, uv), norm)| DefaultVertex::new(*pos, *norm, *uv))
            .collect();
        Self::new(vertices, value.indices)
    }
}

impl Asset for Mesh {
    type Cooked = CookedMesh;

    const ASSET_TYPE: &'static str = "mesh";
    const EXTENSIONS: &'static [&'static str] = &["obj"];

    fn cook(data: &[u8], extension: &str) -> Self::Cooked {
        if extension == "obj" {
            let cooked = cook_obj(data).unwrap();
            return cooked;
        }
        CookedMesh::default()
    }

    fn load(cooked: Self::Cooked, world: &mut World) -> Self {
        let mut backend = world
            .resources
            .get::<ActiveGraphicsBackend>()
            .none_err()
            .unwrap();
        let mut mesh = Mesh::from(cooked);
        let mesh_data =
            MeshData::new(&mut backend, &mesh.vertices, &mesh.indices);
        let mut mesh_data_store =
            world.resources.get::<IdStore<MeshData<DefaultVertex>>>();
        if mesh_data_store.is_none() {
            world
                .resources
                .add(IdStore::<MeshData<DefaultVertex>>::new());
            mesh_data_store = world
                .resources
                .get::<IdStore<MeshData<DefaultVertex>>>();
        }
        let mut mesh_data_store = mesh_data_store.unwrap();
        let id = mesh_data_store.push(mesh_data);
        mesh.data_id = Some(id);
        mesh
    }
}
