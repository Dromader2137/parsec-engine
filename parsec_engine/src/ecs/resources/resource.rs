use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use crate::{
    ecs::resources::ResourceMarker,
    utils::borrowing::{AccessError, BorrowingStats},
};

pub struct Resource<'a, R: ResourceMarker> {
    pub(super) value: *const R,
    pub(super) borrowing: Arc<BorrowingStats>,
    _marker: PhantomData<&'a R>,
}

impl<'a, R: ResourceMarker> Resource<'a, R> {
    pub fn new(
        value: &R,
        borrowing: Arc<BorrowingStats>,
    ) -> Result<Self, AccessError> {
        borrowing.borrow()?;
        Ok(Self {
            value: value as *const R,
            borrowing,
            _marker: PhantomData,
        })
    }
}

pub struct ResourceMut<'a, R: ResourceMarker> {
    pub(super) value: *mut R,
    pub(super) borrowing: Arc<BorrowingStats>,
    _marker: PhantomData<&'a R>,
}

impl<'a, R: ResourceMarker> ResourceMut<'a, R> {
    pub fn new(value: &R, borrowing: Arc<BorrowingStats>) -> Result<Self, AccessError> {
        borrowing.borrow_mut()?;
        Ok(Self {
            value: value as *const R as *mut R,
            borrowing,
            _marker: PhantomData,
        })
    }
}

impl<'a, R: ResourceMarker> Deref for Resource<'a, R> {
    type Target = R;

    fn deref(&self) -> &Self::Target {
        // SAFETY:
        // - Resource is !Send + !Sync so there is no risk regarding
        // the raw pointer being passed between threads.
        // - BorrowingStats track how many resources of a certain type exist
        // and what is their mutability to make sure aliasin rules are upheld.
        unsafe { &*self.value }
    }
}

impl<'a, R: ResourceMarker> Deref for ResourceMut<'a, R> {
    type Target = R;

    fn deref(&self) -> &Self::Target {
        // SAFETY:
        // - Resource is !Send + !Sync so there is no risk regarding
        // the raw pointer being passed between threads.
        // - BorrowingStats track how many resources of a certain type exist
        // and what is their mutability to make sure aliasin rules are upheld.
        unsafe { &*self.value }
    }
}

impl<'a, R: ResourceMarker> DerefMut for ResourceMut<'a, R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY:
        // - Resource is !Send + !Sync so there is no risk regarding
        // the raw pointer being passed between threads.
        // - BorrowingStats track how many resources of a certain type exist
        // and what is their mutability to make sure aliasin rules are upheld.
        unsafe { &mut *self.value }
    }
}

impl<'a, R: ResourceMarker> Drop for Resource<'a, R> {
    fn drop(&mut self) { self.borrowing.free(); }
}

impl<'a, R: ResourceMarker> Drop for ResourceMut<'a, R> {
    fn drop(&mut self) { self.borrowing.free(); }
}
