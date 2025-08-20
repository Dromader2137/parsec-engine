use oxide_engine_macros::{multiple_tuples, impl_spawn};

use super::archetype::{Archetype, ArchetypeError, ArchetypeId};
use super::component::Component;

pub trait Spawn {
    fn archetype_id() -> Result<ArchetypeId, ArchetypeError>;
    fn spawn(self, archetype: &mut Archetype) -> Result<(), ArchetypeError>;
}

multiple_tuples!(impl_spawn, 16);
