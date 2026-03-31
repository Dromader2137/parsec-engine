use crate::{
    ecs::{
        entity::Entity,
        world::{
            World,
            add_component::AddComponent,
            remove_component::{RemoveComponent, RemoveComponentData},
            spawn::Spawn,
        },
    },
    resources::{RemoveResourceData, ResourceMarker, Resources},
};

pub enum Request {
    Spawn(Entity, Box<dyn Spawn>),
    Delete(Entity),
    AddComponents(Entity, Box<dyn AddComponent>),
    RemoveComponents(Entity, RemoveComponentData),
    CreateResource(Box<dyn ResourceMarker>),
    RemoveResource(RemoveResourceData),
}

pub struct Requests {
    entity_counter: u32,
    requests: Vec<Request>,
}

impl Requests {
    pub fn new(entity_counter: u32) -> Requests {
        Requests {
            entity_counter,
            requests: Vec::new(),
        }
    }

    pub fn handle_requests(
        &mut self,
        world: &mut World,
        resources: &mut Resources,
    ) -> Result<(), anyhow::Error> {
        for request in self.requests.drain(0..self.requests.len()) {
            match request {
                Request::Spawn(entity, bundle) => {
                    world.spawn_with_id(entity, bundle)?
                },
                Request::Delete(entity) => world.delete(entity)?,
                Request::AddComponents(entity, bundle_extension) => {
                    world.add_components(entity, bundle_extension)?
                },
                Request::RemoveComponents(entity, bundle_removal) => world
                    .remove_components_using_data(entity, bundle_removal)?,
                Request::CreateResource(resource) => {
                    resources.add_boxed(resource);
                },
                Request::RemoveResource(resource_remove) => {
                    resources.remove_using_data(resource_remove)?
                },
            };
        }
        world.current_id = self.entity_counter;
        Ok(())
    }

    pub fn spawn_entity(&mut self, bundle: impl Spawn) -> Entity {
        let entity = Entity::new(self.entity_counter);
        self.entity_counter += 1;
        self.requests.push(Request::Spawn(entity, Box::new(bundle)));
        entity
    }

    pub fn delete_entity(&mut self, entity: Entity) {
        self.requests.push(Request::Delete(entity));
    }

    pub fn add_components(
        &mut self,
        entity: Entity,
        bundle: impl AddComponent,
    ) {
        self.requests
            .push(Request::AddComponents(entity, Box::new(bundle)));
    }

    pub fn remove_components<T: RemoveComponent>(&mut self, entity: Entity) {
        self.requests.push(Request::RemoveComponents(
            entity,
            RemoveComponentData::components::<T>().unwrap(),
        ));
    }

    pub fn create_resource(&mut self, resource: impl ResourceMarker) {
        self.requests
            .push(Request::CreateResource(Box::new(resource)));
    }

    pub fn remove_resource<T: ResourceMarker>(&mut self) {
        self.requests
            .push(Request::RemoveResource(RemoveResourceData::id::<T>()));
    }
}
