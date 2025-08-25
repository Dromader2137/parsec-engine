use std::{
    any::TypeId,
    cell::RefCell,
    collections::{HashMap, HashSet},
    fmt::Debug, hash::{DefaultHasher, Hash, Hasher},
};

use oxide_engine_macros::{impl_spawn, multiple_tuples};

use crate::ecs::entity::Entity;

use super::{WorldError, component::Component, spawn::Spawn};

#[derive(Debug, Clone, PartialEq)]
pub enum ArchetypeError {
    ArchetypeColumnNotWritable,
    ArchetypeColumnNotReadable,
    TypeNotFound,
    BundleCannotContainManyValuesOfTheSameType,
    BundleCannotContainManyValuesOfTheSameTypeMerge,
}

impl From<ArchetypeError> for WorldError {
    fn from(value: ArchetypeError) -> Self {
        WorldError::ArchetypeError(value)
    }
}

#[derive(Debug, PartialEq, Eq)]
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

        Ok(ArchetypeId { component_types: set, hash })
    }

    pub fn merge_with(self, other: ArchetypeId) -> Result<ArchetypeId, ArchetypeError> {
        let mut set = self.component_types;
        let mut hash = self.hash;
        
        for id in other.component_types.iter() {
            let mut s = DefaultHasher::new();
            id.hash(&mut s);
            hash ^= s.finish();
            if !set.insert(*id) {
                return Err(ArchetypeError::BundleCannotContainManyValuesOfTheSameTypeMerge);
            }
        }

        Ok(ArchetypeId { component_types: set, hash })
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

    pub fn component_count(&self) -> usize {
        self.component_types.len()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ArchetypeColumnAccess {
    ReadWrite,
    Read,
    None,
}

#[derive(Debug)]
struct ArchetypeColumn {
    data: Vec<u8>,
    access: RefCell<ArchetypeColumnAccess>,
    borrow_count: RefCell<usize>,
    rows: usize,
    component_size: usize
}

impl ArchetypeColumn {
    fn new<T: Component>() -> ArchetypeColumn {
        ArchetypeColumn {
            data: Vec::new(),
            access: RefCell::new(ArchetypeColumnAccess::ReadWrite),
            borrow_count: RefCell::new(0),
            rows: 0,
            component_size: std::mem::size_of::<T>()
        }
    }

    fn push<T: Component>(&mut self, value: T) -> Result<(), ArchetypeError> {
        let access = self.access.borrow();

        if *access != ArchetypeColumnAccess::ReadWrite {
            return Err(ArchetypeError::ArchetypeColumnNotWritable);
        }

        let bytes = unsafe { std::slice::from_raw_parts(&value as *const T as *const u8, self.component_size) };

        self.data.extend_from_slice(bytes);
        self.rows += 1;

        Ok(())
    }

    fn pop(&mut self) {
        if self.rows == 0 {
            return;
        }

        let _ = self.data.split_off(self.data.len() - self.component_size);
    }

    fn get_slice<T: Component>(&self) -> Result<&[T], ArchetypeError> {
        let access = self.access.borrow();

        if *access == ArchetypeColumnAccess::None {
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
        let access = self.access.borrow();

        if *access != ArchetypeColumnAccess::ReadWrite {
            return Err(ArchetypeError::ArchetypeColumnNotWritable);
        }

        let t_slice = unsafe {
            std::slice::from_raw_parts_mut(self.data.as_ptr() as *mut T, self.data.len() / self.component_size)
        };

        Ok(t_slice)
    }
}

#[derive(Debug)]
pub struct Archetype {
    columns: HashMap<TypeId, ArchetypeColumn>,
    pub bundle_count: usize,
    component_count: usize,
    pub entities: Vec<Entity>,
}

impl Archetype {
    pub fn new(component_count: usize) -> Archetype {
        Archetype {
            columns: HashMap::new(),
            bundle_count: 0,
            component_count,
            entities: Vec::new(),
        }
    }

    pub fn add<T: Component>(&mut self, value: T) -> Result<(), ArchetypeError> {
        let type_id = TypeId::of::<T>();

        let column = self
            .columns
            .entry(type_id)
            .or_insert_with(|| ArchetypeColumn::new::<T>());

        column.push(value)
    }

    pub fn new_entity(&mut self, id: u32) {
        self.entities.push(Entity::new(id));
    }

    pub fn trim_columns(&mut self) {
        let mut desired_len = self.columns.iter().map(|x| x.1.rows).min().unwrap_or(0);
        if self.columns.len() < self.component_count {
            desired_len = 0;
        }

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

    pub fn get<T: Component>(&self) -> Result<&[T], ArchetypeError> {
        let column = self.get_column::<T>()?;
        let slice = column.get_slice::<T>()?;
        *column.borrow_count.borrow_mut() += 1;
        *column.access.borrow_mut() = ArchetypeColumnAccess::Read;
        Ok(slice)
    }

    pub fn get_mut<T: Component>(&self) -> Result<&mut [T], ArchetypeError> {
        let column = self.get_column::<T>()?;
        let slice = column.get_mut_slice::<T>()?;
        *column.access.borrow_mut() = ArchetypeColumnAccess::None;
        Ok(slice)
    }

    pub fn release_lock<T: Component>(&self) -> Result<(), ArchetypeError> {
        let column = self.get_column::<T>()?;
        let column_access = *column.access.borrow();
        match column_access {
            ArchetypeColumnAccess::None => {
                *column.access.borrow_mut() = ArchetypeColumnAccess::ReadWrite;
            },
            ArchetypeColumnAccess::Read => {
                let mut borrow_count = column.borrow_count.borrow_mut();
                *borrow_count -= 1;
                if *borrow_count == 0 {
                    *column.access.borrow_mut() = ArchetypeColumnAccess::ReadWrite;
                }
            },
            _ => ()
        };

        Ok(())
    }

    pub fn len(&self) -> usize {
        self.bundle_count
    }
}

multiple_tuples!(impl_spawn, 16);
