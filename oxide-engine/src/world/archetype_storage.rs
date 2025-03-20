use std::any::Any;

use super::{bundle::Bundle, WorldError};

pub trait ArchetypeStorageDyn {
    fn add_dyn(&mut self, bundle: Box<dyn Any>) -> Result<u32, WorldError>;
    fn get_dyn(&self) -> Vec<&dyn Any>;
    fn get_dyn_mut(&mut self) -> Vec<&mut dyn Any>;
}

impl<A: Bundle + Send + Sync + 'static> ArchetypeStorageDyn for Vec<A> {
    fn add_dyn(&mut self, bundle: Box<dyn Any>) -> Result<u32, WorldError> {
        if let Ok(concrete) = bundle.downcast::<A>() {
            self.push(*concrete);
            Ok((self.len()-1) as u32)
        } else {
            Err(WorldError::SpawnTypeMismatch)
        }
    }

    fn get_dyn(&self) -> Vec<&dyn Any> {
        self.iter().map(|b| b as &dyn Any).collect()
    }
    
    fn get_dyn_mut(&mut self) -> Vec<&mut dyn Any> {
        self.iter_mut().map(|b| b as &mut dyn Any).collect()
    }
}
