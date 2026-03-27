//! Module responsible for storing and getting global state.

use std::{
    any::{Any, TypeId, type_name},
    collections::HashMap,
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex},
};

use thiserror::Error;

use crate::ecs::{system::SystemInput, world::World};

/// Marks a type as a resource that can be stored in a global storage.
pub trait ResourceMarker: Send + Sync + 'static {}
impl<T: Send + Sync + 'static> ResourceMarker for T {}

/// Stores the information necessary to use a global resource.
pub struct Resource<R: ResourceMarker> {
    lock: Arc<Mutex<R>>,
}

impl<R: ResourceMarker> Clone for Resource<R> {
    fn clone(&self) -> Self {
        Self {
            lock: self.lock.clone(),
        }
    }
}

impl<R: ResourceMarker> Deref for Resource<R> {
    type Target = R;
    fn deref(&self) -> &Self::Target {
        let guard = self.lock.lock().expect("Mutex should not be poisoned");
        let r = guard.deref() as *const Self::Target;
        unsafe { &*r }
    }
}

impl<R: ResourceMarker> DerefMut for Resource<R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let mut guard = self.lock.lock().expect("Mutex should not be poisoned");
        let r = guard.deref_mut() as *mut Self::Target;
        unsafe { &mut *r }
    }
}

impl<T: ResourceMarker> SystemInput for Resource<T> {
    fn borrow(resources: &Resources, _world: &World) -> Self {
        let lock = resources.get().unwrap();
        Resource { lock }
    }
}

pub struct Resources {
    resources: HashMap<TypeId, Box<dyn Any + Send + Sync + 'static>>,
}
impl Resources {
    pub fn new() -> Self {
        Self { resources: HashMap::new() }
    }

    /// Adds `resource` to global storage.
    ///
    /// # Errors
    ///
    /// - If a resource of type `R` already exists.
    pub fn add<R: ResourceMarker>(
        &mut self,
        resource: R,
    ) -> Result<Resource<R>, ResourceError> {
        let type_id = TypeId::of::<R>();
        if self.resources.contains_key(&type_id) {
            return Err(ResourceError::ResourceAlreadyExists);
        }
        let resource = Arc::new(Mutex::new(resource));
        self.resources.insert(type_id, Box::new(resource.clone()));
        Ok(Resource { lock: resource })
    }

    /// Adds `resource` to global storage. If a resource of type `R` already exists it is
    /// replaced.
    pub fn add_or_change<R: ResourceMarker>(&mut self, resource: R) {
        let type_id = TypeId::of::<R>();
        self.resources.insert(type_id, Box::new(Arc::new(Mutex::new(resource))));
    }

    /// Gets the resource of type `R`.
    ///
    /// # Errors
    ///
    /// - If there is no resource of type `R`.
    fn get<R: ResourceMarker>(&self) -> Result<Arc<Mutex<R>>, ResourceError> {
        let type_id = TypeId::of::<R>();
        let lock = match self.resources.get(&type_id) {
            Some(lock_any) => lock_any
                .downcast_ref::<Arc<Mutex<R>>>()
                .expect("This downcast can't fail"),
            None => {
                return Err(ResourceError::ResourceNotFoundExact(
                    type_name::<R>(),
                ));
            },
        };
        Ok(lock.clone())
    }

    /// Removes the resource of type `R`.
    ///
    /// # Errors
    ///
    /// - If there is no resource of type `R`.
    pub fn remove<R: ResourceMarker>(&mut self) -> Result<(), ResourceError> {
        let type_id = TypeId::of::<R>();
        let lock = match self.resources.get(&type_id) {
            Some(lock_any) => lock_any
                .downcast_ref::<Arc<Mutex<R>>>()
                .expect("This downcast can't fail"),
            None => return Err(ResourceError::ResourceNotFound),
        };
        if Arc::weak_count(lock) + Arc::strong_count(lock) > 1 {
            return Err(ResourceError::ResourceNotUnique);
        }
        self.resources
            .remove(&type_id)
            .map_or(Err(ResourceError::ResourceNotFound), |_| Ok(()))
    }
}

#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceError {
    #[error("Failed to borrow this resourcs")]
    UnableToBorrow,
    #[error("Failed to mutably borrow this resourcs")]
    UnableToBorrowMutably,
    #[error("Failed to find a resource of a type")]
    ResourceNotFound,
    #[error("Resource of this type already exists")]
    ResourceAlreadyExists,
    #[error("Resource of this type is also stored somewhere else")]
    ResourceNotUnique,
    #[error("Failed to find a resource of a type")]
    ResourceNotFoundExact(&'static str),
}
