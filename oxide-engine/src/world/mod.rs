use std::fmt::Debug;

use archetype::Archetype;
use bundle::{Bundle, FromColumns, IntoArchetypeId};

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
        World {
            archetypes: vec![],
        }
    }

    pub fn spawn<T: Bundle + IntoArchetypeId + 'static>(&mut self, bundle: T) -> Result<u32, WorldError> {
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

        Ok(self.archetypes[archetype_index].bundle_count)
    }

    pub fn get<T: Bundle + IntoArchetypeId + FromColumns>(&self) -> Result<Vec<T>, WorldError> {
        let bundle_archetype_id = T::archetype_id();

        let mut ret_vec = vec![];

        for archetype in self.archetypes.iter() {
            if !archetype.id.contains(&bundle_archetype_id) {
                continue;
            }

            ret_vec.extend_from_slice(T::from_columns(archetype).as_slice());
        }

        Ok(ret_vec)
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
    fn test_get() {
        let mut world = World::new();
        world.spawn((1.0_f32, "abc")).unwrap();
        world.spawn((1.0_f32, "bcd", 1_u8)).unwrap();
        let ret = world.get::<(f32, &'static str)>().unwrap();
        assert_eq!(vec![(1.0, "abc"), (1.0, "bcd")], ret);
    }
}
