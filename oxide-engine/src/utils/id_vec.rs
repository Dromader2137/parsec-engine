use std::slice::{Iter, IterMut};

pub struct IdVec<T> {
    vec: Vec<T>,
}

impl<T> IdVec<T> {
    pub fn new() -> IdVec<T> {
        IdVec { vec: Vec::new() }
    }
    
    pub fn push(&mut self, value: T) -> u32 {
        self.vec.push(value);
        self.vec.len() as u32 - 1
    }

    pub fn get(&self, id: u32) -> Option<&T> {
        if self.vec.len() as u32 <= id { return None }
        self.vec.get(id as usize)
    }
    
    pub fn get_mut(&mut self, id: u32) -> Option<&mut T> {
        if self.vec.len() as u32 <= id { return None }
        self.vec.get_mut(id as usize)
    }

    pub fn iter(&self) -> Iter<'_, T> {
        self.vec.iter()
    }
    
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        self.vec.iter_mut()
    }

    pub fn len(&self) -> u32 {
        self.vec.len() as u32
    }
}
