use oxide_engine_macros::{multiple_tuples, impl_fetch};

use super::{archetype::Archetype, component::Component};

pub trait Fetch<'a> {
    type Item;
    fn borrow(archetype: &'a Archetype) -> Self;
    fn get(&mut self, row: usize) -> Self::Item;
    fn count(&self) -> usize;
}

impl<'a, T: Component> Fetch<'a> for &'a [T] {
    type Item = &'a T;

    fn borrow(archetype: &'a Archetype) -> Self {
        archetype.get::<T>().unwrap()
    }

    fn get(&mut self, row: usize) -> Self::Item {
        &self[row]
    }

    fn count(&self) -> usize {
        self.len()
    }
}

impl<'a, T: Component> Fetch<'a> for &'a mut [T] {
    type Item = &'a mut T;

    fn borrow(archetype: &'a Archetype) -> Self {
        archetype.get_mut::<T>().unwrap()
    }

    fn get(&mut self, row: usize) -> Self::Item {
        let ptr = self.as_mut_ptr();
        unsafe { &mut *ptr.add(row) }
    }
    
    fn count(&self) -> usize {
        self.len()
    }
}

multiple_tuples!(impl_fetch, 16);
