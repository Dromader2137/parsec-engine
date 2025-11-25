//! Types responsible for handlin archetypes.

use std::{
    any::TypeId,
    collections::{HashMap, HashSet},
    fmt::Debug,
    hash::{DefaultHasher, Hash, Hasher},
    sync::{Arc, RwLock},
};

use oxide_engine_macros::{impl_spawn, multiple_tuples};

use crate::ecs::{
    entity::Entity,
    world::{WorldError, component::Component, spawn::Spawn},
};

#[derive(Debug, Clone, PartialEq)]
pub enum ArchetypeError {
    ArchetypeColumnNotWritable,
    ArchetypeColumnNotReadable,
    TypeNotFound,
    EntityNotFound,
    BundleCannotContainManyValuesOfTheSameType,
    BundleCannotContainManyValuesOfTheSameTypeMerge,
    ArchetypeIdDoesntContainThisType,
}

impl From<ArchetypeError> for WorldError {
    fn from(value: ArchetypeError) -> Self { WorldError::ArchetypeError(value) }
}

/// Uniquely identifies an [`Archetype`]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArchetypeId {
    component_types: HashSet<TypeId>,
    hash: u64,
}

impl Hash for ArchetypeId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) { state.write_u64(self.hash); }
}

impl ArchetypeId {
    /// Creates a new [`ArchetypeId`] from a list of type ids.
    ///
    /// # Errors
    /// []
    pub fn new(component_types: Vec<TypeId>) -> Result<ArchetypeId, ArchetypeError> {
        let mut set = HashSet::new();
        let mut hash = 0_u64;

        for id in component_types.iter() {
            let mut s = DefaultHasher::new();
            id.hash(&mut s);
            hash ^= s.finish();
            if !set.insert(*id) {
                return Err(ArchetypeError::BundleCannotContainManyValuesOfTheSameType);
            }
        }

        Ok(ArchetypeId {
            component_types: set,
            hash,
        })
    }

    pub fn merge_with(&self, other: ArchetypeId) -> Result<ArchetypeId, ArchetypeError> {
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

    pub fn remove_from(&self, other: ArchetypeId) -> Result<ArchetypeId, ArchetypeError> {
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

    pub fn contains_single(&self, component_type: &TypeId) -> bool {
        self.component_types.contains(component_type)
    }

    pub fn component_count(&self) -> usize { self.component_types.len() }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ArchetypeColumnAccess {
    ReadWrite,
    Read,
    None,
}

#[derive(Debug)]
pub struct BorrowingStats {
    access: ArchetypeColumnAccess,
    count: usize,
}

impl BorrowingStats {
    fn new() -> BorrowingStats {
        BorrowingStats {
            access: ArchetypeColumnAccess::ReadWrite,
            count: 0,
        }
    }

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

#[derive(Debug)]
struct ArchetypeColumn {
    data: Vec<u8>,
    borrow: Arc<RwLock<BorrowingStats>>,
    rows: usize,
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

    fn set_component_size<T: Component>(&mut self) { self.component_size = size_of::<T>(); }

    fn push<T: Component>(&mut self, value: T) -> Result<(), ArchetypeError> {
        self.set_component_size::<T>();

        let access = self.borrow.read().unwrap().access;

        if access != ArchetypeColumnAccess::ReadWrite {
            return Err(ArchetypeError::ArchetypeColumnNotWritable);
        }

        let bytes = unsafe {
            std::slice::from_raw_parts(&value as *const T as *const u8, self.component_size)
        };

        self.data.extend_from_slice(bytes);
        self.rows += 1;

        Ok(())
    }

    fn push_raw(&mut self, data: Vec<u8>) -> Result<(), ArchetypeError> {
        let access = self.borrow.read().unwrap().access;

        if access != ArchetypeColumnAccess::ReadWrite {
            return Err(ArchetypeError::ArchetypeColumnNotWritable);
        }

        self.data.extend_from_slice(&data);
        self.rows += 1;

        Ok(())
    }

    fn pop(&mut self) {
        if self.rows == 0 {
            return;
        }

        let _ = self.data.split_off(self.data.len() - self.component_size);
    }

    fn copy(&mut self, from: usize, to: usize) -> Result<(), ArchetypeError> {
        if from == to {
            return Ok(());
        }

        if from >= self.rows || to >= self.rows {
            return Err(ArchetypeError::EntityNotFound);
        }

        let copy_from = from * self.component_size..(from + 1) * self.component_size;
        let copy_to = to * self.component_size;

        self.data.copy_within(copy_from, copy_to);

        Ok(())
    }

    fn get_raw(&self, idx: usize) -> Result<&[u8], ArchetypeError> {
        if idx >= self.rows {
            return Err(ArchetypeError::EntityNotFound);
        }

        Ok(&self.data[self.component_size * idx..self.component_size * (idx + 1)])
    }

    fn get_slice<T: Component>(&self) -> Result<&[T], ArchetypeError> {
        let access = self.borrow.read().unwrap().access;

        if access == ArchetypeColumnAccess::None {
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

    fn get_mut_slice<T: Component>(&self) -> Result<&mut [T], ArchetypeError> {
        let access = self.borrow.read().unwrap().access;

        if access != ArchetypeColumnAccess::ReadWrite {
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

    pub fn create_empty(&mut self, archetype_id: &ArchetypeId) {
        for type_id in archetype_id.component_types.iter() {
            self.columns.insert(*type_id, ArchetypeColumn::new());
        }
    }

    pub fn add<T: Component>(&mut self, value: T) -> Result<(), ArchetypeError> {
        let type_id = TypeId::of::<T>();

        let column = match self.columns.get_mut(&type_id) {
            Some(val) => val,
            None => return Err(ArchetypeError::TypeNotFound),
        };

        column.push(value)
    }

    pub fn add_raw(&mut self, type_id: TypeId, data: Vec<u8>) -> Result<(), ArchetypeError> {
        let column = match self.columns.get_mut(&type_id) {
            Some(val) => val,
            None => return Err(ArchetypeError::TypeNotFound),
        };

        column.push_raw(data)
    }

    pub fn new_entity(&mut self, id: u32) { self.entities.push(Entity::new(id)); }

    pub fn moved_entity(&mut self, entity: Entity) { self.entities.push(entity); }

    pub fn check_entity(&self, entity: Entity) -> bool {
        match self.entities.iter().find(|x| **x == entity) {
            Some(_) => true,
            None => false,
        }
    }

    pub fn delete_entity(&mut self, entity: Entity) -> Result<(), ArchetypeError> {
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
            column.copy(last_pos, entity_pos).expect(&format!(
                "Both entity ids within the bounds 0..{}",
                self.entities.len()
            ));
            column.pop();
        }
        self.entities[entity_pos] = self.entities[last_pos];
        self.entities.pop();

        Ok(())
    }

    pub fn cut_entity(
        &mut self,
        entity: Entity,
    ) -> Result<(Entity, HashMap<TypeId, Vec<u8>>), ArchetypeError> {
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
            let bytes = column
                .get_raw(entity_pos)
                .expect(&format!(
                    "Entity id within the bounds 0..{}",
                    self.entities.len()
                ))
                .to_vec();
            column.copy(last_pos, entity_pos).expect(&format!(
                "Both entity ids within the bounds 0..{}",
                self.entities.len()
            ));
            column.pop();
            ret.insert(*type_id, bytes);
        }
        let ret_entity = self.entities[entity_pos];
        self.entities[entity_pos] = self.entities[last_pos];
        self.entities.pop();

        Ok((ret_entity, ret))
    }

    pub fn trim_columns(&mut self) {
        let desired_len = self.columns.iter().map(|x| x.1.rows).min().unwrap_or(0);

        for (_, column) in self.columns.iter_mut() {
            while column.rows < desired_len {
                column.pop();
            }
        }

        while self.entities.len() > desired_len {
            self.entities.pop();
        }
    }

    fn get_column<T: Component>(&self) -> Result<&ArchetypeColumn, ArchetypeError> {
        match self.columns.get(&TypeId::of::<T>()) {
            Some(val) => Ok(val),
            None => Err(ArchetypeError::TypeNotFound),
        }
    }

    pub fn get<T: Component>(
        &self,
    ) -> Result<(*const [T], Arc<RwLock<BorrowingStats>>, usize), ArchetypeError> {
        let column = self.get_column::<T>()?;
        let slice = column.get_slice::<T>()?;
        column.borrow.write().unwrap().count += 1;
        column.borrow.write().unwrap().access = ArchetypeColumnAccess::Read;
        Ok((slice, column.borrow.clone(), slice.len()))
    }

    pub fn get_mut<T: Component>(
        &self,
    ) -> Result<(*mut [T], Arc<RwLock<BorrowingStats>>, usize), ArchetypeError> {
        let column = self.get_column::<T>()?;
        let slice = column.get_mut_slice::<T>()?;
        column.borrow.write().unwrap().access = ArchetypeColumnAccess::None;
        Ok((slice, column.borrow.clone(), slice.len()))
    }

    pub fn len(&self) -> usize { self.bundle_count }
}

multiple_tuples!(impl_spawn, 16);
