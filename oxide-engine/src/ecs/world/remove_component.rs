//! Trait used in the process of deleting components from entites.

use crate::ecs::world::{
    archetype::{ArchetypeError, ArchetypeId},
    spawn::Spawn,
};

/// Represents a type that can be used to remove components from an entity.
/// It is automatically implemented for all types implementing [`Spawn`].
pub trait RemoveComponent {
    fn archetype_id() -> Result<ArchetypeId, ArchetypeError>;
}

impl<T: Spawn> RemoveComponent for T {
    fn archetype_id() -> Result<ArchetypeId, ArchetypeError> { T::archetype_id() }
}
