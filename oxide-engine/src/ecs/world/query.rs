use std::{collections::HashMap, marker::PhantomData};

use super::{
    archetype::{Archetype, ArchetypeError, ArchetypeId},
    fetch::Fetch,
};

#[derive(Debug)]
pub struct Query<'a, T: Fetch<'a>> {
    outside_len: usize,
    inside_len: usize,
    outside_idx: usize,
    inside_idx: usize,
    released: bool,
    fetch: Vec<T>,
    archetypes: &'a HashMap<ArchetypeId, Archetype>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Fetch<'a>> Query<'a, T> {
    pub fn new(archetypes: &'a HashMap<ArchetypeId, Archetype>) -> Result<Query<'a, T>, ArchetypeError> {
        let mut fetch = Vec::new();
        let t_archetype_id = T::archetype_id()?;

        for (archetype_id, archetype) in archetypes.iter() {
            if archetype_id.contains(&t_archetype_id) {
                fetch.push(T::borrow(archetype)?);
            }
        }

        let inside_len = match fetch.first() {
            Some(val) => val.count(),
            None => 0,
        };

        Ok(Query {
            outside_len: fetch.len(),
            inside_len,
            outside_idx: 0,
            inside_idx: 0,
            fetch,
            released: false,
            archetypes,
            _marker: PhantomData::default(),
        })
    }

    fn release_lock(&mut self) -> Result<(), ArchetypeError> {
        for (_archetype_id, archetype) in self.archetypes.iter() {
            T::release(archetype)?;
        }
        self.released = true;
        Ok(())
    }
}

impl<'a, T: Fetch<'a>> Drop for Query<'a, T> {
    fn drop(&mut self) {
        self.release_lock().unwrap();
    }
}

pub trait QueryIter {
    type Item<'b>
    where
        Self: 'b;
    fn next<'b>(&'b mut self) -> Option<Self::Item<'b>>;
}

impl<'a, T: Fetch<'a>> QueryIter for Query<'a, T> {
    type Item<'b> = T::Item<'b>
    where
        Self: 'b;

    fn next<'b>(&'b mut self) -> Option<Self::Item<'b>> {
        if self.released {
            return None;
        }

        if self.inside_idx >= self.inside_len {
            self.inside_idx = 0;
            self.outside_idx += 1;
        }

        if self.outside_idx >= self.outside_len {
            return None;
        }

        self.inside_len = self.fetch[self.outside_idx].count();
        let row = self.inside_idx;
        self.inside_idx += 1;

        Some(self.fetch[self.outside_idx].get(row))
    }
}
