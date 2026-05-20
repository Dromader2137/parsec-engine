use std::{ops::{Deref, DerefMut}, sync::Arc};

use crate::{ecs::resources::ResourceMarker, utils::borrowing::BorrowingStats};

pub struct Resource<R: ResourceMarker> {
    pub(super) value: *const R,
    pub(super) borrowing: Arc<BorrowingStats>,
}

pub struct ResourceMut<R: ResourceMarker> {
    pub(super) value: *mut R,
    pub(super) borrowing: Arc<BorrowingStats>,
}

impl<'a, R: ResourceMarker> Deref for Resource<R> {
    type Target = R;

    fn deref(&self) -> &Self::Target { unsafe { &*self.value } }
}

impl<'a, R: ResourceMarker> Deref for ResourceMut<R> {
    type Target = R;

    fn deref(&self) -> &Self::Target { unsafe { &*self.value } }
}

impl<'a, R: ResourceMarker> DerefMut for ResourceMut<R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.value }
    }
}

impl<'a, R: ResourceMarker> Drop for Resource<R> {
    fn drop(&mut self) { self.borrowing.free(); }
}

impl<'a, R: ResourceMarker> Drop for ResourceMut<R> {
    fn drop(&mut self) { self.borrowing.free(); }
}
