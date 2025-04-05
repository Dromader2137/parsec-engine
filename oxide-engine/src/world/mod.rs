use std::fmt::Debug;

use archetype::Archetype;
use bundle::{FromColumns, UsableBundle};

pub mod archetype;
pub mod bundle;
pub mod entity;

#[derive(Debug, Clone, PartialEq)]
pub enum WorldError {
    SpawnError(String),
    SpawnTypeMismatch,
    DoubleTypeArchetypeNotAllowed,
    ArchetypeNotFound,
}

pub struct World {
    archetypes: Vec<Archetype>,
}

impl World {
    pub fn new() -> World {
        World { archetypes: vec![] }
    }

    pub fn spawn<T: for<'a> UsableBundle<'a>>(&mut self, bundle: T) -> Result<(), WorldError> {
        let archetype_id = T::archetype_id();
        let archetype_count = self.archetypes.len();

        let archetype_index = match self
            .archetypes
            .iter()
            .enumerate()
            .find(|(_, x)| archetype_id == x.id)
        {
            Some(val) => val.0,
            None => archetype_count,
        };

        if archetype_index == archetype_count {
            self.archetypes.push(Archetype::new(archetype_id));
        }

        bundle.add_to(&mut self.archetypes[archetype_index]);
        self.archetypes[archetype_index].bundle_count += 1;

        Ok(())
    }

    pub fn query<T: for<'a> UsableBundle<'a>>(
        &self,
    ) -> Result<impl Iterator<Item = <T as FromColumns<'_>>::Output>, WorldError> {
        let bundle_archetype_id = T::archetype_id();

        let mut iterators = Vec::new();

        for archetype in self.archetypes.iter() {
            if !archetype.id.contains(&bundle_archetype_id) {
                continue;
            }

            iterators.push(T::iter_from_columns(archetype));
        }

        Ok(iterators.into_iter().flatten())
    }

    pub fn query_mut<T: for<'a> UsableBundle<'a>>(
        &self,
    ) -> Result<impl Iterator<Item = <T as FromColumns<'_>>::Output>, WorldError> {
        let bundle_archetype_id = T::archetype_id();

        let mut iterators = Vec::new();

        for archetype in self.archetypes.iter() {
            if !archetype.id.contains(&bundle_archetype_id) {
                continue;
            }

            iterators.push(T::iter_from_columns(archetype));
        }

        Ok(iterators.into_iter().flatten())
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::World;

    #[test]
    fn test_get_1() {
        let mut world = World::new();
        world.spawn((1.0_f32, "abc")).unwrap();
        world.spawn((1.2_f32, "bcd", 1_u8)).unwrap();
        let mut ret = world.query::<(f32,)>().unwrap();
        assert_eq!(Some((&1.0,)), ret.next());
        assert_eq!(Some((&1.2,)), ret.next());
    }
}
