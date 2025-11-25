//! Types used to query entities from [`World`][crate::ecs::world::World].

use std::marker::PhantomData;

use crate::ecs::{
    entity::Entity,
    system::SystemInput,
    world::{WORLD, fetch::Fetch},
};

/// Stores the data needed to query entities from [`World`][crate::ecs::world::World].
pub struct Query<T: Fetch> {
    fetches: Vec<T::State>,
    entities: Vec<Vec<Entity>>,
}

impl<T: Fetch> SystemInput for Query<T> {
    fn borrow() -> Self {
        let world = WORLD.read().unwrap();
        let archetype_id = T::archetype_id().unwrap();
        let archetypes = world
            .archetypes
            .iter()
            .filter_map(|(id, arch)| {
                if id.contains(&archetype_id) {
                    Some(arch)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let fetches = archetypes
            .iter()
            .map(|arch| T::prepare(arch).unwrap())
            .collect();
        let entities = archetypes
            .iter()
            .map(|arch| arch.entities.clone())
            .collect();
        Query { fetches, entities }
    }
}

impl<T: Fetch> Query<T> {
    /// Creates an iterator over [`self`].
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

/// Iterator created from [`Query`]
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
    type Item = (Entity, T::Item<'a>);
    fn next(&mut self) -> Option<Self::Item> {
        if self.outside_idx >= self.outside_len {
            return None;
        }
        while self.inside_idx >= self.inside_len {
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
        Some((
            self.query.entities[self.outside_idx][inside_idx],
            T::get(state, inside_idx),
        ))
    }
}
