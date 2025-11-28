//! Module responsible for adding components to entities.

use crate::ecs::world::{
    archetype::{Archetype, ArchetypeError, ArchetypeId},
    spawn::Spawn,
};

/// Marks a type that can be used to add components to an entity.
/// It is automatically implemented for all types implementing [`Spawn`].
pub trait AddComponent {
    fn archetype_id() -> Result<ArchetypeId, ArchetypeError>;
    fn add_to(self, archetype: &mut Archetype) -> Result<(), ArchetypeError>;
}

impl<T: Spawn> AddComponent for T {
    fn archetype_id() -> Result<ArchetypeId, ArchetypeError> { T::archetype_id() }
    fn add_to(self, archetype: &mut Archetype) -> Result<(), ArchetypeError> {
        Spawn::spawn(self, archetype)
    }
}
