//! Module responsible for storing and querying entities and their data.

use std::{collections::HashMap, fmt::Debug, sync::RwLock};

use archetype::{Archetype, ArchetypeError, ArchetypeId};
use once_cell::sync::Lazy;
use spawn::Spawn;
use thiserror::Error;

use crate::ecs::{
    entity::Entity,
    world::{add_component::AddComponent, remove_component::RemoveComponent},
};

mod add_component;
mod archetype;
pub mod component;
pub mod fetch;
pub mod query;
mod remove_component;
mod spawn;

#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldError {
    #[error("Failed to spawn an entity because of: {kind}")]
    SpawnError { kind: ArchetypeError },
    #[error("Failed to delete an entity because of: {kind}")]
    DeleteError { kind: ArchetypeError },
    #[error("Failed to add components because of: {kind}")]
    AddComponentError { kind: ArchetypeError },
    #[error("Failed to delete components because of: {kind}")]
    DeleteComponentError { kind: ArchetypeError },
}

/// Global store containing the main instance of [`World`].
pub static WORLD: Lazy<RwLock<World>> = Lazy::new(|| {
    RwLock::new(World {
        archetypes: HashMap::new(),
        current_id: 0,
    })
});

/// Stores all data about components and entities.
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
    ) -> &mut Archetype {
        if !self.archetypes.contains_key(archetype_id) {
            let mut archetype = Archetype::new();
            archetype.create_empty(archetype_id);
            self.archetypes.insert(archetype_id.clone(), archetype);
        }

        self.archetypes.get_mut(archetype_id).unwrap()
    }

    /// Spawns a new entity.
    ///
    /// # Errors
    ///
    /// - If `T` can't produce a valid [`ArchetypeId`].
    /// - If the [archetype][Archetype] is already borrowed in some way.
    pub fn spawn<T: Spawn>(bundle: T) -> Result<Entity, WorldError> {
        let mut world = WORLD.write().unwrap();
        let archetype_id = T::archetype_id()
            .map_err(|e| WorldError::SpawnError { kind: e })?;
        let entity_id = world.current_id;
        let archetype = world.get_archetype_mut(&archetype_id);
        let entity = archetype
            .new_entity(entity_id)
            .map_err(|e| WorldError::SpawnError { kind: e })?;
        bundle
            .spawn(archetype)
            .map_err(|e| WorldError::SpawnError { kind: e })?;
        archetype.bundle_count += 1;
        world.current_id += 1;
        Ok(entity)
    }

    /// Deletes the given entity.
    ///
    /// # Errors
    ///
    /// - If `entity` doesn't exist.
    /// - If the [archetype][Archetype] containing `entity` is already borrowed in some way.
    pub fn delete(entity: Entity) -> Result<(), WorldError> {
        let mut world = WORLD.write().unwrap();
        for (_, archetype) in world.archetypes.iter_mut() {
            if archetype.entities.contains(&entity)
                && !archetype.are_all_columns_mutable()
            {
                return Err(WorldError::DeleteError {
                    kind: ArchetypeError::ArchetypeColumnNotWritable,
                });
            }
            match archetype.delete_entity(entity) {
                Ok(()) => {
                    archetype.bundle_count -= 1;
                    return Ok(());
                },
                Err(ArchetypeError::EntityNotFound) => (),
                Err(err) => return Err(WorldError::DeleteError { kind: err }),
            };
        }
        Err(WorldError::DeleteError {
            kind: ArchetypeError::EntityNotFound,
        })
    }

    /// Add components to an already existing entity.
    ///
    /// # Errors
    ///
    /// - If `T` can't produce a valid [`ArchetypeId`].
    /// - If `T` merged with the type of `entity` can't produce a valid [`ArchetypeId`].
    /// - If either the original [archetype][Archetype] containing `entity` or the destination [archetype][Archetype] is already borrowed in some way.
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
            None => {
                return Err(WorldError::DeleteError {
                    kind: ArchetypeError::EntityNotFound,
                });
            },
        };
        let t_archetype_id = T::archetype_id()
            .map_err(|e| WorldError::AddComponentError { kind: e })?;
        let new_archetype_id = archetype_id
            .merge_with(t_archetype_id)
            .map_err(|e| WorldError::AddComponentError { kind: e })?;

        let (old_entity, map) = old_archetype
            .cut_entity(entity)
            .map_err(|e| WorldError::AddComponentError { kind: e })?;
        old_archetype.bundle_count -= 1;

        let new_archetype = world.get_archetype_mut(&new_archetype_id);

        if !new_archetype.are_all_columns_mutable() {
            let (_, old_archetype) = world
                .archetypes
                .iter_mut()
                .find(|(_, x)| x.check_entity(entity))
                .unwrap();
            for (type_id, (size, data)) in map.iter() {
                old_archetype
                    .add_raw(*size, *type_id, data.clone())
                    .map_err(|e| WorldError::AddComponentError { kind: e })?;
            }
            return Err(WorldError::AddComponentError {
                kind: ArchetypeError::ArchetypeColumnNotWritable,
            });
        }

        for (type_id, (size, data)) in map.iter() {
            new_archetype
                .add_raw(*size, *type_id, data.clone())
                .map_err(|e| WorldError::AddComponentError { kind: e })?;
        }
        bundle_extension
            .add_to(new_archetype)
            .map_err(|e| WorldError::AddComponentError { kind: e })?;
        new_archetype.bundle_count += 1;
        new_archetype.moved_entity(old_entity);

        Ok(())
    }

    /// Removes components from an entity.
    ///
    /// # Errors
    ///
    /// - If `T` can't produce a valid [`ArchetypeId`].
    /// - If `T` subtracted from the type of `entity` can't produce a valid [`ArchetypeId`].
    /// - If either the original [archetype][Archetype] containing `entity` or the destination [archetype][Archetype] is already borrowed in some way.
    pub fn remove_components<T: RemoveComponent>(
        entity: Entity,
    ) -> Result<(), WorldError> {
        let mut world = WORLD.write().unwrap();
        let (archetype_id, old_archetype) = match world
            .archetypes
            .iter_mut()
            .find(|(_, x)| x.check_entity(entity))
        {
            Some(val) => val,
            None => {
                return Err(WorldError::DeleteComponentError {
                    kind: ArchetypeError::EntityNotFound,
                });
            },
        };
        let t_archetype_id = T::archetype_id()
            .map_err(|e| WorldError::DeleteComponentError { kind: e })?;
        let new_archetype_id = archetype_id
            .remove_from(t_archetype_id)
            .map_err(|e| WorldError::DeleteComponentError { kind: e })?;

        let (old_entity, map) = old_archetype
            .cut_entity(entity)
            .map_err(|e| WorldError::DeleteComponentError { kind: e })?;
        old_archetype.bundle_count -= 1;

        let new_archetype = world.get_archetype_mut(&new_archetype_id);

        if !new_archetype.are_all_columns_mutable() {
            let (_, old_archetype) = world
                .archetypes
                .iter_mut()
                .find(|(_, x)| x.check_entity(entity))
                .unwrap();
            for (type_id, (size, data)) in map.iter() {
                old_archetype
                    .add_raw(*size, *type_id, data.clone())
                    .map_err(|e| WorldError::DeleteComponentError { kind: e })?
            }
            return Err(WorldError::DeleteComponentError {
                kind: ArchetypeError::ArchetypeColumnNotWritable,
            });
        }

        for (type_id, (size, data)) in map.iter() {
            if new_archetype_id.contains_single(type_id) {
                new_archetype
                    .add_raw(*size, *type_id, data.clone())
                    .map_err(|e| WorldError::DeleteComponentError {
                        kind: e,
                    })?;
            }
        }
        new_archetype.bundle_count += 1;
        new_archetype.moved_entity(old_entity);

        Ok(())
    }
}
