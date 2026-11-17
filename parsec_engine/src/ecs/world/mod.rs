//! Module responsible for storing and querying entities and their data.

use std::{collections::HashMap, fmt::Debug};

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

    /// Creates a query for all entities matching `T`. Panics on archetype errors.
    pub fn query<T: Fetch>(&self) -> Query<T> {
        Query::<T>::from_world(self).expect("query failed")
    }

    /// Returns a mutable reference to the archetype stored under `archetype_id`.
    /// If there was no such archetype, it is created, added to `self` and a mutable reference is returned.
    fn get_archetype_mut(&mut self, archetype_id: &ArchetypeId) -> &mut Archetype {
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
    pub fn spawn<T: Spawn>(&mut self, bundle: T) -> Result<Entity, WorldError> {
        let archetype_id = bundle.archetype_id().map_err(|e| WorldError::Spawn { kind: e })?;
        let entity_id = ENTITY_ID_COUNTER.next();
        let archetype = self.get_archetype_mut(&archetype_id);
        let entity =
            archetype.new_entity(entity_id).map_err(|e| WorldError::Spawn { kind: e })?;
        bundle.spawn(archetype).map_err(|e| WorldError::Spawn { kind: e })?;
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
                return Err(WorldError::Delete {
                    kind: ArchetypeError::ArchetypeColumnNotWritable,
                });
            }
            match archetype.delete_entity(entity) {
                Ok(()) => {
                    archetype.bundle_count -= 1;
                    return Ok(());
                },
                Err(ArchetypeError::EntityNotFound) => (),
                Err(err) => return Err(WorldError::Delete { kind: err }),
            };
        }
        Err(WorldError::Delete { kind: ArchetypeError::EntityNotFound })
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
        let (archetype_id, old_archetype) =
            match self.archetypes.iter_mut().find(|(_, x)| x.check_entity(entity)) {
                Some(val) => val,
                None => {
                    return Err(WorldError::Delete { kind: ArchetypeError::EntityNotFound });
                },
            };
        let t_archetype_id = bundle_extension
            .archetype_id()
            .map_err(|e| WorldError::AddComponent { kind: e })?;
        let new_archetype_id = archetype_id
            .merge_with(t_archetype_id)
            .map_err(|e| WorldError::AddComponent { kind: e })?;

        let (old_entity, map) = old_archetype
            .cut_entity(entity)
            .map_err(|e| WorldError::AddComponent { kind: e })?;
        old_archetype.bundle_count -= 1;

        let new_archetype = self.get_archetype_mut(&new_archetype_id);

        if !new_archetype.are_all_columns_mutable() {
            let (_, old_archetype) =
                self.archetypes.iter_mut().find(|(_, x)| x.check_entity(entity)).unwrap();
            for (type_id, (size, data)) in map.iter() {
                old_archetype
                    .add_raw(*size, *type_id, data.clone())
                    .map_err(|e| WorldError::AddComponent { kind: e })?;
            }
            return Err(WorldError::AddComponent {
                kind: ArchetypeError::ArchetypeColumnNotWritable,
            });
        }

        for (type_id, (size, data)) in map.iter() {
            new_archetype
                .add_raw(*size, *type_id, data.clone())
                .map_err(|e| WorldError::AddComponent { kind: e })?;
        }
        bundle_extension
            .add_to(new_archetype)
            .map_err(|e| WorldError::AddComponent { kind: e })?;
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
        let t_archetype_id =
            T::archetype_id().map_err(|e| WorldError::DeleteComponent { kind: e })?;
        let (archetype_id, old_archetype) =
            match self.archetypes.iter_mut().find(|(_, x)| x.check_entity(entity)) {
                Some(val) => val,
                None => {
                    return Err(WorldError::DeleteComponent {
                        kind: ArchetypeError::EntityNotFound,
                    });
                },
            };
        let new_archetype_id = archetype_id
            .remove_from(t_archetype_id)
            .map_err(|e| WorldError::DeleteComponent { kind: e })?;

        let (old_entity, map) = old_archetype
            .cut_entity(entity)
            .map_err(|e| WorldError::DeleteComponent { kind: e })?;
        old_archetype.bundle_count -= 1;

        let new_archetype = self.get_archetype_mut(&new_archetype_id);

        if !new_archetype.are_all_columns_mutable() {
            let (_, old_archetype) =
                self.archetypes.iter_mut().find(|(_, x)| x.check_entity(entity)).unwrap();
            for (type_id, (size, data)) in map.iter() {
                old_archetype
                    .add_raw(*size, *type_id, data.clone())
                    .map_err(|e| WorldError::DeleteComponent { kind: e })?
            }
            return Err(WorldError::DeleteComponent {
                kind: ArchetypeError::ArchetypeColumnNotWritable,
            });
        }

        for (type_id, (size, data)) in map.iter() {
            if new_archetype_id.contains_single(type_id) {
                new_archetype
                    .add_raw(*size, *type_id, data.clone())
                    .map_err(|e| WorldError::DeleteComponent { kind: e })?;
            }
        }
        new_archetype.bundle_count += 1;
        new_archetype.moved_entity(old_entity);

        Ok(())
    }
}
