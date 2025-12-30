use std::collections::{
    HashMap,
    hash_map::{Values, ValuesMut},
};

use crate::utils::IdType;

pub trait Identifiable {
    fn id(&self) -> IdType;
}

pub struct IdStore<T: Identifiable> {
    elements: HashMap<IdType, T>,
}

impl<T: Identifiable> IdStore<T> {
    pub fn new() -> IdStore<T> {
        IdStore {
            elements: HashMap::new(),
        }
    }

    pub fn push(&mut self, value: T) -> IdType {
        let id = value.id();
        self.elements.insert(value.id(), value);
        id
    }

    pub fn get(&self, id: IdType) -> Option<&T> { self.elements.get(&id) }

    pub fn get_mut(&mut self, id: IdType) -> Option<&mut T> {
        self.elements.get_mut(&id)
    }

    pub fn iter(&mut self) -> Values<'_, u32, T> { self.elements.values() }

    pub fn iter_mut(&mut self) -> ValuesMut<'_, u32, T> {
        self.elements.values_mut()
    }
}
