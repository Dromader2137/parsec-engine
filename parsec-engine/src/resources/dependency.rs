use std::any::TypeId;

use crate::resources::ResourceMarker;

pub struct ResourceDependencyData {
    pub resource: TypeId,
    pub dependency: TypeId,
}

impl ResourceDependencyData {
    pub fn new<R: ResourceMarker, D: ResourceMarker>() -> Self {
        Self {
            resource: TypeId::of::<R>(),
            dependency: TypeId::of::<D>(),
        }
    }
}
