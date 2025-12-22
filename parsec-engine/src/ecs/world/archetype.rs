//! Module responsible for handling archetypes.

use std::{
    any::TypeId,
    collections::{HashMap, HashSet},
    fmt::Debug,
    hash::{DefaultHasher, Hash, Hasher},
    sync::{Arc, RwLock},
};

use parsec_engine_macros::{impl_spawn, multiple_tuples};
use thiserror::Error;

use crate::ecs::{
    entity::Entity,
    world::{component::Component, spawn::Spawn},
};

#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchetypeError {
    #[error("Archetype column is not writable")]
    ArchetypeColumnNotWritable,
    #[error("Archetype column is not readable")]
    ArchetypeColumnNotReadable,
    #[error("Type not found")]
    TypeNotFound,
    #[error("Entity not found")]
    EntityNotFound,
    #[error("Bundle can not contain multiple values of the same type")]
    BundleCannotContainManyValuesOfTheSameType,
    #[error("Bundle can not contain multiple values of the same type")]
    BundleCannotContainManyValuesOfTheSameTypeMerge,
    #[error("Archetype doesn't contain this type")]
    ArchetypeIdDoesntContainThisType,
}

/// Unique identifier for an [`Archetype`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArchetypeId {
    component_types: HashSet<TypeId>,
    hash: u64,
}

impl Hash for ArchetypeId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(self.hash);
    }
}

impl ArchetypeId {
    /// Creates a new [`ArchetypeId`] from a list of type ids.
    ///
    /// # Errors
    ///
    /// - If `component_types` contains multiple instances of the same [`TypeId`].
    pub fn new(
        component_types: Vec<TypeId>,
    ) -> Result<ArchetypeId, ArchetypeError> {
        let mut set = HashSet::new();
        let mut hash = 0_u64;

        for id in component_types.iter() {
            let mut s = DefaultHasher::new();
            id.hash(&mut s);
            hash ^= s.finish();
            if !set.insert(*id) {
                return Err(
                    ArchetypeError::BundleCannotContainManyValuesOfTheSameType,
                );
            }
        }

        Ok(ArchetypeId {
            component_types: set,
            hash,
        })
    }

    /// Merges this [`ArchetypeId`] with `self` and outputs the resulting id.
    ///
    /// # Errors
    ///
    /// - If the resulting id has duplicate components.
    pub fn merge_with(
        &self,
        other: ArchetypeId,
    ) -> Result<ArchetypeId, ArchetypeError> {
        let mut set = self.component_types.clone();
        let mut hash = self.hash;

        for id in other.component_types.iter() {
            let mut s = DefaultHasher::new();
            id.hash(&mut s);
            hash ^= s.finish();
            if !set.insert(*id) {
                return Err(ArchetypeError::BundleCannotContainManyValuesOfTheSameTypeMerge);
            }
        }

        Ok(ArchetypeId {
            component_types: set,
            hash,
        })
    }

    /// Subtracts the other [`ArchetypeId`] from this [`ArchetypeId`] and outputs the resulting id.
    ///
    /// # Errors
    ///
    /// - If `self` doesn't contain some component types present in `other`.
    pub fn remove_from(
        &self,
        other: ArchetypeId,
    ) -> Result<ArchetypeId, ArchetypeError> {
        let mut set = self.component_types.clone();
        let mut hash = self.hash;

        for id in other.component_types.iter() {
            let mut s = DefaultHasher::new();
            id.hash(&mut s);
            hash ^= s.finish();
            if !set.remove(id) {
                return Err(ArchetypeError::ArchetypeIdDoesntContainThisType);
            }
        }

        Ok(ArchetypeId {
            component_types: set,
            hash,
        })
    }

    /// Checks if `self` contains all components present in `other_id`.
    pub fn contains(&self, other_id: &ArchetypeId) -> bool {
        if self.component_types.len() < other_id.component_types.len() {
            return false;
        }

        for component in other_id.component_types.iter() {
            if !self.component_types.contains(component) {
                return false;
            }
        }
        true
    }

    /// Checks if `self` contains a single component with [`TypeId`] `component_type`.
    pub fn contains_single(&self, component_type: &TypeId) -> bool {
        self.component_types.contains(component_type)
    }

    /// Gets the number of component ids inside `self`.
    pub fn component_count(&self) -> usize { self.component_types.len() }
}

/// Specifies the type of access that is currently possible for an [`ArchetypeColumn`].
///
/// - [ReadWrite][`ArchetypeColumnAccess::ReadWrite`] when the column is not borrowed at all.
/// - [Read][`ArchetypeColumnAccess::Read`] when the column is borrowed immutably.
/// - [None][`ArchetypeColumnAccess::Read`] when the column is borrowed mutably.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ArchetypeColumnAccess {
    ReadWrite,
    Read,
    None,
}

/// Stores information about the way an [`ArchetypeColumn`] is borrowed.
#[derive(Debug)]
pub struct BorrowingStats {
    access: ArchetypeColumnAccess,
    /// Number of active borrows.
    count: usize,
}

impl BorrowingStats {
    fn new() -> BorrowingStats {
        BorrowingStats {
            access: ArchetypeColumnAccess::ReadWrite,
            count: 0,
        }
    }

    /// Releases one borrow of an [`ArchetypeColumn`].
    pub fn release_lock(&mut self) {
        match self.access {
            ArchetypeColumnAccess::None => {
                self.access = ArchetypeColumnAccess::ReadWrite;
            },
            ArchetypeColumnAccess::Read => {
                self.count -= 1;
                if self.count == 0 {
                    self.access = ArchetypeColumnAccess::ReadWrite;
                }
            },
            _ => (),
        };
    }
}

/// Stores the data for a single type inside of an [`Archetype`].
#[derive(Debug)]
pub struct ArchetypeColumn {
    /// Raw components data.
    data: Vec<u8>,
    /// Current borrowing state.
    borrow: Arc<RwLock<BorrowingStats>>,
    /// Number of components.
    rows: usize,
    /// Size of a single component.
    component_size: usize,
}

impl ArchetypeColumn {
    fn new() -> ArchetypeColumn {
        ArchetypeColumn {
            data: Vec::new(),
            borrow: Arc::new(RwLock::new(BorrowingStats::new())),
            rows: 0,
            component_size: 1,
        }
    }

    fn is_mutable(&self) -> bool {
        let access = self.borrow.read().unwrap().access;
        access == ArchetypeColumnAccess::ReadWrite
    }

    fn is_readable(&self) -> bool {
        let access = self.borrow.read().unwrap().access;
        access == ArchetypeColumnAccess::Read
            || access == ArchetypeColumnAccess::ReadWrite
    }

    /// Sets component size to the size of `T`.
    fn set_component_size<T: Component>(&mut self) {
        self.component_size = size_of::<T>();
    }

    /// Adds a component to `self`.
    ///
    /// # Errors
    ///
    /// - If the column is not writable <=> `self.borrow.access` != [`ArchetypeColumnAccess::ReadWrite`].
    fn push<T: Component>(&mut self, value: T) -> Result<(), ArchetypeError> {
        self.set_component_size::<T>();

        if !self.is_mutable() {
            return Err(ArchetypeError::ArchetypeColumnNotWritable);
        }

        let bytes = unsafe {
            std::slice::from_raw_parts(
                &value as *const T as *const u8,
                self.component_size,
            )
        };

        self.data.extend_from_slice(bytes);
        self.rows += 1;

        Ok(())
    }

    /// Adds raw component data to `self`.
    ///
    /// # Errors
    ///
    /// - If the column is not writable (`self.borrow.access` != [`ArchetypeColumnAccess::ReadWrite`]).
    fn push_raw(&mut self, data: Vec<u8>) -> Result<(), ArchetypeError> {
        if !self.is_mutable() {
            return Err(ArchetypeError::ArchetypeColumnNotWritable);
        }

        self.data.extend_from_slice(&data);
        self.rows += 1;

        Ok(())
    }

    /// Removes the last component from `self`.
    ///
    /// # Errors
    ///
    /// - If the column is not writable (`self.borrow.access` != [`ArchetypeColumnAccess::ReadWrite`]).
    fn pop(&mut self) -> Result<(), ArchetypeError> {
        if !self.is_mutable() {
            return Err(ArchetypeError::ArchetypeColumnNotWritable);
        }

        if self.rows == 0 {
            return Ok(());
        }

        let _ = self.data.split_off(self.data.len() - self.component_size);
        Ok(())
    }

    unsafe fn pop_unchecked(&mut self) {
        if self.rows == 0 {
            return;
        }

        let _ = self.data.split_off(self.data.len() - self.component_size);
    }

    /// Copies component from row `from` to row `to`.
    ///
    /// # Errors
    ///
    /// - If `from` or `to` are larger than `self.rows` (out of bounds).
    /// - If the column is not writable (`self.borrow.access` != [`ArchetypeColumnAccess::ReadWrite`]).
    fn copy(&mut self, from: usize, to: usize) -> Result<(), ArchetypeError> {
        if !self.is_mutable() {
            return Err(ArchetypeError::ArchetypeColumnNotWritable);
        }

        if from == to {
            return Ok(());
        }

        if from >= self.rows || to >= self.rows {
            return Err(ArchetypeError::EntityNotFound);
        }

        let copy_from =
            from * self.component_size..(from + 1) * self.component_size;
        let copy_to = to * self.component_size;

        self.data.copy_within(copy_from, copy_to);

        Ok(())
    }

    /// Gets the raw data for the component in row `idx`.
    ///
    /// # Errors
    ///
    /// - If `idx` are larger than `self.rows` (out of bounds).
    /// - If the column is not readable (`self.borrow.access` != [`ArchetypeColumnAccess::Read`]).
    fn get_raw(&self, idx: usize) -> Result<&[u8], ArchetypeError> {
        if !self.is_readable() {
            return Err(ArchetypeError::ArchetypeColumnNotReadable);
        }

        if idx >= self.rows {
            return Err(ArchetypeError::EntityNotFound);
        }

        Ok(&self.data
            [self.component_size * idx..self.component_size * (idx + 1)])
    }

    /// Gets a slice of stored components.
    ///
    /// # Errors
    ///
    /// - If the column is not readable (`self.borrow.access` != [`ArchetypeColumnAccess::Read`]).
    fn get_slice<T: Component>(&self) -> Result<&[T], ArchetypeError> {
        if !self.is_readable() {
            return Err(ArchetypeError::ArchetypeColumnNotReadable);
        }

        let t_slice = unsafe {
            std::slice::from_raw_parts(
                self.data.as_ptr() as *const T,
                self.data.len() / self.component_size,
            )
        };

        Ok(t_slice)
    }

    /// Gets a mutable slice of stored components.
    ///
    /// # Errors
    ///
    /// - If the column is not writable (`self.borrow.access` != [`ArchetypeColumnAccess::ReadWrite`]).
    fn get_mut_slice<T: Component>(&self) -> Result<&mut [T], ArchetypeError> {
        if !self.is_mutable() {
            return Err(ArchetypeError::ArchetypeColumnNotWritable);
        }

        let t_slice = unsafe {
            std::slice::from_raw_parts_mut(
                self.data.as_ptr() as *mut T,
                self.data.len() / self.component_size,
            )
        };

        Ok(t_slice)
    }
}

/// Stores all data corresponding to entities containing a set of
/// [`Components`][crate::ecs::world::component::Component], along with entity ids.
#[derive(Debug)]
pub struct Archetype {
    columns: HashMap<TypeId, ArchetypeColumn>,
    pub bundle_count: usize,
    pub entities: Vec<Entity>,
}

impl Archetype {
    pub fn new() -> Archetype {
        Archetype {
            columns: HashMap::new(),
            bundle_count: 0,
            entities: Vec::new(),
        }
    }

    /// Creates a new empty entity.
    pub fn create_empty(&mut self, archetype_id: &ArchetypeId) {
        for type_id in archetype_id.component_types.iter() {
            self.columns.insert(*type_id, ArchetypeColumn::new());
        }
    }

    /// Adds a component to the last entity.
    ///
    /// # Errors
    ///
    /// - If `self` doesn't store components of type `T`.
    /// - If column storing components of type `T` is not writable.
    pub fn add<T: Component>(
        &mut self,
        value: T,
    ) -> Result<(), ArchetypeError> {
        let type_id = TypeId::of::<T>();

        let column = match self.columns.get_mut(&type_id) {
            Some(val) => val,
            None => return Err(ArchetypeError::TypeNotFound),
        };

        column.push(value)
    }

    /// Adds raw component data to the last entity.
    ///
    /// # Errors
    ///
    /// - If `self` doesn't store components of type `type_id`.
    /// - If column storing components of type `type_id` is not writable.
    pub fn add_raw(
        &mut self,
        type_size: usize,
        type_id: TypeId,
        data: Vec<u8>,
    ) -> Result<(), ArchetypeError> {
        let column = match self.columns.get_mut(&type_id) {
            Some(val) => val,
            None => return Err(ArchetypeError::TypeNotFound),
        };
        column.component_size = type_size;

        column.push_raw(data)
    }

    /// Adds a new entity.
    ///
    /// # Errors
    ///
    /// - If any column is not writable.
    pub fn new_entity(&mut self, id: u32) -> Result<Entity, ArchetypeError> {
        if !self.are_all_columns_mutable() {
            return Err(ArchetypeError::ArchetypeColumnNotWritable);
        }
        self.entities.push(Entity::new(id));
        Ok(Entity::new(id))
    }

    /// Adds a moved entity.
    pub fn moved_entity(&mut self, entity: Entity) {
        self.entities.push(entity);
    }

    /// Checks if `entity` is stored in `self`.
    pub fn check_entity(&self, entity: Entity) -> bool {
        match self.entities.iter().find(|x| **x == entity) {
            Some(_) => true,
            None => false,
        }
    }

    /// Check if all columns all mutable. Useful for spawns/deletes/cuts.
    pub fn are_all_columns_mutable(&self) -> bool {
        for (_, column) in self.columns.iter() {
            if !column.is_mutable() {
                return false;
            }
        }
        true
    }

    /// Deletes `entity` and all of it's data.
    ///
    /// # Errors
    ///
    /// - If `self` doesn`t store `entity`.
    /// - If any column is not writable.
    pub fn delete_entity(
        &mut self,
        entity: Entity,
    ) -> Result<(), ArchetypeError> {
        if !self.are_all_columns_mutable() {
            return Err(ArchetypeError::ArchetypeColumnNotWritable);
        }

        if self.entities.is_empty() {
            return Err(ArchetypeError::EntityNotFound);
        }

        let last_pos = self.entities.len() - 1;
        let entity_pos = match self
            .entities
            .iter()
            .enumerate()
            .find(|(_, x)| **x == entity)
        {
            Some((pos, _)) => pos,
            None => return Err(ArchetypeError::EntityNotFound),
        };

        for (_, column) in self.columns.iter_mut() {
            column.copy(last_pos, entity_pos)?;
            column.pop()?;
        }
        self.entities[entity_pos] = self.entities[last_pos];
        self.entities.pop();

        Ok(())
    }

    /// Cuts `entity` and returns it's data.
    ///
    /// # Errors
    ///
    /// - If `self` doesn`t store `entity`.
    /// - If any column is not writable.
    pub fn cut_entity(
        &mut self,
        entity: Entity,
    ) -> Result<(Entity, HashMap<TypeId, (usize, Vec<u8>)>), ArchetypeError>
    {
        if !self.are_all_columns_mutable() {
            return Err(ArchetypeError::ArchetypeColumnNotWritable);
        }

        let last_pos = self.entities.len() - 1;
        let entity_pos = match self
            .entities
            .iter()
            .enumerate()
            .find(|(_, x)| **x == entity)
        {
            Some((pos, _)) => pos,
            None => return Err(ArchetypeError::EntityNotFound),
        };

        let mut ret = HashMap::new();
        for (type_id, column) in self.columns.iter_mut() {
            let bytes = column.get_raw(entity_pos)?.to_vec();
            column.copy(last_pos, entity_pos)?;
            column.pop()?;
            ret.insert(*type_id, (column.component_size, bytes));
        }
        let ret_entity = self.entities[entity_pos];
        self.entities[entity_pos] = self.entities[last_pos];
        self.entities.pop();

        Ok((ret_entity, ret))
    }

    /// Makes all columns the same lenght (deletes the excess). Used only after a failed spawn or
    /// component add/remove.
    pub fn trim_columns(&mut self) {
        let desired_len =
            self.columns.iter().map(|x| x.1.rows).min().unwrap_or(0);

        for (_, column) in self.columns.iter_mut() {
            while column.rows < desired_len {
                unsafe { column.pop_unchecked() };
            }
        }

        while self.entities.len() > desired_len {
            self.entities.pop();
        }
    }

    /// Gets a reference to the column storing components of type `T`.
    ///
    /// # Errors
    ///
    /// - If `self` doesn't store components of type `T`.
    fn get_column<T: Component>(&self) -> Option<&ArchetypeColumn> {
        self.columns.get(&TypeId::of::<T>())
    }

    /// Gets column data needed to query this archetype's components.
    ///
    /// # Errors
    ///
    /// - If `self` doesn't store components of type `T`.
    /// - If column storing `T` components is not readable.
    pub fn get<T: Component>(
        &self,
    ) -> Result<(*const [T], Arc<RwLock<BorrowingStats>>, usize), ArchetypeError>
    {
        let column =
            self.get_column::<T>().ok_or(ArchetypeError::TypeNotFound)?;
        let slice = column.get_slice::<T>()?;
        column.borrow.write().unwrap().count += 1;
        column.borrow.write().unwrap().access = ArchetypeColumnAccess::Read;
        Ok((slice, column.borrow.clone(), slice.len()))
    }

    /// Gets column data needed to mutably query this archetype's components.
    ///
    /// # Errors
    ///
    /// - If `self` doesn't store components of type `T`.
    /// - If column storing `T` components is not writable.
    pub fn get_mut<T: Component>(
        &self,
    ) -> Result<(*mut [T], Arc<RwLock<BorrowingStats>>, usize), ArchetypeError>
    {
        let column =
            self.get_column::<T>().ok_or(ArchetypeError::TypeNotFound)?;
        let slice = column.get_mut_slice::<T>()?;
        column.borrow.write().unwrap().access = ArchetypeColumnAccess::None;
        Ok((slice, column.borrow.clone(), slice.len()))
    }

    /// Gets the number of entities stored in `self`.
    pub fn len(&self) -> usize { self.bundle_count }
}

multiple_tuples!(impl_spawn, 16);
