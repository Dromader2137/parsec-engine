use std::{
    any::{Any, TypeId}, cell::{Ref, RefCell, RefMut}, collections::HashMap, marker::PhantomData, ops::{Deref, DerefMut}
};

use crate::error::EngineError;

pub trait Resource: 'static {}
impl<T: 'static> Resource for T {}

pub struct ResourceCollection {
    resources: HashMap<TypeId, RefCell<Box<dyn Any>>>,
}

pub struct Rsc<'a, T: Resource> {
    borrow: Ref<'a, Box<dyn Any>>,
    _marker: PhantomData<T>
}

impl<'a, T: Resource> Deref for Rsc<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.borrow.downcast_ref::<T>().expect("Rsc downcast should never fail")
    }
}

pub struct RscMut<'a, T: Resource> {
    borrow: RefMut<'a, Box<dyn Any>>,
    _marker: PhantomData<T>
}

impl<'a, T: Resource> Deref for RscMut<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.borrow.downcast_ref::<T>().expect("RscMut downcast should never fail")
    }
}

impl<'a, T: Resource> DerefMut for RscMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.borrow.downcast_mut::<T>().expect("RscMut downcast should never fail")
    }
}

impl ResourceCollection {
    pub fn new() -> ResourceCollection {
        ResourceCollection { resources: HashMap::new() }
    }

    pub fn add<R: Resource>(&mut self, resource: R) -> Result<(), ResourceError> {
        let type_id = TypeId::of::<R>();
        if self.resources.contains_key(&type_id) {
            return Err(ResourceError::ResourceAlreadyExists)
        } 
        self.resources.insert(type_id, RefCell::new(Box::new(resource)));
        Ok(())
    }

    pub fn get<'a, 'b, R: Resource>(&'a self) -> Result<Rsc<'b, R>, ResourceError> where 'a: 'b {
        let type_id = TypeId::of::<R>();
        if let Some(resource_cell) = self.resources.get(&type_id) {
            let resource = match resource_cell.try_borrow() {
                Ok(val) => val,
                Err(_) => return Err(ResourceError::UnableToBorrow)
            };
            return Ok(Rsc { borrow: resource, _marker: PhantomData::default() });
        }
        Err(ResourceError::ResourceNotFound)
    }

    pub fn get_mut<'a, 'b, R: Resource>(&'a self) -> Result<RscMut<'b, R>, ResourceError> where 'a: 'b {
        let type_id = TypeId::of::<R>();
        if let Some(resource_cell) = self.resources.get(&type_id) {
            let resource = match resource_cell.try_borrow_mut() {
                Ok(val) => val,
                Err(_) => return Err(ResourceError::UnableToBorrow)
            };
            return Ok(RscMut { borrow: resource, _marker: PhantomData::default() });
        }
        Err(ResourceError::ResourceNotFound)
    }

    pub fn remove<R: Resource>(&mut self) -> Result<(), ResourceError> {
        let type_id = TypeId::of::<R>();
        self.resources.remove(&type_id).map_or(Err(ResourceError::ResourceNotFound), |_| Ok(()))
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
    fn from(value: ResourceError) -> Self {
        EngineError::ResourceError(value)
    }
}
