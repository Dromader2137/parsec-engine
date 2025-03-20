use std::{any::Any, fmt::Debug};

use archetype::Archetype;
use archetype_storage::ArchetypeStorageDyn;
use bundle::Bundle;

pub mod archetype;
pub mod archetype_storage;
pub mod bundle;
pub mod entity;

#[derive(Debug, Clone, PartialEq)]
pub enum WorldError {
    SpawnError(String),
    SpawnTypeMismatch,
    DoubleTypeArchetypeNotAllowed,
    ArchetypeNotFound
}

pub struct World {
    archetypes: Vec<Archetype>,
    entities: Vec<Box<dyn ArchetypeStorageDyn>>,
}

impl World {
    pub fn new() -> World {
        World {
            archetypes: vec![],
            entities: vec![],
        }
    }

    pub fn spawn<T: Bundle>(&mut self, bundle: T) -> Result<u32, WorldError> {
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
            self.entities.push(Box::new(Vec::<T>::new()));
        }

        let any_bundle: Box<dyn Any> = Box::new(bundle);
        let id = self.entities[archetype_id].add_dyn(any_bundle)?;

        Ok(id)
    }

    pub fn get<T: Bundle>(&self) -> Result<Vec<&T>, WorldError> {
        let bundle_archetype = T::archetype()?;

        let mut ret_vec = vec![];

        for (archetype_id, archetype) in self.archetypes.iter().enumerate() {
            if !archetype.contains(&bundle_archetype) { continue; }
            println!("{:?}", archetype.mapping(&bundle_archetype));
            let iter = self.entities[archetype_id].get_dyn()
                .into_iter()
                .filter_map(|x| x.downcast_ref::<T>());
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
        world.spawn((1.0_f32,"abc")).unwrap();
        world.spawn((1.0_f32,"bcd",1_u8)).unwrap();
        let ret = world.get::<(f32,&'static str)>().unwrap();
        assert_eq!(vec![&(1.0,"abc"), &(1.0, "bcd")], ret);
    }
}
