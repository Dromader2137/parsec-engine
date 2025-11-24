use std::{any::TypeId, marker::PhantomData, sync::{Arc, RwLock}};

use crate::ecs::world::{
    archetype::{Archetype, ArchetypeError, ArchetypeId, BorrowingStats},
    component::Component,
};

pub trait Fetch: Sized {
    type Item<'a> where Self: 'a;
    type Arr<'a> where Self: 'a;
    type State: Clone;
    fn archetype_id() -> Result<ArchetypeId, ArchetypeError>;
    fn prepare(archetype: &Archetype) -> Result<Self::State, ArchetypeError>;
    fn borrow<'a>(state: Self::State) -> Self::Arr<'a>;
    fn release(state: Self::State) -> Result<(), ArchetypeError>;
    fn get<'a>(array: &mut Self::Arr<'a>, row: usize) -> Self::Item<'a>;
}

#[derive(Debug, Clone)]
pub struct FetchState<T> {
    ptr: *const [T],
    access: Arc<RwLock<BorrowingStats>>
}

impl<T: Component> Fetch for T {
    type Item<'a> = &'a T where Self: 'a;
    type Arr<'a> = &'a [T] where Self: 'a;
    type State = FetchState<T>;

    fn archetype_id() -> Result<ArchetypeId, ArchetypeError> {
        ArchetypeId::new(vec![TypeId::of::<T>()])
    }

    fn prepare(archetype: &Archetype) -> Result<Self::State, ArchetypeError> {
        let (ptr, access) = archetype.get()?;
        Ok(FetchState { ptr, access })
    }

    fn borrow<'a>(state: Self::State) -> Self::Arr<'a> {
        let ptr = state.ptr;
        unsafe {
            &*ptr
        }
    }

    fn release(state: Self::State) -> Result<(), ArchetypeError> {
        let mut lock = state.access.write().unwrap();
        lock.release_lock();
        Ok(())
    }

    fn get<'a>(array: &mut Self::Arr<'a>, row: usize) -> Self::Item<'a> {
        &array[row]
    }
}

pub struct Mut<T> {
    _marker: PhantomData<T>
}

#[derive(Debug, Clone)]
pub struct FetchMutState<T> {
    ptr: *mut [T],
    access: Arc<RwLock<BorrowingStats>>
}

impl<T: Component> Fetch for Mut<T> {
    type Item<'a> = &'a mut T where Self: 'a;
    type Arr<'a> = &'a mut [T] where Self: 'a;
    type State = FetchMutState<T>;

    fn archetype_id() -> Result<ArchetypeId, ArchetypeError> {
        ArchetypeId::new(vec![TypeId::of::<T>()])
    }

    fn prepare(archetype: &Archetype) -> Result<Self::State, ArchetypeError> {
        let (ptr, access) = archetype.get_mut()?;
        Ok(FetchMutState { ptr, access })
    }

    fn borrow<'a>(state: Self::State) -> Self::Arr<'a> {
        let ptr = state.ptr;
        unsafe {
            &mut *ptr
        }
    }

    fn release(state: Self::State) -> Result<(), ArchetypeError> {
        let mut lock = state.access.write().unwrap();
        lock.release_lock();
        Ok(())
    }

    fn get<'a>(array: Self::Arr<'a>, row: usize) -> Self::Item<'a> {
        &mut array[row]
    }
}

// multiple_tuples!(impl_fetch, 16);
