//! Module responsible for creating new entities.

use crate::world::{
    archetype::{Archetype, ArchetypeError, ArchetypeId},
    component::Component,
};

/// Represents a type that can be used as a bundle when spawning an entity.
/// It is automatically implemented for all types implementing [`Component`]
/// and all tuples containging up to 16 values that implement [`Spawn`].
pub trait Spawn: Send + Sync + 'static {
    fn archetype_id(&self) -> Result<ArchetypeId, ArchetypeError>;
    fn spawn(&self, archetype: &mut Archetype) -> Result<(), ArchetypeError>;
}

impl<T: Component> Spawn for T {
    fn archetype_id(&self) -> Result<ArchetypeId, ArchetypeError> {
        ArchetypeId::new(vec![std::any::TypeId::of::<T>()])
    }
    fn spawn(&self, archetype: &mut Archetype) -> Result<(), ArchetypeError> {
        archetype.add(*self)?;
        Ok(())
    }
}

impl Spawn for Box<dyn Spawn> {
    fn archetype_id(&self) -> Result<ArchetypeId, ArchetypeError> {
        (**self).archetype_id()
    }
    fn spawn(&self, archetype: &mut Archetype) -> Result<(), ArchetypeError> {
        (**self).spawn(archetype)
    }
}
