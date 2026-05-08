use std::fs::File;

use parsec_engine_graphics::pipeline::DefaultVertex;
use parsec_engine_utils::{IdType, create_counter, identifiable::Identifiable};

use crate::Asset;

pub mod obj;

#[derive(Debug, serde::Serialize)]
pub struct CookedMesh {
    xd: [u8; 32],
}

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

impl Asset for Mesh {
    type Cooked = CookedMesh;

    const ASSET_TYPE: &'static str = "mesh";
    const EXTENSIONS: &'static [&'static str] = &["obj"];

    fn cook(file: File) -> Self::Cooked {
        CookedMesh { xd: [3; 32] } 
    }

    fn load(_cooked: Self::Cooked, _world: &parsec_engine_ecs::world::World) -> Self {
        Self::new(vec![], vec![])
    }
}
