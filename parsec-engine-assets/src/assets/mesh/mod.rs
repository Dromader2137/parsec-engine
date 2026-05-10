use parsec_engine_ecs::world::World;
use parsec_engine_graphics::pipeline::DefaultVertex;
use parsec_engine_math::vec::{Vec2f, Vec3f};
use parsec_engine_utils::{IdType, create_counter, identifiable::Identifiable};

use crate::{Asset, assets::mesh::obj::cook_obj};

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

    fn load(cooked: Self::Cooked, _world: &World) -> Self {
        let mesh = cooked.into();

        mesh
    }
}
