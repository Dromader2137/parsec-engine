use std::{collections::HashMap, fmt::Debug};

use archetype::{Archetype, ArchetypeError, ArchetypeId};
use fetch::Fetch;
use query::Query;
use spawn::Spawn;

use crate::{ecs::world::{add_component::AddComponent, remove_component::RemoveComponent}, error::EngineError};

use super::entity::Entity;

pub mod archetype;
pub mod component;
pub mod fetch;
pub mod query;
pub mod spawn;
pub mod add_component;
pub mod remove_component;

#[derive(Debug, Clone, PartialEq)]
pub enum WorldError {
    ArchetypeError(ArchetypeError),
}

impl From<WorldError> for EngineError {
    fn from(value: WorldError) -> Self {
        EngineError::WorldError(value)
    }
}

/// World holds all data about components and entities
#[derive(Debug)]
pub struct World {
    archetypes: HashMap<ArchetypeId, Archetype>,
    current_id: u32,
}

impl World {
    pub fn new() -> World {
        World { archetypes: HashMap::new(), current_id: 1 }
    }

    fn get_archetype_mut(&mut self, archetype_id: &ArchetypeId) -> Result<&mut Archetype, WorldError> {
        if !self.archetypes.contains_key(archetype_id) {
            let mut archetype = Archetype::new();
            archetype.create_empty(archetype_id);
            self.archetypes.insert(archetype_id.clone(), archetype);
        }

        Ok(self.archetypes.get_mut(archetype_id).unwrap())
    }
    
    /// Spawn a new entity
    pub fn spawn<T: Spawn>(&mut self, bundle: T) -> Result<(), WorldError> {
        let archetype_id = T::archetype_id()?;
        let entity_id = self.current_id;
        let archetype = self.get_archetype_mut(&archetype_id)?;
        archetype.new_entity(entity_id);
        let result = bundle.spawn(archetype);
        archetype.trim_columns();
        result?;
        archetype.bundle_count += 1;
        self.current_id += 1;
        Ok(())
    }

    /// Query all entities containing component specified inside T
    pub fn query<'a, T: Fetch<'a>>(&'a self) -> Result<Query<'a, T>, WorldError> {
        Ok(Query::new(&self.archetypes)?)
    }

    /// Delete an entity
    pub fn delete(&mut self, entity: Entity) -> Result<(), WorldError> {
        for (_, archetype) in self.archetypes.iter_mut() {
            match archetype.delete_entity(entity) {
                Ok(()) => {
                    archetype.bundle_count -= 1;
                    return Ok(());
                }
                Err(ArchetypeError::EntityNotFound) => (),
                Err(err) => return Err(err.into())
            };
        }
        Err(ArchetypeError::EntityNotFound.into())
    }

    /// Add components to an already existing entity
    pub fn add_components<T: AddComponent>(&mut self, entity: Entity, bundle_extension: T) -> Result<(), WorldError> {
        let (archetype_id, old_archetype) = match self.archetypes.iter_mut().find(|(_, x)| x.check_entity(entity)) {
            Some(val) => val,
            None => return Err(ArchetypeError::EntityNotFound.into())
        };
        let new_archetype_id = archetype_id.merge_with(T::archetype_id()?)?;

        let (old_entity, map) = old_archetype.cut_entity(entity).expect("Correct entity cut");
        old_archetype.bundle_count -= 1;

        let new_archetype = self.get_archetype_mut(&new_archetype_id)?;

        for (type_id, data) in map.iter() {
            new_archetype.add_raw(*type_id, data.clone()).expect("Correct add raw");
        }
        let result = bundle_extension.add_to(new_archetype);
        new_archetype.trim_columns();
        result?;
        new_archetype.bundle_count += 1;
        new_archetype.moved_entity(old_entity);

        Ok(())
    }

    /// Remove components from an entity
    pub fn remove_components<T: RemoveComponent>(&mut self, entity: Entity) -> Result<(), WorldError> {
        let (archetype_id, old_archetype) = match self.archetypes.iter_mut().find(|(_, x)| x.check_entity(entity)) {
            Some(val) => val,
            None => return Err(ArchetypeError::EntityNotFound.into())
        };
        let new_archetype_id = archetype_id.remove_from(T::archetype_id()?)?;

        let (old_entity, map) = old_archetype.cut_entity(entity).expect("Correct entity cut");
        old_archetype.bundle_count -= 1;

        let new_archetype = self.get_archetype_mut(&new_archetype_id)?;

        for (type_id, data) in map.iter() {
            if new_archetype_id.contains_single(type_id) {
                new_archetype.add_raw(*type_id, data.clone()).expect("Correct add raw");
            }
        }
        new_archetype.trim_columns();
        new_archetype.bundle_count += 1;
        new_archetype.moved_entity(old_entity);

        Ok(())
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
    fn spawn_archetype_reuse_same_bundle_composition() {
        let mut world = World::new();
        world.spawn((Position(0, 0), Velocity(0, 0))).unwrap();
        let before = world.archetypes.len();

        world.spawn((Position(1, 1), Velocity(1, 1))).unwrap();
        assert_eq!(world.archetypes.len(), before);
    }

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
    fn spawn_query_delete_single_component_bundle() {
        let mut world = World::new();
        world.spawn(Position(1, 2)).unwrap();

        assert_eq!(world.archetypes.len(), 1);
        assert_eq!(world.archetypes.iter().next().unwrap().1.len(), 1);

        let mut query = world.query::<&[Position]>().unwrap();
        let (entity, _) = query.next().unwrap();
        drop(query);
        world.delete(entity).unwrap();
        
        assert_eq!(world.archetypes.len(), 1);
        assert_eq!(world.archetypes.iter().next().unwrap().1.len(), 0);

        let mut query = world.query::<&[Position]>().unwrap();

        assert_eq!(query.next(), None);
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
    fn spawn_query_delete_multi_component_bundle() {
        let mut world = World::new();
        world.spawn((Position(5, 6), Velocity(1, 1))).unwrap();

        let mut query = world.query::<(&[Position], &[Velocity])>().unwrap();
        let (entity, _) = query.next().unwrap();
        drop(query);
        world.delete(entity).unwrap();
        
        assert_eq!(world.archetypes.len(), 1);
        assert_eq!(world.archetypes.iter().next().unwrap().1.len(), 0);

        let mut query = world.query::<(&[Position], &[Velocity])>().unwrap();

        assert_eq!(query.next(), None);
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
    fn spawn_add_query_multi_component_bundle() {
        let mut world = World::new();
        world.spawn(Position(1, 2)).unwrap();

        let entity = {
            let mut query = world.query::<&[Position]>().unwrap();
            let (entity, pos) = query.next().unwrap();
            assert_eq!(pos, &Position(1, 2));
            entity
        };

        world.add_components(entity, Velocity(1, 3)).unwrap();

        let mut query = world.query::<(&[Position], &[Velocity])>().unwrap();
        let (_, components) = query.next().unwrap();
        assert_eq!(components, (&Position(1, 2), &Velocity(1, 3)));
    }
    
    #[test]
    fn spawn_remove_query_multi_component_bundle() {
        let mut world = World::new();
        world.spawn((Position(1, 2), Velocity(1, 3))).unwrap();

        let entity = {
            let mut query = world.query::<&[Position]>().unwrap();
            let (entity, pos) = query.next().unwrap();
            assert_eq!(pos, &Position(1, 2));
            entity
        };

        world.remove_components::<Position>(entity).unwrap();

        let mut query = world.query::<&[Velocity]>().unwrap();
        let (_, vel) = query.next().unwrap();
        assert_eq!(vel, &Velocity(1, 3));
    }
    
    #[test]
    fn spawn_add_remove_query_multi_component_bundle() {
        let mut world = World::new();
        world.spawn(Position(1, 2)).unwrap();

        let entity = {
            let mut query = world.query::<&[Position]>().unwrap();
            let (entity, pos) = query.next().unwrap();
            assert_eq!(pos, &Position(1, 2));
            entity
        };

        world.add_components(entity, Velocity(1, 3)).unwrap();

        let entity = {
            let mut query = world.query::<(&[Position], &[Velocity])>().unwrap();
            let (entity, components) = query.next().unwrap();
            assert_eq!(components, (&Position(1, 2), &Velocity(1, 3)));
            entity
        };

        world.remove_components::<Position>(entity).unwrap();

        let mut query = world.query::<&[Velocity]>().unwrap();
        let (_, vel) = query.next().unwrap();
        assert_eq!(vel, &Velocity(1, 3));
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

