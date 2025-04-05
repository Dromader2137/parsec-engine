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

pub struct Archetype {
    pub id: ArchetypeId,
    columns: HashMap<TypeId, Box<dyn Column>>,
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

        let bucket = self
            .columns
            .entry(type_id)
            .or_insert_with(|| Box::new(TypedColumn::<T> { data: Vec::new() }));

        if let Some(column) = bucket.as_mut_any().downcast_mut::<TypedColumn<T>>() {
            column.data.push(value);
        }
    }

    pub fn get<T: 'static>(&self) -> Option<&[T]> {
        self.columns
            .get(&TypeId::of::<T>())?
            .as_any()
            .downcast_ref::<TypedColumn<T>>()
            .map(|storage| &storage.data[..])
    }

    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut [T]> {
        self.columns
            .get_mut(&TypeId::of::<T>())?
            .as_mut_any()
            .downcast_mut::<TypedColumn<T>>()
            .map(|storage| &mut storage.data[..])
    }
}
