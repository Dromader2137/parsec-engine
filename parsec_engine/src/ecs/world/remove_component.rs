//! Module responsible for deleting components from entites.

use crate::ecs::world::archetype::{ArchetypeError, ArchetypeId};

/// Represents a type that can be used to remove components from an entity.
/// It is automatically implemented for all types implementing [`Spawn`].
pub trait RemoveComponent: Send + Sync + 'static {
    fn archetype_id() -> Result<ArchetypeId, ArchetypeError>
    where
        Self: Sized;
}
