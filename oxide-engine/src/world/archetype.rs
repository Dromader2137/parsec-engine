use core::any::TypeId;
use std::{
    any::Any,
    cell::RefCell,
    collections::{HashMap, HashSet},
    fmt::Debug,
    rc::Rc,
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
    SpawnError,
    RefCellAlreadyBorrowed,
}

#[derive(Debug, Clone)]
enum ColumnState {
    Free,
    Borrowed,
    MutBorrowed,
}

#[derive(Debug)]
pub struct ColumnStateWrapper {
    state: Rc<RefCell<ColumnState>>,
}

impl ColumnStateWrapper {
    fn new() -> ColumnStateWrapper {
        ColumnStateWrapper {
            state: Rc::new(RefCell::new(ColumnState::Free)),
        }
    }

    fn check_borrowed(&self) -> Result<(), ArchetypeError> {
        match self.state.try_borrow() {
            Ok(val) => match val.clone() {
                ColumnState::Free => Ok(()),
                ColumnState::Borrowed => Ok(()),
                ColumnState::MutBorrowed => Err(ArchetypeError::AlreadyMutablyBorrowed),
            },
            Err(_) => Err(ArchetypeError::RefCellAlreadyBorrowed),
        }
    }

    fn check_borrowed_mut(&self) -> Result<(), ArchetypeError> {
        match self.state.try_borrow() {
            Ok(val) => match val.clone() {
                ColumnState::Free => Ok(()),
                ColumnState::Borrowed => Err(ArchetypeError::AlreadyBorrowed),
                ColumnState::MutBorrowed => Err(ArchetypeError::AlreadyMutablyBorrowed),
            },
            Err(_) => Err(ArchetypeError::RefCellAlreadyBorrowed),
        }
    }

    pub fn borrow(&self) -> Result<(), ArchetypeError> {
        self.check_borrowed()?;
        match self.state.try_borrow_mut() {
            Ok(mut val) => {
                *val = ColumnState::Borrowed;
                Ok(())
            }
            Err(_) => Err(ArchetypeError::RefCellAlreadyBorrowed),
        }
    }

    pub fn borrow_mut(&self) -> Result<(), ArchetypeError> {
        self.check_borrowed_mut()?;
        match self.state.try_borrow_mut() {
            Ok(mut val) => {
                *val = ColumnState::MutBorrowed;
                Ok(())
            }
            Err(_) => Err(ArchetypeError::RefCellAlreadyBorrowed),
        }
    }

    pub fn free(&self) -> Result<(), ArchetypeError> {
        match self.state.try_borrow_mut() {
            Ok(mut val) => {
                *val = ColumnState::Free;
                Ok(())
            }
            Err(_) => Err(ArchetypeError::RefCellAlreadyBorrowed),
        }
    }
}

struct ArchetypeColumn {
    data: RefCell<Box<dyn Column>>,
    state: ColumnStateWrapper,
}

impl Debug for ArchetypeColumn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArchetypeColumn")
            .field("state", &self.state)
            .finish()
    }
}

impl ArchetypeColumn {
    fn new<T: 'static>() -> ArchetypeColumn {
        ArchetypeColumn {
            data: RefCell::new(Box::new(TypedColumn::<T> { data: Vec::new() })),
            state: ColumnStateWrapper::new(),
        }
    }

    fn push<T: 'static>(&mut self, value: T) -> Result<(), ArchetypeError> {
        self.state.check_borrowed_mut()?;

        let mut mut_borrow = self.data.borrow_mut();

        if let Some(column) = mut_borrow.as_mut_any().downcast_mut::<TypedColumn<T>>() {
            column.data.push(value);
            Ok(())
        } else {
            Err(ArchetypeError::SpawnError)
        }
    }

    fn get_slice<T: 'static>(&self) -> Result<*const [T], ArchetypeError> {
        self.state.check_borrowed()?;

        let borrow = self.data.borrow();
        let typed_column = borrow.as_any().downcast_ref::<TypedColumn<T>>();

        match typed_column {
            Some(val) => Ok(&val.data as &[T]),
            None => Err(ArchetypeError::InternalTypeMismatch),
        }
    }

    fn get_mut_slice<T: 'static>(&self) -> Result<*mut [T], ArchetypeError> {
        self.state.check_borrowed()?;

        let mut mut_borrow = self.data.borrow_mut();
        let typed_column = mut_borrow.as_mut_any().downcast_mut::<TypedColumn<T>>();

        match typed_column {
            Some(val) => Ok(&mut val.data as &mut [T]),
            None => Err(ArchetypeError::InternalTypeMismatch),
        }
    }
}

#[derive(Debug)]
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

    pub fn add<T: 'static>(&mut self, value: T) -> Result<(), ArchetypeError> {
        let type_id = TypeId::of::<T>();

        let column = self
            .columns
            .entry(type_id)
            .or_insert_with(|| ArchetypeColumn::new::<T>());

        column.push(value)
    }

    fn get_column<T: 'static>(&self) -> Result<&ArchetypeColumn, ArchetypeError> {
        match self.columns.get(&TypeId::of::<T>()) {
            Some(val) => Ok(val),
            None => Err(ArchetypeError::TypeNotFound),
        }
    }

    pub fn get<T: 'static>(&self) -> Result<(*const [T], &ColumnStateWrapper), ArchetypeError> {
        let column = self.get_column::<T>()?;
        let slice = column.get_slice::<T>()?;
        Ok((slice, &column.state))
    }

    pub fn get_mut<T: 'static>(&self) -> Result<(*mut [T], &ColumnStateWrapper), ArchetypeError> {
        let column = self.get_column::<T>()?;
        let slice = column.get_mut_slice::<T>()?;
        Ok((slice, &column.state))
    }
}
