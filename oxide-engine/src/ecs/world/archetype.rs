use std::{
    any::TypeId,
    collections::{HashMap, HashSet},
    fmt::Debug,
};

use super::{WorldError, component::Component};

#[derive(Debug, Clone, PartialEq)]
pub enum ArchetypeError {
    InternalTypeMismatch,
    TypeNotFound,
    BundleCannotContainManyValuesOfTheSameType,
}

impl From<ArchetypeError> for WorldError {
    fn from(value: ArchetypeError) -> Self {
        WorldError::ArchetypeError(value)
    }
}

#[derive(Debug, PartialEq)]
pub struct ArchetypeId {
    component_types: HashSet<TypeId>,
}

impl ArchetypeId {
    pub fn new(component_types: Vec<TypeId>) -> Result<ArchetypeId, ArchetypeError> {
        let mut set = HashSet::new();

        for id in component_types.iter() {
            if !set.insert(*id) {
                return Err(ArchetypeError::BundleCannotContainManyValuesOfTheSameType);
            }
        }

        Ok(ArchetypeId { component_types: set })
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
}

struct ArchetypeColumn {
    data: Vec<u8>,
}

impl Debug for ArchetypeColumn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArchetypeColumn").finish()
    }
}

impl ArchetypeColumn {
    fn new<T: Component>() -> ArchetypeColumn {
        ArchetypeColumn {
            data: Vec::new()
        }
    }

    fn push<T: Component>(&mut self, value: T) -> Result<(), ArchetypeError> {
        let bytes = unsafe {
            std::slice::from_raw_parts(
                &value as *const T as *const u8,
                std::mem::size_of::<T>(),
            )
        };

        self.data.extend_from_slice(bytes);

        Ok(())
    }

    fn get_slice<T: Component>(&self) -> Result<&[T], ArchetypeError> {
        let t_slice = unsafe {
            std::slice::from_raw_parts(self.data.as_ptr() as *const T, self.data.len() / std::mem::size_of::<T>())
        };

        Ok(t_slice)
    }

    fn get_mut_slice<T: Component>(&self) -> Result<&mut [T], ArchetypeError> {
        let t_slice = unsafe {
            std::slice::from_raw_parts_mut(self.data.as_ptr() as *mut T, self.data.len() / std::mem::size_of::<T>())
        };

        Ok(t_slice)
    }
}

#[derive(Debug)]
pub struct Archetype {
    pub id: ArchetypeId,
    columns: HashMap<TypeId, ArchetypeColumn>,
    pub bundle_count: usize,
}

impl Archetype {
    pub fn new(id: ArchetypeId) -> Archetype {
        Archetype {
            id,
            columns: HashMap::new(),
            bundle_count: 0,
        }
    }

    pub fn add<T: Component>(&mut self, value: T) -> Result<(), ArchetypeError> {
        let type_id = TypeId::of::<T>();

        if !self.id.contains_single(&type_id) {
            return Err(ArchetypeError::TypeNotFound);
        }

        let column = self
            .columns
            .entry(type_id)
            .or_insert_with(|| ArchetypeColumn::new::<T>());

        column.push(value)
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
        Ok(slice)
    }

    pub fn get_mut<T: Component>(&self) -> Result<&mut [T], ArchetypeError> {
        let column = self.get_column::<T>()?;
        let slice = column.get_mut_slice::<T>()?;
        Ok(slice)
    }

    pub fn len(&self) -> usize {
        self.bundle_count
    }
}
