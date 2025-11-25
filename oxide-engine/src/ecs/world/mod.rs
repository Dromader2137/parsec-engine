use std::{collections::HashMap, fmt::Debug, sync::RwLock};

use archetype::{Archetype, ArchetypeError, ArchetypeId};
use once_cell::sync::Lazy;
use spawn::Spawn;

use crate::{
    ecs::{
        entity::Entity,
        world::{add_component::AddComponent, remove_component::RemoveComponent},
    },
    error::EngineError,
};

pub mod add_component;
pub mod archetype;
pub mod component;
pub mod fetch;
pub mod query;
pub mod remove_component;
pub mod spawn;

#[derive(Debug, Clone, PartialEq)]
pub enum WorldError {
    ArchetypeError(ArchetypeError),
}

impl From<WorldError> for EngineError {
    fn from(value: WorldError) -> Self { EngineError::WorldError(value) }
}

/// Global store containing the main instance of [`World`].
pub static WORLD: Lazy<RwLock<World>> = Lazy::new(|| {
    RwLock::new(World {
        archetypes: HashMap::new(),
        current_id: 0,
    })
});

/// Holds all data about components and entities.
#[derive(Debug)]
pub struct World {
    /// Contains all archetypes indexed by their id.
    archetypes: HashMap<ArchetypeId, Archetype>,
    /// New entity id counter.
    current_id: u32,
}

impl World {
    /// Returns a mutable reference to the archetype stored under `archetype_id`.
    /// If there was no such archetype, it is created, added to `self` and a mutable reference is returned.
    fn get_archetype_mut(
        &mut self,
        archetype_id: &ArchetypeId,
    ) -> Result<&mut Archetype, WorldError> {
        if !self.archetypes.contains_key(archetype_id) {
            let mut archetype = Archetype::new();
            archetype.create_empty(archetype_id);
            self.archetypes.insert(archetype_id.clone(), archetype);
        }

        Ok(self.archetypes.get_mut(archetype_id).unwrap())
    }

    /// Spawns a new entity.
    pub fn spawn<T: Spawn>(bundle: T) -> Result<(), WorldError> {
        let mut world = WORLD.write().unwrap();
        let archetype_id = T::archetype_id()?;
        let entity_id = world.current_id;
        let archetype = world.get_archetype_mut(&archetype_id)?;
        archetype.new_entity(entity_id);
        let result = bundle.spawn(archetype);
        archetype.trim_columns();
        result?;
        archetype.bundle_count += 1;
        world.current_id += 1;
        Ok(())
    }

    /// Deletes the given entity.
    pub fn delete(entity: Entity) -> Result<(), WorldError> {
        let mut world = WORLD.write().unwrap();
        for (_, archetype) in world.archetypes.iter_mut() {
            match archetype.delete_entity(entity) {
                Ok(()) => {
                    archetype.bundle_count -= 1;
                    return Ok(());
                },
                Err(ArchetypeError::EntityNotFound) => (),
                Err(err) => return Err(err.into()),
            };
        }
        Err(ArchetypeError::EntityNotFound.into())
    }

    /// Add components to an already existing entity.
    pub fn add_components<T: AddComponent>(
        entity: Entity,
        bundle_extension: T,
    ) -> Result<(), WorldError> {
        let mut world = WORLD.write().unwrap();
        let (archetype_id, old_archetype) = match world
            .archetypes
            .iter_mut()
            .find(|(_, x)| x.check_entity(entity))
        {
            Some(val) => val,
            None => return Err(ArchetypeError::EntityNotFound.into()),
        };
        let new_archetype_id = archetype_id.merge_with(T::archetype_id()?)?;

        let (old_entity, map) = old_archetype
            .cut_entity(entity)
            .expect("Correct entity cut");
        old_archetype.bundle_count -= 1;

        let new_archetype = world.get_archetype_mut(&new_archetype_id)?;

        for (type_id, data) in map.iter() {
            new_archetype
                .add_raw(*type_id, data.clone())
                .expect("Correct add raw");
        }
        let result = bundle_extension.add_to(new_archetype);
        new_archetype.trim_columns();
        result?;
        new_archetype.bundle_count += 1;
        new_archetype.moved_entity(old_entity);

        Ok(())
    }

    /// Removes components from an entity.
    pub fn remove_components<T: RemoveComponent>(entity: Entity) -> Result<(), WorldError> {
        let mut world = WORLD.write().unwrap();
        let (archetype_id, old_archetype) = match world
            .archetypes
            .iter_mut()
            .find(|(_, x)| x.check_entity(entity))
        {
            Some(val) => val,
            None => return Err(ArchetypeError::EntityNotFound.into()),
        };
        let new_archetype_id = archetype_id.remove_from(T::archetype_id()?)?;

        let (old_entity, map) = old_archetype
            .cut_entity(entity)
            .expect("Correct entity cut");
        old_archetype.bundle_count -= 1;

        let new_archetype = world.get_archetype_mut(&new_archetype_id)?;

        for (type_id, data) in map.iter() {
            if new_archetype_id.contains_single(type_id) {
                new_archetype
                    .add_raw(*type_id, data.clone())
                    .expect("Correct add raw");
            }
        }
        new_archetype.trim_columns();
        new_archetype.bundle_count += 1;
        new_archetype.moved_entity(old_entity);

        Ok(())
    }
}
