use std::{
    any::{Any, TypeId},
    collections::HashMap,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use once_cell::sync::Lazy;

use crate::error::EngineError;

pub trait Resource: Send + Sync + 'static {}
impl<T: Send + Sync + 'static> Resource for T {}

pub struct ResourceCollection {
    resources: HashMap<TypeId, RwLock<Box<dyn Any + Send + Sync>>>,
}

pub struct Rsc<'a, T: Resource> {
    borrow: RwLockReadGuard<'a, Box<dyn Any + Send + Sync>>,
    _marker: PhantomData<T>,
}

impl<'a, T: Resource> Deref for Rsc<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.borrow
            .downcast_ref::<T>()
            .expect("Rsc downcast should never fail")
    }
}

pub struct RscMut<'a, T: Resource> {
    borrow: RwLockWriteGuard<'a, Box<dyn Any + Send + Sync>>,
    _marker: PhantomData<T>,
}

impl<'a, T: Resource> Deref for RscMut<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.borrow
            .downcast_ref::<T>()
            .expect("RscMut downcast should never fail")
    }
}

impl<'a, T: Resource> DerefMut for RscMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.borrow
            .downcast_mut::<T>()
            .expect("RscMut downcast should never fail")
    }
}

static RESOURCES: Lazy<RwLock<ResourceCollection>> =
    Lazy::new(|| RwLock::new(ResourceCollection::new()));

pub struct Resources {}
impl Resources {
    pub fn add<R: Resource>(resource: R) -> Result<(), ResourceError> {
        let mut resources = RESOURCES.write();
    }
} 

impl ResourceCollection {
    pub fn new() -> ResourceCollection {
        ResourceCollection {
            resources: HashMap::new(),
        }
    }

    pub fn add<R: Resource>(&mut self, resource: R) -> Result<(), ResourceError> {
        let type_id = TypeId::of::<R>();
        if self.resources.contains_key(&type_id) {
            return Err(ResourceError::ResourceAlreadyExists);
        }
        self.resources
            .insert(type_id, RwLock::new(Box::new(resource)));
        Ok(())
    }

    pub fn add_or_change<R: Resource>(&mut self, resource: R) -> Result<(), ResourceError> {
        let type_id = TypeId::of::<R>();
        self.resources
            .insert(type_id, RwLock::new(Box::new(resource)));
        Ok(())
    }

    pub fn get<'a, 'b, R: Resource>(&'a self) -> Result<Rsc<'b, R>, ResourceError>
    where
        'a: 'b,
    {
        let type_id = TypeId::of::<R>();
        if let Some(resource_cell) = self.resources.get(&type_id) {
            let resource = match resource_cell.try_read() {
                Ok(val) => val,
                Err(_) => return Err(ResourceError::UnableToBorrow),
            };
            return Ok(Rsc {
                borrow: resource,
                _marker: PhantomData::default(),
            });
        }
        Err(ResourceError::ResourceNotFound)
    }

    pub fn get_mut<'a, 'b, R: Resource>(&'a self) -> Result<RscMut<'b, R>, ResourceError>
    where
        'a: 'b,
    {
        let type_id = TypeId::of::<R>();
        if let Some(resource_cell) = self.resources.get(&type_id) {
            let resource = match resource_cell.try_write() {
                Ok(val) => val,
                Err(_) => return Err(ResourceError::UnableToBorrow),
            };
            return Ok(RscMut {
                borrow: resource,
                _marker: PhantomData::default(),
            });
        }
        Err(ResourceError::ResourceNotFound)
    }

    pub fn remove<R: Resource>(&mut self) -> Result<(), ResourceError> {
        let type_id = TypeId::of::<R>();
        self.resources
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
