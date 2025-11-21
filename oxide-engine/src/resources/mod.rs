use std::{
    any::{Any, TypeId},
    collections::HashMap,
    ops::{Deref, DerefMut},
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use once_cell::sync::Lazy;

use crate::error::EngineError;

pub trait Resource: Send + Sync + Sized + 'static {}
impl<T: Send + Sync + Sized + 'static> Resource for T {}

struct ResourceCollection {
    //                         Box<Arc<RwLock<dyn Any + Send + Sync>>>>,
    resources: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

static RESOURCES: Lazy<RwLock<ResourceCollection>> =
    Lazy::new(|| RwLock::new(ResourceCollection::new()));

pub struct Rsc<'a, R: Resource> {
    inner: Arc<RwLock<R>>,
    guard: Option<RwLockReadGuard<'a, R>>,
}

impl<'a, R: Resource> Rsc<'a, R> {
    pub fn add(resource: R) -> Result<(), ResourceError> {
        let mut resources = RESOURCES.write().unwrap();
        resources.add(resource)
    }

    pub fn add_overwrite(resource: R) -> Result<(), ResourceError> {
        let mut resources = RESOURCES.write().unwrap();
        resources.add_or_change(resource)
    }

    pub fn get() -> Result<Rsc<'a, R>, ResourceError> {
        let resources = RESOURCES.read().unwrap();
        let inner = resources.get::<R>()?;
        Ok(Rsc { inner, guard: None })
    }

    pub fn remove() -> Result<(), ResourceError> {
        let mut resources = RESOURCES.write().unwrap();
        resources.remove::<R>()
    }
}

impl<'a, R: Resource> Deref for Rsc<'a, R> {
    type Target = R;
    fn deref(&self) -> &Self::Target {
        unsafe {
            let this = self as *const _ as *mut Self;
            if (*this).guard.is_none() {
                (*this).guard = Some((*this).inner.read().unwrap());
            }
            (*this).guard.as_ref().unwrap()
        }
    }
}

pub struct RscMut<'a, R: Resource> {
    inner: Arc<RwLock<R>>,
    guard: Option<RwLockWriteGuard<'a, R>>,
}

impl<'a, R: Resource> RscMut<'a, R> {
    pub fn add(resource: R) -> Result<(), ResourceError> {
        let mut resources = RESOURCES.write().unwrap();
        resources.add(resource)
    }

    pub fn add_overwrite(resource: R) -> Result<(), ResourceError> {
        let mut resources = RESOURCES.write().unwrap();
        resources.add_or_change(resource)
    }

    pub fn get() -> Result<RscMut<'a, R>, ResourceError> {
        let resources = RESOURCES.read().unwrap();
        let inner = resources.get::<R>()?;
        Ok(RscMut { inner, guard: None })
    }

    pub fn remove() -> Result<(), ResourceError> {
        let mut resources = RESOURCES.write().unwrap();
        resources.remove::<R>()
    }
}

impl<'a, R: Resource> Deref for RscMut<'a, R> {
    type Target = R;
    fn deref(&self) -> &Self::Target {
        unsafe {
            let this = self as *const _ as *mut Self;
            if (*this).guard.is_none() {
                (*this).guard = Some((*this).inner.write().unwrap());
            }
            (*this).guard.as_ref().unwrap()
        }
    }
}

impl<'a, R: Resource> DerefMut for RscMut<'a, R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            let this = self as *const _ as *mut Self;
            if (*this).guard.is_none() {
                (*this).guard = Some((*this).inner.write().unwrap());
            }
            (*this).guard.as_mut().unwrap()
        }
    }
}

impl ResourceCollection {
    fn new() -> ResourceCollection {
        ResourceCollection {
            resources: HashMap::new(),
        }
    }

    fn add<R: Resource>(&mut self, resource: R) -> Result<(), ResourceError> {
        let type_id = TypeId::of::<R>();
        if self.resources.contains_key(&type_id) {
            return Err(ResourceError::ResourceAlreadyExists);
        }
        self.resources
            .insert(type_id, Box::new(Arc::new(RwLock::new(resource))));
        Ok(())
    }

    fn add_or_change<R: Resource>(&mut self, resource: R) -> Result<(), ResourceError> {
        let type_id = TypeId::of::<R>();
        self.resources
            .insert(type_id, Box::new(Arc::new(RwLock::new(resource))));
        Ok(())
    }

    fn get<R: Resource>(&self) -> Result<Arc<RwLock<R>>, ResourceError> {
        let type_id = TypeId::of::<R>();
        if let Some(resource_any) = self.resources.get(&type_id) {
            let resource = resource_any
                .downcast_ref::<Arc<RwLock<R>>>()
                .expect("This downcast can't fail");
            Ok(resource.clone())
        } else {
            Err(ResourceError::ResourceNotFound)
        }
    }

    fn remove<R: Resource>(&mut self) -> Result<(), ResourceError> {
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
