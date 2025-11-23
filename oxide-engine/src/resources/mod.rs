use std::{
    any::{Any, TypeId},
    collections::HashMap,
    ops::{Deref, DerefMut},
    sync::{Arc, RwLock},
};

use once_cell::sync::Lazy;

use crate::{ecs::{system::SystemInput, world::World}, error::EngineError};

pub trait ResourceMarker: Send + Sync + 'static {}
impl<T: Send + Sync + 'static> ResourceMarker for T {}

static RESOURCES: Lazy<RwLock<HashMap<TypeId, Box<dyn Any + Send + Sync>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

pub struct Resource<R: ResourceMarker> {
    lock: Arc<RwLock<R>>,
}

impl<R: ResourceMarker> Deref for Resource<R> {
    type Target = R;
    fn deref(&self) -> &Self::Target {
        let guard = self.lock.read().unwrap();
        let r = &*guard as *const Self::Target;
        unsafe { &*r }
    }
}

impl<R: ResourceMarker> DerefMut for Resource<R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let mut guard = self.lock.write().unwrap();
        let x = guard.deref_mut();
        let r = x as *mut Self::Target;
        unsafe { &mut *r }
    }
}

impl<T: ResourceMarker> SystemInput for Resource<T> {
    fn borrow() -> Self {
        let lock = Resources::get().unwrap();
        Resource { lock }
    }
}

pub struct Resources {}
impl Resources {
    pub fn add<R: ResourceMarker>(resource: R) -> Result<(), ResourceError> {
        let mut resources = RESOURCES.write().unwrap();
        let type_id = TypeId::of::<R>();
        if resources.contains_key(&type_id) {
            return Err(ResourceError::ResourceAlreadyExists);
        }
        resources.insert(type_id, Box::new(Arc::new(RwLock::new(resource))));
        Ok(())
    }

    pub fn add_or_change<R: ResourceMarker>(resource: R) {
        let mut resources = RESOURCES.write().unwrap();
        let type_id = TypeId::of::<R>();
        resources.insert(type_id, Box::new(Arc::new(RwLock::new(resource))));
    }

    fn get<R: ResourceMarker>() -> Result<Arc<RwLock<R>>, ResourceError> {
        let resources = RESOURCES.read().expect("Clean resources read");
        let type_id = TypeId::of::<R>();
        let lock = match resources.get(&type_id) {
            Some(lock_any) => lock_any
                .downcast_ref::<Arc<RwLock<R>>>()
                .expect("This downcast can't fail"),
            None => return Err(ResourceError::ResourceNotFound),
        };
        Ok(lock.clone())
    }

    pub fn remove<R: ResourceMarker>() -> Result<(), ResourceError> {
        let mut resources = RESOURCES.write().unwrap();
        let type_id = TypeId::of::<R>();
        resources
            .remove(&type_id)
            .map_or(Err(ResourceError::ResourceNotFound), |_| Ok(()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceError {
    UnableToBorrow,
    UnableToBorrowMutably,
    ResourceNotFound,
    ResourceAlreadyExists,
}

impl From<ResourceError> for EngineError {
    fn from(value: ResourceError) -> Self { EngineError::ResourceError(value) }
}
