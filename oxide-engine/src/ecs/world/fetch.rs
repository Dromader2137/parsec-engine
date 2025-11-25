//! Trait used in the process of querying entities.

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

/// Represents a type that can be used to query entities from [`World`][`crate::ecs::world::World`].
/// It is automatically implemented for all types implementing [`Component`]
/// and all tuples containging up to 16 values that implement [`Fetch`].
pub trait Fetch: Sized {
    /// Type of elements returned when iterating over entites with a [`QueryIter`][`crate::ecs::world::query::QueryIter`].
    type Item<'a>
    where
        Self: 'a;
    /// Type that stores borrowing info
    type State: Clone;
    fn archetype_id() -> Result<ArchetypeId, ArchetypeError>;
    /// Creates the state used to later get specific entities.
    fn prepare(archetype: &Archetype) -> Result<Self::State, ArchetypeError>;
    /// Releases the lock on borrowed types.
    fn release(state: Self::State) -> Result<(), ArchetypeError>;
    /// Gets n-th element from the state.
    fn get<'a>(state: Self::State, row: usize) -> Self::Item<'a>;
    /// Gets the amount of entities stored in the state.
    fn len(state: &Self::State) -> usize;
}

/// Stores info about a non-mutable fetch.
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

/// Marks a type to be borrowed mutably inside a [`Query`][crate::ecs::world::query::Query].
pub struct Mut<T> {
    _marker: PhantomData<T>,
}

/// Stores info about a mutable fetch.
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
