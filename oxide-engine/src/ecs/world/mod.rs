use std::fmt::Debug;

use archetype::{Archetype, ArchetypeError};
use bundle::{FromColumns, FromColumnsMut, UsableBundle, UsableBundleMut};

use crate::error::EngineError;

pub mod archetype;
pub mod bundle;

#[derive(Debug, Clone, PartialEq)]
pub enum WorldError {
    ArchetypeError(ArchetypeError),
}

impl From<WorldError> for EngineError {
    fn from(value: WorldError) -> Self {
        EngineError::WorldError(value)
    }
}

#[derive(Debug)]
pub struct World {
    archetypes: Vec<Archetype>,
}

impl World {
    pub fn new() -> World {
        World { archetypes: vec![] }
    }

    pub fn spawn<T: for<'a> UsableBundle<'a>>(&mut self, bundle: T) -> Result<(), WorldError> {
        let archetype_id = T::archetype_id()?;

        let archetype = self.archetypes.iter_mut().find(|x| archetype_id == x.id);
        match archetype {
            Some(val) => bundle.add_to(val)?,
            None => {
                let mut arch = Archetype::new(archetype_id);
                bundle.add_to(&mut arch)?;
                self.archetypes.push(arch);
            }
        }

        Ok(())
    }

    pub fn query<T: for<'a> UsableBundle<'a>>(
        &self,
    ) -> Result<impl Iterator<Item = <T as FromColumns<'_>>::Output>, WorldError> {
        let bundle_archetype_id = T::archetype_id()?;
        let mut iterators = Vec::new();

        for archetype in self.archetypes.iter().filter(|x| x.id.contains(&bundle_archetype_id)) {
            let archetype_iter = T::iter_from_columns(archetype)?;
            iterators.push(archetype_iter);
        }

        Ok(iterators.into_iter().flatten())
    }

    pub fn query_mut<T: for<'a> UsableBundleMut<'a>>(
        &self,
    ) -> Result<impl Iterator<Item = <T as FromColumnsMut<'_>>::Output>, WorldError> {
        let bundle_archetype_id = T::archetype_id()?;
        let mut iterators = Vec::new();

        for archetype in self.archetypes.iter().filter(|x| x.id.contains(&bundle_archetype_id)) {
            let archetype_iter = T::iter_from_columns(archetype)?;
            iterators.push(archetype_iter);
        }

        Ok(iterators.into_iter().flatten())
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
