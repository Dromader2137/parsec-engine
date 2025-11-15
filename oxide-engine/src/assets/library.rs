use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use crate::assets::Asset;

pub struct AssetLibrary {
    assets: HashMap<TypeId, Box<dyn AssetVec>>,
}

#[derive(Debug)]
pub enum AssetLibraryError {
    AssetVectorTypeMismatch,
}

pub trait AssetVec {
    fn add_boxed(&mut self, value: Box<dyn Any>) -> Result<u32, AssetLibraryError>;
    fn get_all(&self) -> Vec<&dyn Any>;
}

impl<A: Asset> AssetVec for Vec<A> {
    fn add_boxed(&mut self, value: Box<dyn Any>) -> Result<u32, AssetLibraryError> {
        if let Ok(downcasted) = value.downcast::<A>() {
            self.push(*downcasted);
            Ok(self.len() as u32 - 1)
        } else {
            Err(AssetLibraryError::AssetVectorTypeMismatch)
        }
    }

    fn get_all(&self) -> Vec<&dyn Any> {
        self.iter().map(|a| a as &dyn Any).collect()
    }
}

impl AssetLibrary {
    pub fn new() -> AssetLibrary {
        AssetLibrary {
            assets: HashMap::new(),
        }
    }

    pub fn add<A: Asset>(&mut self, value: A) -> Result<u32, AssetLibraryError> {
        let type_id = TypeId::of::<A>();

        let vec = self
            .assets
            .entry(type_id)
            .or_insert(Box::new(Vec::<A>::new()));

        vec.add_boxed(Box::new(value))
    }

    pub fn get_all<A: Asset>(&self) -> Vec<&A> {
        let type_id = TypeId::of::<A>();

        if let Some(vec) = self.assets.get(&type_id) {
            return vec
                .get_all()
                .iter()
                .map(|x| x.downcast_ref::<A>().unwrap())
                .collect();
        }

        vec![]
    }

    pub fn get_one<A: Asset>(&self, id: usize) -> Option<&A> {
        let type_id = TypeId::of::<A>();

        if let Some(vec) = self.assets.get(&type_id) {
            return vec
                .get_all()
                .iter()
                .map(|x| x.downcast_ref::<A>().unwrap())
                .nth(id);
        }

        None
    }
}
