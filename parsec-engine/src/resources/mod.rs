//! Module responsible for storing and getting global state.

use std::{
    any::{Any, TypeId, type_name},
    collections::HashMap,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex},
};

use thiserror::Error;

use crate::ecs::{system::SystemInput, world::World};

/// Marks a type as a resource that can be stored in a global storage.
pub trait ResourceMarker: Send + Sync + 'static {
    fn resource_id(&self) -> TypeId;
    fn as_any(self: Box<Self>) -> Box<dyn Any + Send + Sync + 'static>;
}
impl<T: Send + Sync + 'static> ResourceMarker for T {
    fn resource_id(&self) -> TypeId { self.type_id() }
    fn as_any(self: Box<Self>) -> Box<dyn Any + Send + Sync + 'static> { self }
}

pub struct RemoveResourceData {
    pub type_id: TypeId,
}

impl RemoveResourceData {
    pub fn id<R: ResourceMarker>() -> RemoveResourceData {
        RemoveResourceData {
            type_id: TypeId::of::<R>(),
        }
    }
}

/// Stores the information necessary to use a global resource.
pub struct Resource<R: ResourceMarker> {
    lock: Arc<Mutex<Box<dyn Any + Send + Sync>>>,
    _marker: PhantomData<R>,
}

impl<R: ResourceMarker> Clone for Resource<R> {
    fn clone(&self) -> Self {
        Self {
            lock: self.lock.clone(),
            _marker: PhantomData::default(),
        }
    }
}

impl<R: ResourceMarker> Deref for Resource<R> {
    type Target = R;
    fn deref(&self) -> &Self::Target {
        let guard = self.lock.lock().expect("Mutex should not be poisoned");
        let reference = guard.downcast_ref::<R>().unwrap();
        let r = reference as *const R;
        unsafe { &*r }
    }
}

impl<R: ResourceMarker> DerefMut for Resource<R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let mut guard = self.lock.lock().expect("Mutex should not be poisoned");
        let reference = guard.downcast_mut::<R>().unwrap();
        let r = reference as *mut R;
        unsafe { &mut *r }
    }
}

impl<T: ResourceMarker> SystemInput for Resource<T> {
    fn borrow(resources: &Resources, _world: &World) -> Self {
        let lock = resources.get::<T>().unwrap();
        Resource {
            lock,
            _marker: PhantomData::default(),
        }
    }
}

#[derive(Debug)]
pub struct Resources {
    resources: HashMap<
        TypeId,
        (
            Arc<u8>,
            Box<Arc<Mutex<Box<dyn Any + Send + Sync + 'static>>>>,
        ),
    >,
}
impl Resources {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }

    /// Adds a box containing `resource` to global storage.
    pub fn add_boxed(&mut self, resource: Box<dyn ResourceMarker>) {
        let type_id = (*resource).resource_id();
        let any_resource = resource.as_any();
        self.resources.insert(
            type_id,
            (Arc::new(0), Box::new(Arc::new(Mutex::new(any_resource)))),
        );
    }

    /// Adds `resource` to global storage. If a resource of type `R` already exists it is
    /// replaced.
    pub fn add<R: ResourceMarker>(&mut self, resource: R) {
        let type_id = resource.resource_id();
        self.resources.insert(
            type_id,
            (
                Arc::new(0),
                Box::new(Arc::new(Mutex::new(Box::new(resource)))),
            ),
        );
    }

    /// Gets the resource of type `R`.
    ///
    /// # Errors
    ///
    /// - If there is no resource of type `R`.
    pub fn get<R: ResourceMarker>(
        &self,
    ) -> Result<Arc<Mutex<Box<dyn Any + Send + Sync>>>, ResourceError> {
        let type_id = TypeId::of::<R>();
        let lock = match self.resources.get(&type_id) {
            Some(lock_any) => lock_any.1.clone(),
            None => {
                return Err(ResourceError::ResourceNotFoundExact(
                    type_name::<R>(),
                ));
            },
        };
        Ok(*lock)
    }

    /// Removes the resource of type `R`.
    ///
    /// # Errors
    ///
    /// - If there is no resource of type `R`.
    pub fn remove<R: ResourceMarker>(&mut self) -> Result<(), ResourceError> {
        let type_id = TypeId::of::<R>();
        let lock = match self.resources.get(&type_id) {
            Some(lock_any) => lock_any,
            None => return Err(ResourceError::ResourceNotFound),
        };
        if Arc::weak_count(&lock.0) + Arc::strong_count(&lock.0) > 1 {
            return Err(ResourceError::ResourceNotUnique);
        }
        self.resources
            .remove(&type_id)
            .map_or(Err(ResourceError::ResourceNotFound), |_| Ok(()))
    }

    pub fn remove_using_data(
        &mut self,
        data: RemoveResourceData,
    ) -> Result<(), ResourceError> {
        let type_id = data.type_id;
        let lock = match self.resources.get(&type_id) {
            Some(lock_any) => lock_any,
            None => return Err(ResourceError::ResourceNotFound),
        };
        if Arc::weak_count(&lock.0) + Arc::strong_count(&lock.0) > 1 {
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
