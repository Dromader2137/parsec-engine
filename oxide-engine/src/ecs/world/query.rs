use std::marker::PhantomData;

use super::{archetype::Archetype, fetch::Fetch};

pub trait QueryColumn {
    fn get(&self, idx: usize);
}

#[derive(Debug)]
pub struct Query<'a, T: Fetch<'a>> {
    outside_len: usize,
    inside_len: usize,
    outside_idx: usize,
    inside_idx: usize,
    fetch: Vec<T>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Fetch<'a>> Query<'a, T> {
    pub fn new(archetypes: &'a [Archetype]) -> Query<'a, T> {
        let mut fetch = Vec::new();
        for archetype in archetypes.iter() {
            fetch.push(T::borrow(archetype));
        }

        let inside_len = match fetch.first() {
            Some(val) => val.count(),
            None => 0,
        };

        Query { 
            outside_len: fetch.len(), 
            inside_len, 
            outside_idx: 0, 
            inside_idx: 0, 
            fetch, 
            _marker: PhantomData::default() 
        }
    }

    pub fn empty() -> Query<'a, T> {
        Query {
            outside_len: 0,
            inside_len: 0,
            outside_idx: 0,
            inside_idx: 0,
            fetch: Vec::new(),
            _marker: PhantomData::<&'a T>::default(),
        }
    }
}

impl<'a, T: Fetch<'a>> Iterator for Query<'a, T> {
    type Item = T::Item;
    fn next(&mut self) -> Option<Self::Item> {
        if self.inside_idx >= self.inside_len {
            self.inside_idx = 0;
            self.outside_idx += 1;
        }

        if self.outside_idx >= self.outside_len {
            return None;
        }

        self.inside_len = self.fetch[self.outside_idx].count();
        self.inside_idx += 1;
        Some(self.fetch[self.outside_idx].get(self.inside_idx - 1))
    }
}
