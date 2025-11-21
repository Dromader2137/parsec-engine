use crate::{
    assets::{Asset, AssetError, AssetLoadInput},
    graphics::renderer::{DefaultVertex, mesh_data::create_mesh_data},
};

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

impl Asset for Mesh {
    fn on_load(
        &mut self,
        AssetLoadInput { .. }: AssetLoadInput,
    ) -> Result<(), AssetError> {
        self.data_id = Some(create_mesh_data(&self.vertices, &self.indices).unwrap());
        Ok(())
    }
}
