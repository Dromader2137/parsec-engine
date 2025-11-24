use std::marker::PhantomData;

use crate::ecs::{
    system::SystemInput,
    world::{WORLD, fetch::Fetch},
};

pub struct Query<T: Fetch> {
    fetches: Vec<T::State>,
}

impl<T: Fetch> SystemInput for Query<T> {
    fn borrow() -> Self {
        let world = WORLD.read().unwrap();
        let archetype_id = T::archetype_id().unwrap();
        let archetypes = world.archetypes.iter().filter_map(|(id, arch)| {
            if id.contains(&archetype_id) {
                Some(arch)
            } else {
                None
            }
        });
        let fetches = archetypes.map(|arch| T::prepare(arch).unwrap()).collect();
        Query { fetches }
    }
}

impl<T: Fetch> Query<T> {
    pub fn into_iter<'a>(&'a self) -> QueryIter<'a, T> {
        let inside_len = match self.fetches.first() {
            Some(first_fetch) => T::len(&first_fetch),
            None => 0,
        };
        QueryIter {
            outside_len: self.fetches.len(),
            inside_len,
            outside_idx: 0,
            inside_idx: 0,
            query: &self,
            _marker: PhantomData::default(),
        }
    }
}

impl<T: Fetch> Drop for Query<T> {
    fn drop(&mut self) {
        for fetch in self.fetches.iter_mut() {
            T::release(fetch.clone()).unwrap();
        }
    }
}

#[derive(Clone)]
pub struct QueryIter<'a, T: Fetch + 'static> {
    outside_len: usize,
    inside_len: usize,
    outside_idx: usize,
    inside_idx: usize,
    query: &'a Query<T>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Fetch + 'static> Iterator for QueryIter<'a, T> {
    type Item = T::Item<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.outside_idx >= self.outside_len {
            return None;
        }
        if self.inside_idx >= self.inside_len {
            self.outside_idx += 1;
            if self.outside_idx >= self.outside_len {
                return None;
            }
            self.inside_idx = 0;
            self.inside_len = T::len(&self.query.fetches[self.outside_idx]);
        }
        let state = self.query.fetches[self.outside_idx].clone();
        let inside_idx = self.inside_idx;
        self.inside_idx += 1;
        Some(T::get(state, inside_idx))
    }
}
