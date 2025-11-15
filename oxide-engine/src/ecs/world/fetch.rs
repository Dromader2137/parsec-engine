use std::any::TypeId;

use oxide_engine_macros::{impl_fetch, multiple_tuples};

use super::{
  archetype::{Archetype, ArchetypeError, ArchetypeId},
  component::Component,
};

pub trait Fetch<'a>: Sized {
  type Item<'b>
  where
    'a: 'b,
    Self: 'b;
  fn archetype_id() -> Result<ArchetypeId, ArchetypeError>;
  fn borrow(archetype: &'a Archetype) -> Result<Self, ArchetypeError>;
  fn release(archetype: &'a Archetype) -> Result<(), ArchetypeError>;
  fn get<'b>(&'b mut self, row: usize) -> Self::Item<'b>;
  fn count(&self) -> usize;
}

impl<'a, T: Component> Fetch<'a> for &'a [T] {
  type Item<'b>
    = &'b T
  where
    'a: 'b,
    Self: 'b;

  fn archetype_id() -> Result<ArchetypeId, ArchetypeError> {
    ArchetypeId::new(vec![TypeId::of::<T>()])
  }

  fn borrow(archetype: &'a Archetype) -> Result<Self, ArchetypeError> {
    archetype.get::<T>()
  }

  fn release(archetype: &'a Archetype) -> Result<(), ArchetypeError> {
    archetype.release_lock::<T>()
  }

  fn get<'b>(&'b mut self, row: usize) -> Self::Item<'b> {
    &self[row]
  }

  fn count(&self) -> usize {
    self.len()
  }
}

impl<'a, T: Component> Fetch<'a> for &'a mut [T] {
  type Item<'b>
    = &'b mut T
  where
    'a: 'b,
    Self: 'b;

  fn archetype_id() -> Result<ArchetypeId, ArchetypeError> {
    ArchetypeId::new(vec![TypeId::of::<T>()])
  }

  fn borrow(archetype: &'a Archetype) -> Result<Self, ArchetypeError> {
    archetype.get_mut::<T>()
  }

  fn release(archetype: &'a Archetype) -> Result<(), ArchetypeError> {
    archetype.release_lock::<T>()
  }

  fn get<'b>(&'b mut self, row: usize) -> Self::Item<'b> {
    let ptr = self.as_mut_ptr();
    unsafe { &mut *ptr.add(row) }
  }

  fn count(&self) -> usize {
    self.len()
  }
}

multiple_tuples!(impl_fetch, 16);
