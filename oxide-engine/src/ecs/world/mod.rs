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
        let archetype_count = self.archetypes.len();

        let archetype_index = match self.archetypes.iter().enumerate().find(|(_, x)| archetype_id == x.id) {
            Some(val) => val.0,
            None => archetype_count,
        };

        if archetype_index == archetype_count {
            self.archetypes.push(Archetype::new(archetype_id));
        }

        bundle.add_to(&mut self.archetypes[archetype_index])?;
        self.archetypes[archetype_index].bundle_count += 1;

        Ok(())
    }

    pub fn query<T: for<'a> UsableBundle<'a>>(
        &self,
    ) -> Result<impl Iterator<Item = <T as FromColumns<'_>>::Output>, WorldError> {
        let bundle_archetype_id = T::archetype_id()?;

        let mut iterators = Vec::new();

        for archetype in self.archetypes.iter() {
            if !archetype.id.contains(&bundle_archetype_id) {
                continue;
            }

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

        for archetype in self.archetypes.iter() {
            if !archetype.id.contains(&bundle_archetype_id) {
                continue;
            }

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
