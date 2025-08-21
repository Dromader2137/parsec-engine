use std::{collections::HashMap, fmt::Debug};

use archetype::{Archetype, ArchetypeError, ArchetypeId};
use fetch::Fetch;
use query::Query;
use spawn::Spawn;

use crate::error::EngineError;

pub mod archetype;
pub mod component;
pub mod fetch;
pub mod query;
pub mod spawn;

#[derive(Debug, Clone, PartialEq)]
pub enum WorldError {
    ArchetypeError(ArchetypeError),
}

impl From<WorldError> for EngineError {
    fn from(value: WorldError) -> Self {
        EngineError::WorldError(value)
    }
}

#[derive(Debug)]
pub struct World {
    archetypes: HashMap<ArchetypeId, Archetype>,
}

impl World {
    pub fn new() -> World {
        World { archetypes: HashMap::new() }
    }

    pub fn spawn<T: Spawn>(&mut self, bundle: T) -> Result<(), WorldError> {
        let archetype_id = T::archetype_id()?;
        let component_count = archetype_id.component_count();

        let archetype = self.archetypes.entry(archetype_id).or_insert(Archetype::new(component_count));
        let result = bundle.spawn(archetype);
        archetype.trim_columns();
        result?;
        Ok(())
    }

    pub fn query<'a, T: Fetch<'a>>(&'a self) -> Result<Query<'a, T>, WorldError> {
        Ok(Query::new(&self.archetypes)?)
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::ecs::world::query::QueryIter;
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct Position(f32, f32);

    #[derive(Debug, Clone, PartialEq)]
    struct Velocity(f32, f32);

    #[test]
    fn spawn_query_single_component_bundle() {
        let mut world = World::new();
        world.spawn((Position(1.0, 2.0),)).unwrap();

        assert_eq!(world.archetypes.len(), 1);
        assert_eq!(world.archetypes.iter().next().unwrap().1.len(), 1);

        let mut query = world.query::<&[Position]>().unwrap();
        let pos = query.next().unwrap();
        assert_eq!(pos, &Position(1.0, 2.0));
    }

    #[test]
    fn spawn_query_multi_component_bundle() {
        let mut world = World::new();
        world.spawn((Position(5.0, 6.0), Velocity(1.0, 1.0))).unwrap();

        let mut query = world.query::<(&[Position], &[Velocity])>().unwrap();
        let (pos, vel) = query.next().unwrap();
        assert_eq!(pos, &Position(5.0, 6.0));
        assert_eq!(vel, &Velocity(1.0, 1.0));
    }

    #[test]
    fn spawn_query_multi_component_multi_bundle() {
        let mut world = World::new();
        world.spawn((Position(5.0, 6.0), Velocity(1.0, 1.0))).unwrap();
        world.spawn((Position(15.0, 6.0), Velocity(2.0, 1.0), 0_u32)).unwrap();
        world.spawn((Position(25.0, 6.0), Velocity(3.0, 1.0), "asdfsaf", 1.0_f32)).unwrap();
        world.spawn((Position(35.0, 6.0), )).unwrap();

        let mut query = world.query::<(&[Position], &[Velocity])>().unwrap();
        let (pos, vel) = query.next().unwrap();
        assert_eq!(pos, &Position(5.0, 6.0));
        assert_eq!(vel, &Velocity(1.0, 1.0));

        let (pos, vel) = query.next().unwrap();
        assert_eq!(pos, &Position(15.0, 6.0));
        assert_eq!(vel, &Velocity(2.0, 1.0));

        let (pos, vel) = query.next().unwrap();
        assert_eq!(pos, &Position(25.0, 6.0));
        assert_eq!(vel, &Velocity(3.0, 1.0));
        
        let (pos, vel) = query.next().unwrap();
        assert_eq!(pos, &Position(35.0, 6.0));
        assert_eq!(vel, &Velocity(3.0, 1.0));
    }

    #[test]
    fn spawn_archetype_reuse_same_bundle_composition() {
        let mut world = World::new();
        world.spawn((Position(0.0, 0.0), Velocity(0.0, 0.0))).unwrap();
        let before = world.archetypes.len();

        world.spawn((Position(1.0, 1.0), Velocity(1.0, 1.0))).unwrap();
        assert_eq!(world.archetypes.len(), before);
    }

    #[test] 
    fn query_mut_component_update() {
        let mut world = World::new();
        world.spawn((Position(0.0, 0.0), Velocity(2.0, 3.0))).unwrap();

        {
            let mut query = world.query::<(&[Position], &mut [Velocity])>().unwrap();
            while let Some((_,  vel)) = query.next() {
                vel.0 += 10.0;
                vel.1 += 20.0;
            }
        }

        let mut query = world.query::<(&[Position], &[Velocity])>().unwrap();
        let (_, vel) = query.next().unwrap();
        assert_eq!(vel, &Velocity(12.0, 23.0));
    }
}

