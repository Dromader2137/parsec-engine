//! Module responsible for storing and getting global state.

mod data;
pub mod dependency;

use std::{
    any::{Any, TypeId, type_name},
    collections::{HashMap, HashSet},
    marker::PhantomData,
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex, MutexGuard},
};

use thiserror::Error;

use crate::{
    ecs::{system::SystemInput, world::World},
    error::ParsecError,
    resources::{data::ResourceData, dependency::ResourceDependencyData},
};

/// Marks a type as a resource that can be stored in a global storage.
pub trait ResourceMarker: Send + Sync + 'static {
    fn resource_id(&self) -> TypeId;
    fn as_any(self: Box<Self>) -> Box<dyn Any + Send + Sync + 'static>;
}

impl<T: Send + Sync + 'static> ResourceMarker for T {
    fn resource_id(&self) -> TypeId { self.type_id() }
    fn as_any(self: Box<Self>) -> Box<dyn Any + Send + Sync + 'static> { self }
}

pub struct ResourceRemoveData {
    pub type_id: TypeId,
}

impl ResourceRemoveData {
    pub fn id<R: ResourceMarker>() -> ResourceRemoveData {
        ResourceRemoveData {
            type_id: TypeId::of::<R>(),
        }
    }
}

/// Stores the information necessary to use a global resource.
pub struct Resource<R: ResourceMarker> {
    guard:
        ManuallyDrop<MutexGuard<'static, Box<dyn Any + Send + Sync + 'static>>>,
    _arc: Arc<Mutex<Box<dyn Any + Send + Sync + 'static>>>,
    _marker: PhantomData<R>,
}

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
                MutexGuard<'_, Box<dyn Any + Send + Sync + 'static>>,
                MutexGuard<'static, Box<dyn Any + Send + Sync + 'static>>,
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
    resources: HashMap<TypeId, ResourceData>,
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
        self.resources
            .insert(type_id, ResourceData::new_any(resource));
    }

    pub fn add_dependency<R: ResourceMarker, D: ResourceMarker>(
        &mut self,
    ) -> Result<(), ResourceError> {
        let dependency_data = ResourceDependencyData::new::<R, D>();
        self.add_dependency_using_data(dependency_data)
    }

    pub fn add_dependency_using_data(
        &mut self,
        dependency_data: ResourceDependencyData,
    ) -> Result<(), ResourceError> {
        let resource = self
            .resources
            .get_mut(&dependency_data.resource)
            .ok_or(ResourceError::ResourceNotFound("UNKNOWN NAME"))?;
        resource.dependencies.insert(dependency_data.dependency);
        let dependency = self
            .resources
            .get_mut(&dependency_data.dependency)
            .ok_or(ResourceError::ResourceNotFound("UNKNOWN NAME"))?;
        dependency.depended_on.insert(dependency_data.resource);
        Ok(())
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
        let resource = self
            .resources
            .get(&type_id)
            .ok_or(ResourceError::ResourceNotFound(type_name::<R>()))?;
        Ok(resource.data.clone())
    }

    /// Removes the resource of type `R`.
    ///
    /// # Errors
    ///
    /// - If there is no resource of type `R`.
    pub fn remove<R: ResourceMarker>(&mut self) -> Result<(), ResourceError> {
        let remove_data = ResourceRemoveData::id::<R>();
        self.remove_using_data(remove_data)
    }

    pub fn remove_using_data(
        &mut self,
        data: ResourceRemoveData,
    ) -> Result<(), ResourceError> {
        let type_id = data.type_id;
        let resource = self
            .resources
            .get(&type_id)
            .ok_or(ResourceError::ResourceNotFound("UNKNOWN"))?;
        if Arc::weak_count(&resource.data) + Arc::strong_count(&resource.data)
            > 1
        {
            return Err(ResourceError::ResourceNotUnique);
        }
        if !resource.depended_on.is_empty() {
            return Err(ResourceError::ResourceDependedOn);
        }
        let mut resource = self
            .resources
            .remove(&type_id)
            .expect("resource doesn't exist");
        for dependency_id in resource.dependencies.drain() {
            let dependency = self
                .resources
                .get_mut(&dependency_id)
                .ok_or(ResourceError::ResourceNotFound("UNKNOWN"))?;
            dependency.depended_on.remove(&type_id);
        }
        for dependent_id in resource.depended_on.drain() {
            let dependent = self
                .resources
                .get_mut(&dependent_id)
                .ok_or(ResourceError::ResourceNotFound("UNKNOWN"))?;
            dependent.dependencies.remove(&type_id);
        }
        Ok(())
    }
}

fn resource_dfs(
    node_id: TypeId,
    resources: &HashMap<TypeId, ResourceData>,
    order: &mut Vec<TypeId>,
    visited: &mut HashSet<TypeId>,
) {
    visited.insert(node_id);
    let resource = resources
        .get(&node_id)
        .expect("tried to access a nonexistent resource");
    for dependency in resource.depended_on.iter() {
        if visited.contains(dependency) {
            continue;
        }
        resource_dfs(*dependency, resources, order, visited);
    }
    order.push(node_id);
}

impl Drop for Resources {
    fn drop(&mut self) {
        let mut visited = HashSet::new();
        let mut deletion_order = Vec::new();
        for key in self.resources.keys() {
            if visited.contains(key) {
                continue;
            }
            resource_dfs(
                *key,
                &self.resources,
                &mut deletion_order,
                &mut visited,
            );
        }

        for key in deletion_order {
            self.resources
                .remove(&key)
                .expect("tried to remove a nonexistent resource");
        }
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
    #[error("Other resources depend on this on so it can't be deleted")]
    ResourceDependedOn,
}
