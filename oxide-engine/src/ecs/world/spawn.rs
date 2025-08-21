use super::archetype::{Archetype, ArchetypeError, ArchetypeId};

pub trait Spawn {
    fn archetype_id() -> Result<ArchetypeId, ArchetypeError>;
    fn spawn(self, archetype: &mut Archetype) -> Result<(), ArchetypeError>;
}
