use std::{
    any::TypeId,
    marker::PhantomData,
    sync::{Arc, RwLock},
};

use oxide_engine_macros::{impl_fetch, multiple_tuples};

use crate::ecs::world::{
    archetype::{Archetype, ArchetypeError, ArchetypeId, BorrowingStats},
    component::Component,
};

pub trait Fetch: Sized {
    type Item<'a>
    where
        Self: 'a;
    type State: Clone;
    fn archetype_id() -> Result<ArchetypeId, ArchetypeError>;
    fn prepare(archetype: &Archetype) -> Result<Self::State, ArchetypeError>;
    fn release(state: Self::State) -> Result<(), ArchetypeError>;
    fn get<'a>(state: Self::State, row: usize) -> Self::Item<'a>;
    fn len(state: &Self::State) -> usize;
}

#[derive(Debug, Clone)]
pub struct FetchState<T> {
    ptr: *const [T],
    len: usize,
    access: Arc<RwLock<BorrowingStats>>,
}

impl<T: Component> Fetch for T {
    type Item<'a>
        = &'a T
    where
        Self: 'a;
    type State = FetchState<T>;

    fn archetype_id() -> Result<ArchetypeId, ArchetypeError> {
        ArchetypeId::new(vec![TypeId::of::<T>()])
    }

    fn prepare(archetype: &Archetype) -> Result<Self::State, ArchetypeError> {
        let (ptr, access, len) = archetype.get()?;
        Ok(FetchState { ptr, access, len })
    }

    fn release(state: Self::State) -> Result<(), ArchetypeError> {
        let mut lock = state.access.write().unwrap();
        lock.release_lock();
        Ok(())
    }

    fn get<'a>(state: Self::State, row: usize) -> Self::Item<'a> {
        let ptr = state.ptr;
        let array = unsafe { &*ptr };
        &array[row]
    }

    fn len(state: &Self::State) -> usize { state.len }
}

pub struct Mut<T> {
    _marker: PhantomData<T>,
}

#[derive(Debug, Clone)]
pub struct FetchMutState<T> {
    ptr: *mut [T],
    len: usize,
    access: Arc<RwLock<BorrowingStats>>,
}

impl<T: Component> Fetch for Mut<T> {
    type Item<'a>
        = &'a mut T
    where
        Self: 'a;
    type State = FetchMutState<T>;

    fn archetype_id() -> Result<ArchetypeId, ArchetypeError> {
        ArchetypeId::new(vec![TypeId::of::<T>()])
    }

    fn prepare(archetype: &Archetype) -> Result<Self::State, ArchetypeError> {
        let (ptr, access, len) = archetype.get_mut()?;
        Ok(FetchMutState { ptr, access, len })
    }

    fn release(state: Self::State) -> Result<(), ArchetypeError> {
        let mut lock = state.access.write().unwrap();
        lock.release_lock();
        Ok(())
    }

    fn get<'a>(state: Self::State, row: usize) -> Self::Item<'a> {
        let ptr = state.ptr;
        let array = unsafe { &mut *ptr };
        &mut array[row]
    }

    fn len(state: &Self::State) -> usize { state.len }
}

multiple_tuples!(impl_fetch, 16);
