use std::{collections::HashMap, marker::PhantomData};

use crate::ecs::{
    entity::Entity, system::SystemInput, world::{
        WORLD, World, archetype::{Archetype, ArchetypeError, ArchetypeId}, fetch::Fetch
    }
};

pub struct Query<T>
where 
    T: for<'a> Fetch<'a>
{
    archetype_ids: Vec<ArchetypeId>,
    fetch: PhantomData<T>
}

pub struct QueryIter<'a, T: Fetch<'a>> {
    outside_len: usize,
    inside_len: usize,
    outside_idx: usize,
    inside_idx: usize,
    released: bool,
    fetch: Vec<T>,
    archetypes: Vec<&'a Archetype>,
}

#[derive(Debug)]
pub struct OldQuery<'a, T: Fetch<'a>> {
    outside_len: usize,
    inside_len: usize,
    outside_idx: usize,
    inside_idx: usize,
    released: bool,
    fetch: Vec<T>,
    archetypes: Vec<&'a Archetype>,
}

impl<'a, T: Fetch<'a>> OldQuery<'a, T> {
    pub fn new(
        archetype_map: &'a HashMap<ArchetypeId, Archetype>,
    ) -> Result<OldQuery<'a, T>, ArchetypeError> {
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

        Ok(OldQuery {
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

impl<'a, T: Fetch<'a>> Drop for OldQuery<'a, T> {
    fn drop(&mut self) {
        if !self.released {
            self.release_lock().expect("Clean lock release");
        }
    }
}

pub trait OldQueryIter {
    type Item<'b>
    where
        Self: 'b;
    fn next<'b>(&'b mut self) -> Option<Self::Item<'b>>;
}

impl<'a, T: Fetch<'a>> OldQueryIter for OldQuery<'a, T> {
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
