use crate::{
    ecs::world::component::Component, graphics::renderer::DefaultVertex,
    resources::ResourceCollection,
};

#[derive(Component, Clone)]
pub struct MeshRenderer {
    pub id: u32,
    pub vertices: Vec<DefaultVertex>,
    pub indices: Vec<u32>,
}

impl MeshRenderer {
    pub fn new(
        resources: &ResourceCollection,
        vertices: Vec<DefaultVertex>,
        indices: Vec<u32>,
    ) -> MeshRenderer {
        let mut mesh_renderer_controller = resources.get_mut::<MeshRendererController>().unwrap();
        let id = mesh_renderer_controller.id_counter;
        mesh_renderer_controller.id_counter += 1;
        mesh_renderer_controller.just_added.push(id);
        MeshRenderer {
            id,
            vertices,
            indices,
        }
    }
}

#[derive(Default)]
pub struct MeshRendererController {
    id_counter: u32,
    pub just_added: Vec<u32>,
}
