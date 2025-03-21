use std::{any::Any, fmt::Debug};

use archetype::Archetype;
use bundle::{Bundle, IntoArchetype};

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

    pub fn spawn<T: Bundle + IntoArchetype + 'static>(&mut self, bundle: T) -> Result<u32, WorldError> {
        let archetype = T::archetype()?;
        let archetype_count = self.archetypes.len();

        let archetype_id = match self
            .archetypes
            .iter()
            .enumerate()
            .find(|(_, x)| archetype == **x)
        {
            Some(val) => val.0,
            None => archetype_count,
        };

        if archetype_id == archetype_count {
            self.archetypes.push(archetype);
        }

        let id = self.entities[archetype_id].add(Box::new(bundle))?;

        Ok(id)
    }

    pub fn get<T: Bundle + IntoArchetype>(&self) -> Result<Vec<&T>, WorldError> {
        let bundle_archetype = T::archetype()?;

        let mut ret_vec = vec![];

        for (archetype_id, archetype) in self.archetypes.iter().enumerate() {
            if !archetype.contains(&bundle_archetype) {
                continue;
            }
            println!("{:?}", archetype.mapping(&bundle_archetype));
            let iter = self.entities[archetype_id].get();
            ret_vec.extend(iter);
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
        assert_eq!(vec![&(1.0, "abc"), &(1.0, "bcd")], ret);
    }
}
