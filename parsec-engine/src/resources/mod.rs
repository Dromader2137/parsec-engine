//! Module responsible for storing and getting global state.

use std::{
    any::{Any, TypeId, type_name},
    collections::HashMap,
    marker::PhantomData,
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex, MutexGuard},
};

use thiserror::Error;

use crate::{ecs::{system::SystemInput, world::World}, error::ParsecError};

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
    guard: ManuallyDrop<MutexGuard<'static, Box<dyn Any + Send + Sync>>>,
    _arc: Arc<Mutex<Box<dyn Any + Send + Sync>>>,
    _marker: PhantomData<R>,
}

// impl<R: ResourceMarker> Clone for Resource<R> {
//     fn clone(&self) -> Self {
//         Self {
//             lock: self.lock.clone(),
//             _marker: PhantomData::default(),
//         }
//     }
// }

impl<R: ResourceMarker> Deref for Resource<R> {
    type Target = R;
    fn deref(&self) -> &Self::Target {
        (*self.guard).downcast_ref::<R>().unwrap()
    }
}

impl<R: ResourceMarker> DerefMut for Resource<R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        (*self.guard).downcast_mut::<R>().unwrap()
    }
}

impl<R: ResourceMarker> Drop for Resource<R> {
    fn drop(&mut self) { unsafe { ManuallyDrop::drop(&mut self.guard) }; }
}

impl<T: ResourceMarker> SystemInput for Resource<T> {
    fn borrow(
        resources: &Resources,
        _world: &World,
    ) -> Result<Self, ParsecError> {
        let arc = resources.get::<T>()?;
        let locked = arc.lock().expect("mutex poisoned");
        let guard = unsafe {
            std::mem::transmute::<
                MutexGuard<'_, Box<dyn Any + Send + Sync>>,
                MutexGuard<'static, Box<dyn Any + Send + Sync>>,
            >(locked)
        };
        Ok(Resource {
            guard: ManuallyDrop::new(guard),
            _arc: arc,
            _marker: PhantomData::default(),
        })
    }
}

#[derive(Debug)]
pub struct Resources {
    resources: HashMap<TypeId, Arc<Mutex<Box<dyn Any + Send + Sync>>>>,
}
impl Resources {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }

    /// Adds `resource` to global storage. If a resource of type `R` already exists it is
    /// replaced.
    pub fn add<R: ResourceMarker>(&mut self, resource: R) {
        self.add_boxed(Box::new(resource));
    }

    /// Adds a box containing `resource` to global storage.
    pub fn add_boxed(&mut self, resource: Box<dyn ResourceMarker>) {
        let type_id = (*resource).resource_id();
        let any_resource = resource.as_any();
        self.resources
            .insert(type_id, Arc::new(Mutex::new(any_resource)));
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
        let lock = self
            .resources
            .get(&type_id)
            .ok_or(ResourceError::ResourceNotFound(type_name::<R>()))?;
        Ok(lock.clone())
    }

    /// Removes the resource of type `R`.
    ///
    /// # Errors
    ///
    /// - If there is no resource of type `R`.
    pub fn remove<R: ResourceMarker>(&mut self) -> Result<(), ResourceError> {
        let remove_data = RemoveResourceData::id::<R>();
        self.remove_using_data(remove_data)
    }

    pub fn remove_using_data(
        &mut self,
        data: RemoveResourceData,
    ) -> Result<(), ResourceError> {
        let type_id = data.type_id;
        let lock = self
            .resources
            .get(&type_id)
            .ok_or(ResourceError::ResourceNotFound("UNKNOWN"))?;
        if Arc::weak_count(&lock) + Arc::strong_count(&lock) > 1 {
            return Err(ResourceError::ResourceNotUnique);
        }
        self.resources
            .remove(&type_id)
            .expect("resource doesn't exist");
        Ok(())
    }
}

#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceError {
    #[error("Failed to borrow this resourcs")]
    UnableToBorrow,
    #[error("Failed to mutably borrow this resourcs")]
    UnableToBorrowMutably,
    #[error("Resource of this type already exists")]
    ResourceAlreadyExists,
    #[error("Resource of this type is also stored somewhere else")]
    ResourceNotUnique,
    #[error("Failed to find a resource of a type")]
    ResourceNotFound(&'static str),
}
