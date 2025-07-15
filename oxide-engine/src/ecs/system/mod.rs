use std::collections::HashMap;

use crate::{assets::library::AssetLibrary, graphics::Graphics, input::Input};

use super::world::World;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum SystemType {
    Start,
    Update,
    Close,
}

pub struct Systems {
    systems: HashMap<SystemType, Vec<System>>,
}

impl Systems {
    pub fn new() -> Systems {
        Systems {
            systems: HashMap::new(),
        }
    }

    pub fn push(&mut self, system: System) {
        match self.systems.get_mut(&system.system_type) {
            Some(systems) => systems.push(system),
            None => {
                let _ = self.systems.insert(system.system_type, vec![system]);
            }
        };
    }

    pub fn execute_type(&self, system_type: SystemType, mut system_input: SystemInput) {
        if let Some(systems) = self.systems.get(&system_type) {
            for system in systems.iter() {
                system.execute(&mut system_input);
            }
        }
    }
}

pub struct SystemInput<'a> {
    pub world: &'a mut World,
    pub assets: &'a mut AssetLibrary,
    pub graphics: &'a mut Graphics,
    pub input: &'a Input,
}

pub struct System {
    system_type: SystemType,
    function: Box<dyn Fn(&mut SystemInput)>,
}

impl System {
    pub fn new(system_type: SystemType, function: impl Fn(&mut SystemInput) + 'static) -> System {
        System {
            system_type,
            function: Box::new(function),
        }
    }

    pub fn execute(&self, system_input: &mut SystemInput) {
        (self.function)(system_input);
    }
}
