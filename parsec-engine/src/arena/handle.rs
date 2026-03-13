use std::marker::PhantomData;

use crate::arena::ArenaFor;

#[derive(Debug)]
pub struct Handle<T> {
    pub(super) id: u32,
    pub(super) generation: u32,
    _marker: PhantomData<T>,
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Handle::new(self.id, self.generation)    
    }
}   

impl<T> Copy for Handle<T> {}

pub struct WeakHandle<T>(pub(super) Handle<T>);

impl<T> Handle<T> {
    pub(super) fn new(id: u32, generation: u32) -> Self {
        Handle {
            id,
            generation,
            _marker: PhantomData::default(),
        }
    }

    pub fn id(&self) -> u32 { self.id }

    pub fn generation(&self) -> u32 { self.generation }
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
