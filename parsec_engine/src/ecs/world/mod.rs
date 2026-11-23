//! Module responsible for storing and querying entities and their data.

use std::{any::TypeId, collections::HashMap, fmt::Debug};

use archetype::{Archetype, ArchetypeError, ArchetypeId};
use spawn::Spawn;

use crate::{
    create_counter,
    ecs::{
        entity::Entity,
        world::{
            add_component::AddComponent, fetch::Fetch, query::Query,
            remove_component::RemoveComponent,
        },
    },
};

pub mod add_component;
mod archetype;
pub mod component;
pub mod fetch;
pub mod query;
pub mod remove_component;
pub mod spawn;

#[derive(thiserror::Error, Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum WorldError {
    #[error("failed to spawn entity: {0}")]
    Spawn(ArchetypeError),
    #[error("failed to delete entity: {0}")]
    Delete(ArchetypeError),
    #[error("failed to add components: {0}")]
    AddComponent(ArchetypeError),
    #[error("failed to delete components: {0}")]
    DeleteComponent(ArchetypeError),
    #[error("failed to query: {0}")]
    Query(ArchetypeError),
}

/// Stores all data about components and entities.
#[derive(Debug)]
pub struct World {
    /// Contains all archetypes indexed by their id.
    archetypes: HashMap<ArchetypeId, Archetype>,
}

impl Default for World {
    fn default() -> Self { Self::new() }
}

create_counter! {ENTITY_ID_COUNTER}
impl World {
    pub fn new() -> Self { Self { archetypes: HashMap::new() } }

    /// Creates a query for all entities matching `T`.
    pub fn query<T: Fetch>(&self) -> Result<Query<T>, WorldError> {
        Query::<T>::from_world(self).map_err(WorldError::Query)
    }

    /// Returns a mutable reference to the archetype stored under `archetype_id`.
    /// If there was no such archetype, it is created, added to `self` and a mutable reference is returned.
    fn get_archetype_mut(&mut self, archetype_id: &ArchetypeId) -> &mut Archetype {
        self.archetypes.entry(archetype_id.clone()).or_insert_with(|| {
            let mut archetype = Archetype::new();
            archetype.create_empty(archetype_id);
            archetype
        })
    }

    /// Spawns a new entity.
    ///
    /// # Errors
    ///
    /// - If `T` can't produce a valid [`ArchetypeId`].
    /// - If the [archetype][Archetype] is already borrowed in some way.
    pub fn spawn<T: Spawn>(&mut self, bundle: T) -> Result<Entity, WorldError> {
        let archetype_id = bundle.archetype_id().map_err(WorldError::Spawn)?;
        let entity_id = ENTITY_ID_COUNTER.next();
        let archetype = self.get_archetype_mut(&archetype_id);
        let entity = archetype.new_entity(entity_id).map_err(WorldError::Spawn)?;
        bundle.spawn(archetype).map_err(WorldError::Spawn)?;
        archetype.bundle_count += 1;
        Ok(entity)
    }

    /// Deletes the given entity.
    ///
    /// # Errors
    ///
    /// - If `entity` doesn't exist.
    /// - If the [archetype][Archetype] containing `entity` is already borrowed in some way.
    pub fn delete(&mut self, entity: Entity) -> Result<(), WorldError> {
        for (_, archetype) in self.archetypes.iter_mut() {
            if archetype.entities.contains(&entity) && !archetype.are_all_columns_mutable() {
                return Err(WorldError::Delete(ArchetypeError::ArchetypeColumnNotWritable));
            }
            match archetype.delete_entity(entity) {
                Ok(()) => {
                    archetype.bundle_count -= 1;
                    return Ok(());
                },
                Err(ArchetypeError::EntityNotFound) => (),
                Err(err) => return Err(WorldError::Delete(err)),
            };
        }
        Err(WorldError::Delete(ArchetypeError::EntityNotFound))
    }

    /// Add components to an already existing entity.
    ///
    /// # Errors
    ///
    /// - If `T` can't produce a valid [`ArchetypeId`].
    /// - If `T` merged with the type of `entity` can't produce a valid [`ArchetypeId`].
    /// - If either the original [archetype][Archetype] containing `entity` or the destination [archetype][Archetype] is already borrowed in some way.
    pub fn add_components<T: AddComponent>(
        &mut self,
        entity: Entity,
        bundle_extension: T,
    ) -> Result<(), WorldError> {
        let (old_archetype_id, old_archetype) = self
            .archetypes
            .iter_mut()
            .find(|(_, x)| x.check_entity(entity))
            .ok_or(WorldError::AddComponent(ArchetypeError::EntityNotFound))?;
        let old_archetype_id = old_archetype_id.clone();
        let t_archetype_id =
            bundle_extension.archetype_id().map_err(WorldError::AddComponent)?;
        let new_archetype_id =
            old_archetype_id.merge_with(t_archetype_id).map_err(WorldError::AddComponent)?;

        let (old_entity, map) =
            old_archetype.cut_entity(entity).map_err(WorldError::AddComponent)?;
        old_archetype.bundle_count -= 1;

        let new_archetype = self.get_archetype_mut(&new_archetype_id);

        if !new_archetype.are_all_columns_mutable() {
            self.restore_entity(&old_archetype_id, old_entity, &map)
                .map_err(WorldError::AddComponent)?;
            return Err(WorldError::AddComponent(ArchetypeError::ArchetypeColumnNotWritable));
        }

        for (type_id, (size, data)) in map.iter() {
            new_archetype
                .add_raw(*size, *type_id, data.clone())
                .map_err(WorldError::AddComponent)?;
        }
        bundle_extension.add_to(new_archetype).map_err(WorldError::AddComponent)?;
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
        &mut self,
        entity: Entity,
    ) -> Result<(), WorldError> {
        let t_archetype_id = T::archetype_id().map_err(WorldError::DeleteComponent)?;
        let (old_archetype_id, old_archetype) = self
            .archetypes
            .iter_mut()
            .find(|(_, x)| x.check_entity(entity))
            .ok_or(WorldError::DeleteComponent(ArchetypeError::EntityNotFound))?;
        let old_archetype_id = old_archetype_id.clone();
        let new_archetype_id =
            old_archetype_id.remove_from(t_archetype_id).map_err(WorldError::DeleteComponent)?;

        let (old_entity, map) =
            old_archetype.cut_entity(entity).map_err(WorldError::DeleteComponent)?;
        old_archetype.bundle_count -= 1;

        let new_archetype = self.get_archetype_mut(&new_archetype_id);

        if !new_archetype.are_all_columns_mutable() {
            self.restore_entity(&old_archetype_id, old_entity, &map)
                .map_err(WorldError::DeleteComponent)?;
            return Err(WorldError::DeleteComponent(ArchetypeError::ArchetypeColumnNotWritable));
        }

        for (type_id, (size, data)) in map.iter() {
            if new_archetype_id.contains_single(type_id) {
                new_archetype
                    .add_raw(*size, *type_id, data.clone())
                    .map_err(WorldError::DeleteComponent)?;
            }
        }
        new_archetype.bundle_count += 1;
        new_archetype.moved_entity(old_entity);

        Ok(())
    }

    fn restore_entity(
        &mut self,
        archetype_id: &ArchetypeId,
        entity: Entity,
        data: &HashMap<TypeId, (usize, Vec<u8>)>,
    ) -> Result<(), ArchetypeError> {
        let archetype =
            self.archetypes.get_mut(archetype_id).ok_or(ArchetypeError::EntityNotFound)?;
        for (type_id, (size, data)) in data.iter() {
            archetype.add_raw(*size, *type_id, data.clone())?;
        }
        archetype.bundle_count += 1;
        archetype.moved_entity(entity);
        Ok(())
    }
}
