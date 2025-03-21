use core::any::TypeId;
use std::{any::Any, fmt::Debug};

use super::{bundle::Bundle, WorldError};

#[derive(Debug)]
pub struct Archetype {
    component_types: Vec<TypeId>,
    columns: Box<dyn ColumnarTuple>
}

pub trait ColumnarTuple: Debug {
    fn push(&mut self, value: Box<dyn Bundle>);
    fn get<'a>(&self, index: usize) -> Vec<&'a dyn Any>;
}

impl<A: Debug + 'static, B: Debug + 'static> ColumnarTuple for [Vec<A>, Vec<B>] {
    fn get<'a>(&self, index: usize) -> Vec<&'a dyn Any> {
        self[index]
    }

    fn push(&mut self, value: Box<dyn Bundle>) {
    }
}

impl Archetype {
    pub fn new(component_types: Vec<TypeId>) -> Result<Archetype, WorldError> {
        Ok(
            Archetype { 
                component_types,
                columns: vec![Vec::new(); component_types.len()]
            }
        )
    }

    pub fn contains(&self, other: &Archetype) -> bool {
        if self.component_types.len() < other.component_types.len() {
            return false;
        }
     
        for component in other.component_types.iter() {
            if !self.component_types.contains(component) {
                return false;
            }
        }

        true
    }
    
    pub fn mapping(&self, other: &Archetype) -> Option<Vec<usize>> {
        if self.component_types.len() < other.component_types.len() {
            return None;
        }

        let mut self_iter = self.component_types.iter().enumerate();
        let other_iter = other.component_types.iter();
        let mut res = vec![];

        for other_current in other_iter {
            while let Some((id, self_current)) = self_iter.next() {
                if self_current == other_current {
                    res.push(id);
                    break;
                }
            }
        }
        
        if res.len() != other.component_types.len() {
            None
        } else {
            Some(res)
        }
    }

    pub fn add_bundle(&mut self, bundle: )
}
