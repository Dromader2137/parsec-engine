use super::{
    archetype::{Archetype, ArchetypeError, ArchetypeId},
    component::Component,
};

pub trait Spawn {
    fn archetype_id() -> Result<ArchetypeId, ArchetypeError>;
    fn spawn(self, archetype: &mut Archetype) -> Result<(), ArchetypeError>;
}

impl<T: Component> Spawn for T {
    fn archetype_id() -> Result<ArchetypeId, ArchetypeError> {
        ArchetypeId::new(vec![std::any::TypeId::of::<T>()])
    }
    fn spawn(self, archetype: &mut Archetype) -> Result<(), ArchetypeError> {
        archetype.add(self)?;
        Ok(())
    }
}
