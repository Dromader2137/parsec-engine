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
    current_id: u32,
}

impl World {
    pub fn new() -> World {
        World { archetypes: HashMap::new(), current_id: 1 }
    }

    pub fn spawn<T: Spawn>(&mut self, bundle: T) -> Result<(), WorldError> {
        let archetype_id = T::archetype_id()?;
        let component_count = archetype_id.component_count();

        let archetype = self.archetypes.entry(archetype_id).or_insert(Archetype::new(component_count));
        archetype.new_entity(self.current_id);
        let result = bundle.spawn(archetype);
        archetype.trim_columns();
        result?;
        archetype.bundle_count += 1;
        self.current_id += 1;
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
    use std::collections::HashSet;

    use crate::ecs::world::query::QueryIter;
    use super::{*, component::Component};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
    struct Position(i32, i32);

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
    struct Velocity(i32, i32);

    #[test]
    fn spawn_query_single_component_bundle() {
        let mut world = World::new();
        world.spawn(Position(1, 2)).unwrap();

        assert_eq!(world.archetypes.len(), 1);
        assert_eq!(world.archetypes.iter().next().unwrap().1.len(), 1);

        let mut query = world.query::<&[Position]>().unwrap();
        let (_, pos) = query.next().unwrap();
        assert_eq!(pos, &Position(1, 2));
    }

    #[test]
    fn spawn_query_nested_multi_component_bundle() {
        let mut world = World::new();
        world.spawn(Position(1, 2)).unwrap();
        world.spawn((Position(25, 6), Velocity(3, 1), "asdfsaf", 1_u32)).unwrap();
        world.spawn((Position(15, 6), Velocity(30, 1), "alfsaf", 2_f32)).unwrap();
        world.spawn((Position(35, 6), Velocity(3, 1), "aaf", 3_f64)).unwrap();

        let mut query = world.query::<(((&[Position], &[Velocity]), &[&'static str]), &[u32])>().unwrap();
        let (_, pos) = query.next().unwrap();
        assert_eq!(pos, (((&Position(25, 6), &Velocity(3, 1)), &"asdfsaf"), &1_u32));
    }
    
    #[test]
    fn spawn_nested_query_multi_component_bundle() {
        let mut world = World::new();
        world.spawn((Position(15, 6), Velocity(1, 1))).unwrap();
        world.spawn((Position(15, 6), (Velocity(2, 1), 0_u32))).unwrap();
        world.spawn(((Position(25, 6), Velocity(3, 1)), ("asdfsaf", 1_f32))).unwrap();
        world.spawn(Position(35, 6)).unwrap();

        let mut expected = HashSet::from([
            (Position(15, 6), Velocity(1, 1)),
            (Position(15, 6), Velocity(2, 1)),
            (Position(25, 6), Velocity(3, 1))
        ]);

        let mut query = world.query::<(&[Position], &[Velocity])>().unwrap();

        while let Some((_, val)) = query.next() {
            let find = expected.get(&(*val.0, *val.1));
            assert!(matches!(find, Some(_)));
            let find_un = *find.unwrap();
            expected.remove(&find_un);
        }
    }

    #[test]
    fn spawn_query_multi_component_bundle() {
        let mut world = World::new();
        world.spawn((Position(5, 6), Velocity(1, 1))).unwrap();

        let mut query = world.query::<(&[Position], &[Velocity])>().unwrap();
        let (_, (pos, vel)) = query.next().unwrap();
        assert_eq!(pos, &Position(5, 6));
        assert_eq!(vel, &Velocity(1, 1));
    }

    #[test]
    fn spawn_query_multi_component_multi_bundle() {
        let mut world = World::new();
        world.spawn((Position(15, 6), Velocity(1, 1))).unwrap();
        world.spawn((Position(15, 6), Velocity(2, 1), 0_u32)).unwrap();
        world.spawn((Position(25, 6), Velocity(3, 1), "asdfsaf", 1_f32)).unwrap();
        world.spawn(Position(35, 6)).unwrap();

        let mut expected = HashSet::from([
            (Position(15, 6), Velocity(1, 1)),
            (Position(15, 6), Velocity(2, 1)),
            (Position(25, 6), Velocity(3, 1))
        ]);

        let mut query = world.query::<(&[Position], &[Velocity])>().unwrap();

        while let Some((_, val)) = query.next() {
            let find = expected.get(&(*val.0, *val.1));
            assert!(matches!(find, Some(_)));
            let find_un = *find.unwrap();
            expected.remove(&find_un);
        }
    }

    #[test]
    fn spawn_archetype_reuse_same_bundle_composition() {
        let mut world = World::new();
        world.spawn((Position(0, 0), Velocity(0, 0))).unwrap();
        let before = world.archetypes.len();

        world.spawn((Position(1, 1), Velocity(1, 1))).unwrap();
        assert_eq!(world.archetypes.len(), before);
    }

    #[test] 
    fn query_mut_component_update() {
        let mut world = World::new();
        world.spawn((Position(0, 0), Velocity(2, 3))).unwrap();

        {
            let mut query = world.query::<(&[Position], &mut [Velocity])>().unwrap();
            while let Some((_, (_,  vel))) = query.next() {
                vel.0 += 10;
                vel.1 += 20;
            }
        }

        let mut query = world.query::<(&[Position], &[Velocity])>().unwrap();
        let (_, (_, vel)) = query.next().unwrap();
        assert_eq!(vel, &Velocity(12, 23));
    }
}

