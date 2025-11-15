use crate::ecs::world::{
    archetype::{ArchetypeError, ArchetypeId},
    spawn::Spawn,
};

pub trait RemoveComponent {
    fn archetype_id() -> Result<ArchetypeId, ArchetypeError>;
}

impl<T: Spawn> RemoveComponent for T {
    fn archetype_id() -> Result<ArchetypeId, ArchetypeError> { T::archetype_id() }
}
