use core::any::TypeId;
use std::{
    any::Any,
    collections::{HashMap, HashSet},
};

pub trait Column: Any {
    fn as_any(&self) -> &dyn Any;
    fn as_mut_any(&mut self) -> &mut dyn Any;
}

struct TypedColumn<T: 'static> {
    data: Vec<T>,
}

impl<T: 'static> Column for TypedColumn<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Debug, PartialEq)]
pub struct ArchetypeId {
    component_types: HashSet<TypeId>,
}

impl ArchetypeId {
    pub fn new(component_types: HashSet<TypeId>) -> ArchetypeId {
        ArchetypeId { component_types }
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
}

#[derive(Debug)]
pub enum ArchetypeError {
    AlreadyBorrowed,
    AlreadyMutablyBorrowed,
    InternalTypeMismatch,
    TypeNotFound,
}

enum ColumnState {
    Free,
    Borrwed,
    MutBorrowed,
}

struct ArchetypeColumn {
    data: Box<dyn Column>,
    state: ColumnState,
}

impl ArchetypeColumn {
    fn new<T: 'static>() -> ArchetypeColumn {
        ArchetypeColumn {
            data: Box::new(TypedColumn::<T> { data: Vec::new() }),
            state: ColumnState::Free,
        }
    }

    fn push<T: 'static>(&mut self, value: T) {
        if let Some(column) = self.data.as_mut_any().downcast_mut::<TypedColumn<T>>() {
            column.data.push(value);
        }
    }

    fn get_slice<T: 'static>(&self) -> Result<&[T], ArchetypeError> {
        match self.state {
            ColumnState::MutBorrowed => return Err(ArchetypeError::AlreadyMutablyBorrowed),
            ColumnState::Free | ColumnState::Borrwed => (),
        }

        let typed_column = self.data.as_any().downcast_ref::<TypedColumn<T>>();

        match typed_column {
            Some(val) => Ok(&val.data),
            None => Err(ArchetypeError::InternalTypeMismatch),
        }
    }

    fn get_mut_slice<T: 'static>(&mut self) -> Result<*mut [T], ArchetypeError> {
        match self.state {
            ColumnState::Borrwed => return Err(ArchetypeError::AlreadyBorrowed),
            ColumnState::MutBorrowed => return Err(ArchetypeError::AlreadyMutablyBorrowed),
            ColumnState::Free => (),
        }

        let typed_column = self.data.as_mut_any().downcast_mut::<TypedColumn<T>>();

        match typed_column {
            Some(val) => Ok(&mut val.data as &mut [T]),
            None => Err(ArchetypeError::InternalTypeMismatch),
        }
    }
}

pub struct Archetype {
    pub id: ArchetypeId,
    columns: HashMap<TypeId, ArchetypeColumn>,
    pub bundle_count: u32,
}

impl Archetype {
    pub fn new(id: ArchetypeId) -> Archetype {
        Archetype {
            id,
            columns: HashMap::new(),
            bundle_count: 0,
        }
    }

    pub fn add<T: 'static>(&mut self, value: T) {
        let type_id = TypeId::of::<T>();

        let column = self
            .columns
            .entry(type_id)
            .or_insert_with(|| ArchetypeColumn::new::<T>());

        column.push(value);
    }

    fn get_column<T: 'static>(&self) -> Result<&ArchetypeColumn, ArchetypeError> {
        match self.columns.get(&TypeId::of::<T>()) {
            Some(val) => Ok(val),
            None => Err(ArchetypeError::TypeNotFound),
        }
    }

    fn get_mut_column<T: 'static>(&mut self) -> Result<&mut ArchetypeColumn, ArchetypeError> {
        match self.columns.get_mut(&TypeId::of::<T>()) {
            Some(val) => Ok(val),
            None => Err(ArchetypeError::TypeNotFound),
        }
    }

    pub fn get<T: 'static>(&self) -> Result<&[T], ArchetypeError> {
        self.get_column::<T>()?.get_slice()
    }

    pub fn get_mut<T: 'static>(&mut self) -> Result<*mut [T], ArchetypeError> {
        self.get_mut_column::<T>()?.get_mut_slice()
    }
}
