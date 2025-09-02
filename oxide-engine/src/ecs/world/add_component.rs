use crate::ecs::world::spawn::Spawn;

use super::archetype::{Archetype, ArchetypeError, ArchetypeId};

pub trait AddComponent {
    fn archetype_id() -> Result<ArchetypeId, ArchetypeError>;
    fn add_to(self, archetype: &mut Archetype) -> Result<(), ArchetypeError>;
}

impl<T: Spawn> AddComponent for T {
    fn archetype_id() -> Result<ArchetypeId, ArchetypeError> {
        T::archetype_id()
    }
    fn add_to(self, archetype: &mut Archetype) -> Result<(), ArchetypeError> {
        Spawn::spawn(self, archetype)
    }
}
