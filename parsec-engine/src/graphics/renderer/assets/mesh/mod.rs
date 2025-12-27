use crate::{graphics::renderer::DefaultVertex, utils::{identifiable::Identifiable, IdType}};

pub mod obj;

pub struct Mesh {
    mesh_id: IdType,
    pub vertices: Vec<DefaultVertex>,
    pub indices: Vec<u32>,
    pub data_id: Option<u32>,
}

crate::create_counter!{ID_COUNTER}
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
    fn id(&self) -> IdType {
        self.mesh_id
    }
}
