use std::collections::HashMap;

use super::world::World;
use crate::{assets::library::AssetLibrary, resources::ResourceCollection};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum SystemTrigger {
    Render,
    Start,
    LateStart,
    Update,
    End,
    WindowResized,
    WindowCursorLeft,
    KeyboardInput,
}

pub struct Systems {
    systems: HashMap<SystemTrigger, Vec<System>>,
}

impl Systems {
    pub fn new() -> Systems {
        Systems {
            systems: HashMap::new(),
        }
    }

    fn get_systems_by_trigger(&mut self, system_trigger: SystemTrigger) -> &mut Vec<System> {
        self.systems.entry(system_trigger).or_insert(Vec::new())
    }

    pub fn add(&mut self, system: System) {
        let system_vec = self.get_systems_by_trigger(system.system_trigger);
        system_vec.push(system);
    }

    pub fn add_bundle(&mut self, bundle: impl SystemBundle) {
        for system in bundle.systems() {
            let system_vec = self.get_systems_by_trigger(system.system_trigger);
            system_vec.push(system);
        }
    }

    pub fn execute_type(&self, system_type: SystemTrigger, mut system_input: SystemInput) {
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
    pub resources: &'a mut ResourceCollection,
}

pub struct System {
    system_trigger: SystemTrigger,
    function: Box<dyn Fn(&mut SystemInput)>,
}

impl System {
    pub fn new(
        system_trigger: SystemTrigger,
        function: impl Fn(&mut SystemInput) + 'static,
    ) -> System {
        System {
            system_trigger,
            function: Box::new(function),
        }
    }

    pub fn execute(&self, system_input: &mut SystemInput) {
        (self.function)(system_input);
    }
}

pub trait SystemBundle {
    fn systems(self) -> Vec<System>;
}
