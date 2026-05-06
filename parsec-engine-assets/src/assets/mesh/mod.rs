use parsec_engine_graphics::pipeline::DefaultVertex;
use parsec_engine_utils::{IdType, create_counter, identifiable::Identifiable};

use crate::{Asset, AssetSource};

pub mod obj;

#[derive(Debug, serde::Serialize)]
pub struct CookedMesh {
    xd: [u8; 32]
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

impl AssetSource for Vec<u8> {
    fn parse(bytes: &[u8]) -> Self {
        bytes.to_vec()
    }
}

impl Asset for Mesh {
    type Source = Vec<u8>;
    type Cooked = CookedMesh;

    const ASSET_TYPE: &'static str = "mesh";
    const EXTENSIONS: &'static [&'static str] = &["obj"];

    fn cook(source: Self::Source) -> Self::Cooked {
        CookedMesh { xd: [3; 32] }
    }

    fn load(cooked: Self::Cooked) -> Self {
        Self::new(vec![], vec![])
    }
}
