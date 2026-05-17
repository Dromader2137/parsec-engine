use std::{
    any::{Any, TypeId},
    collections::HashSet,
};

use crate::ecs::resources::ResourceMarker;

#[derive(Debug)]
pub struct ResourceData {
    pub data: Box<dyn Any + Send + Sync + 'static>,
    pub dependencies: HashSet<TypeId>,
    pub depended_on: HashSet<TypeId>,
}

impl ResourceData {
    pub fn new_any(resource: Box<dyn ResourceMarker>) -> ResourceData {
        let any_resource = resource.as_any();
        ResourceData {
            data: any_resource,
            dependencies: HashSet::new(),
            depended_on: HashSet::new(),
        }
    }
}
