use std::collections::HashMap;

use crate::ecs::{
    entity::Entity,
    world::{
        archetype::{Archetype, ArchetypeError, ArchetypeId},
        fetch::Fetch,
    },
};

#[derive(Debug)]
pub struct Query<'a, T: Fetch<'a>> {
    outside_len: usize,
    inside_len: usize,
    outside_idx: usize,
    inside_idx: usize,
    released: bool,
    fetch: Vec<T>,
    archetypes: Vec<&'a Archetype>,
}

impl<'a, T: Fetch<'a>> Query<'a, T> {
    pub fn new(
        archetype_map: &'a HashMap<ArchetypeId, Archetype>,
    ) -> Result<Query<'a, T>, ArchetypeError> {
        let mut fetch = Vec::new();
        let mut archetypes = Vec::new();
        let t_archetype_id = T::archetype_id()?;

        for (archetype_id, archetype) in archetype_map.iter() {
            if archetype_id.contains(&t_archetype_id) {
                archetypes.push(archetype);
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
            released: false,
            fetch,
            archetypes,
        })
    }

    fn release_lock(&mut self) -> Result<(), ArchetypeError> {
        for archetype in self.archetypes.iter() {
            T::release(archetype)?;
        }
        self.released = true;
        Ok(())
    }
}

impl<'a, T: Fetch<'a>> Drop for Query<'a, T> {
    fn drop(&mut self) {
        if !self.released {
            self.release_lock().expect("Clean lock release");
        }
    }
}

pub trait QueryIter {
    type Item<'b>
    where
        Self: 'b;
    fn next<'b>(&'b mut self) -> Option<Self::Item<'b>>;
}

impl<'a, T: Fetch<'a>> QueryIter for Query<'a, T> {
    type Item<'b>
        = (Entity, T::Item<'b>)
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

        Some((
            self.archetypes[self.outside_idx].entities[row],
            self.fetch[self.outside_idx].get(row),
        ))
    }
}
