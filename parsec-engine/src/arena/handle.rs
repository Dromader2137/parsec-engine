use std::{
    marker::PhantomData,
    sync::{Arc, atomic::AtomicUsize},
};

use crate::arena::ArenaFor;

#[derive(Debug)]
pub struct Handle<T> {
    pub(super) id: u32,
    pub(super) generation: u32,
    pub(super) strong_ref_counter: Arc<AtomicUsize>,
    _marker: PhantomData<T>,
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Handle::new(self.id, self.generation, self.strong_ref_counter.clone())
    }
}

pub struct WeakHandle<T> {
    pub(super) id: u32,
    pub(super) generation: u32,
    _marker: PhantomData<T>,
}

impl<T> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
         self.id == other.id && self.generation == other.generation
     } 
}

impl<T> PartialEq for WeakHandle<T> {
    fn eq(&self, other: &Self) -> bool {
         self.id == other.id && self.generation == other.generation
     } 
}

impl<T> Handle<T> {
    pub(super) fn new(
        id: u32,
        generation: u32,
        strong_ref_counter: Arc<AtomicUsize>,
    ) -> Self {
        Handle {
            id,
            generation,
            strong_ref_counter,
            _marker: PhantomData::default(),
        }
    }

    pub fn id(&self) -> u32 { self.id }

    pub fn generation(&self) -> u32 { self.generation }

    pub fn downgrade(self) -> WeakHandle<T> {
        WeakHandle {
            id: self.id,
            generation: self.generation,
            _marker: PhantomData::default(),
        }
    }
}

impl<T> WeakHandle<T> {
    pub fn upgrade(
        self,
        arena_for: &mut impl ArenaFor<T>,
    ) -> Option<Handle<T>> {
        let arena = arena_for.arena_for();
        arena.upgrade(self)
    }
}
