use core::any::TypeId;

use super::WorldError;

#[derive(Debug, PartialEq, Eq)]
pub struct Archetype {
    component_types: Vec<TypeId>,
}

impl Archetype {
    pub fn new(component_types: Vec<TypeId>) -> Result<Self, WorldError> {
        Ok(Self { component_types })
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
}
