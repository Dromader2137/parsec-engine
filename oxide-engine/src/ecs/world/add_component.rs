use crate::ecs::world::{
    archetype::{Archetype, ArchetypeError, ArchetypeId},
    spawn::Spawn,
};

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
