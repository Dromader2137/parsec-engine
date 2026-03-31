//! Module responsible for deleting components from entites.

use crate::ecs::world::{
    archetype::{ArchetypeError, ArchetypeId},
};

/// Represents a type that can be used to remove components from an entity.
/// It is automatically implemented for all types implementing [`Spawn`].
pub trait RemoveComponent: Send + Sync + 'static {
    fn archetype_id() -> Result<ArchetypeId, ArchetypeError>
    where
        Self: Sized;
}

impl RemoveComponent for RemoveComponentData {
    fn archetype_id() -> Result<ArchetypeId, ArchetypeError> {
        ArchetypeId::new(vec![std::any::TypeId::of::<Self>()])
    }
}

pub struct RemoveComponentData {
    pub archetype_id: ArchetypeId,
}

impl RemoveComponentData {
    pub fn components<T: RemoveComponent>()
    -> Result<RemoveComponentData, ArchetypeError> {
        Ok(RemoveComponentData {
            archetype_id: T::archetype_id()?,
        })
    }
}
