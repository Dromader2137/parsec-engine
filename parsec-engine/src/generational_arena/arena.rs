use std::{collections::HashMap, hash::Hash};

use crate::generational_arena::handle::{Handle, WeakHandle};

pub trait Arena<T, I = u32> {
    fn get(&self, handle: Handle<T, I>) -> &T;
    fn get_mut(&mut self, handle: Handle<T, I>) -> &mut T;
    fn get_weak(&self, handle: WeakHandle<T, I>) -> Option<&T>;
    fn get_weak_mut(&mut self, handle: WeakHandle<T, I>) -> Option<&mut T>;
}

pub struct StandardArena<T, I = u32> {
    objects: HashMap<I, (I, T)>
}

impl<T, I> StandardArena<T, I> {
    pub fn new() -> Self {
        Self { objects: HashMap::new() }
    }
}

impl<T, I: Eq + Hash + Copy> Arena<T, I> for StandardArena<T, I> {
    fn get(&self, handle: Handle<T, I>) -> &T {
        &self.objects.get(&handle.id())
            .expect("Stong handles are always valid").1
    }

    fn get_mut(&mut self, handle: Handle<T, I>) -> &mut T { 
        &mut self.objects.get_mut(&handle.id())
            .expect("Stong handles are always valid").1
    }

    fn get_weak(&self, handle: WeakHandle<T, I>) -> Option<&T> { 
        let ret = self.objects.get(&handle.id())?; 
        if handle.generation() == ret.0 {
            return Some(&ret.1);
        }
        None
    }

    fn get_weak_mut(&mut self, handle: WeakHandle<T, I>) -> Option<&mut T> {
        let ret = self.objects.get_mut(&handle.id())?; 
        if handle.generation() == ret.0 {
            return Some(&mut ret.1);
        }
        None
    }
}

pub trait ArenaFor<T, I = u32> {
    fn arena_for() -> impl Arena<T, I>;
}
