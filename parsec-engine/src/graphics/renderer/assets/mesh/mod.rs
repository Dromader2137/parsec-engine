use crate::graphics::renderer::DefaultVertex;

pub mod obj;

pub struct Mesh {
    pub vertices: Vec<DefaultVertex>,
    pub indices: Vec<u32>,
    pub data_id: Option<u32>,
}

impl Mesh {
    pub fn new(vertices: Vec<DefaultVertex>, indices: Vec<u32>) -> Mesh {
        Mesh {
            vertices,
            indices,
            data_id: None,
        }
    }
}
