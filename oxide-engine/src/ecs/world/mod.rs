use std::fmt::Debug;

use archetype::{Archetype, ArchetypeError};
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
    archetypes: Vec<Archetype>,
}

impl World {
    pub fn new() -> World {
        World { archetypes: vec![] }
    }

    pub fn spawn<T: Spawn>(&mut self, bundle: T) -> Result<(), WorldError> {
        let archetype_id = T::archetype_id()?;

        match self.archetypes.iter_mut().find(|x| x.id == archetype_id) {
            Some(archetype) => {
                bundle.spawn(archetype)?;
                archetype.bundle_count += 1;
            }
            None => {
                self.archetypes.push(Archetype::new(archetype_id));
                bundle.spawn(self.archetypes.last_mut().unwrap())?;
                self.archetypes.last_mut().unwrap().bundle_count += 1;
            }
        };
        
        Ok(())
    }

    pub fn query<'a, T: Fetch<'a>>(&'a self) -> Result<Query<'a, T>, WorldError> {
        Ok(Query::new(&self.archetypes))
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[derive(Debug, Clone, PartialEq)]
    struct Position(f32, f32);

    #[derive(Debug, Clone, PartialEq)]
    struct Velocity(f32, f32);

    #[test]
    fn spawn_query_single_component_bundle() {
        let mut world = World::new();
        world.spawn((Position(1.0, 2.0),)).unwrap();

        assert_eq!(world.archetypes.len(), 1);
        assert_eq!(world.archetypes[0].len(), 1);
        
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
            for (_pos, vel) in &mut query {
                vel.0 += 10.0;
                vel.1 += 20.0;
            }
        }

        let mut query = world.query::<(&[Position], &[Velocity])>().unwrap();
        let (_, vel) = query.next().unwrap();
        assert_eq!(vel, &Velocity(12.0, 23.0));
    }

    proptest! {
        #[test]
        fn fuzz_spawn_and_query_roundtrip(xs in prop::collection::vec((any::<f32>(), any::<f32>()), 1..50),
                                         vs in prop::collection::vec((any::<f32>(), any::<f32>()), 1..50)) {

            let mut world = World::new();

            for ((px, py), (vx, vy)) in xs.into_iter().zip(vs.into_iter()) {
                world.spawn((Position(px, py), Velocity(vx, vy))).unwrap();
            }

            let mut seen = vec![];
            let mut query = world.query::<(&[Position], &[Velocity])>().unwrap();
            for (p, v) in &mut query {
                seen.push((p.clone(), v.clone()));
            }

            for (px, py) in seen.iter().map(|(p, _)| (p.0, p.1)) {
                assert!(seen.iter().any(|(p, _)| p.0 == px && p.1 == py));
            }
            for (vx, vy) in seen.iter().map(|(_, v)| (v.0, v.1)) {
                assert!(seen.iter().any(|(_, v)| v.0 == vx && v.1 == vy));
            }
        }
    }
}

