use std::{
    any::{Any, TypeId},
    collections::HashSet,
    sync::{Arc, Mutex},
};

use crate::resources::ResourceMarker;

#[derive(Debug)]
pub struct ResourceData {
    pub data: Arc<Mutex<Box<dyn Any + Send + Sync + 'static>>>,
    pub dependencies: HashSet<TypeId>,
    pub depended_on: HashSet<TypeId>,
}

impl ResourceData {
    pub fn new_any(resource: Box<dyn ResourceMarker>) -> ResourceData {
        let any_resource = resource.as_any();
        ResourceData {
            data: Arc::new(Mutex::new(any_resource)),
            dependencies: HashSet::new(),
            depended_on: HashSet::new(),
        }
    }
}
