use std::{
    collections::HashMap,
    sync::{Arc, atomic::AtomicUsize},
};

use crate::arena::handle::{Handle, WeakHandle};

pub mod handle;

struct ArenaObjectData<T> {
    object: T,
    generation: u32,
    strong_ref_count: Arc<AtomicUsize>,
}

impl<T> ArenaObjectData<T> {
    fn new(object: T) -> Self {
        Self {
            object,
            generation: 1,
            strong_ref_count: Arc::new(AtomicUsize::default()),
        }
    }
}

pub struct Arena<T> {
    id_counter: u32,
    objects: HashMap<u32, ArenaObjectData<T>>,
}

impl<T> Arena<T> {
    pub fn new() -> Self {
        Arena {
            id_counter: 1,
            objects: HashMap::new(),
        }
    }

    pub fn add(&mut self, value: T) -> Handle<T> {
        let object_data = ArenaObjectData::new(value);
        let ref_counter = object_data.strong_ref_count.clone();
        self.objects.insert(self.id_counter, object_data);
        self.id_counter += 1;
        Handle::new(self.id_counter - 1, 1, ref_counter)
    }

    pub fn get(&self, handle: Handle<T>) -> &T {
        let object_data = self
            .objects
            .get(&handle.id)
            .expect("Strong reference has to be valid!");
        &object_data.object
    }

    pub fn get_mut(&mut self, handle: Handle<T>) -> &mut T {
        let object_data = self
            .objects
            .get_mut(&handle.id)
            .expect("Strong reference has to be valid!");
        &mut object_data.object
    }

    pub fn upgrade(&mut self, weak_handle: WeakHandle<T>) -> Option<Handle<T>> {
        let object_data = self.objects.get_mut(&weak_handle.id)?;
        if object_data.generation != weak_handle.generation {
            None
        } else {
            Some(Handle::new(
                weak_handle.id,
                weak_handle.generation,
                object_data.strong_ref_count.clone(),
            ))
        }
    }
}

pub trait ArenaFor<T> {
    fn arena_for(&mut self) -> &mut Arena<T>;
}
