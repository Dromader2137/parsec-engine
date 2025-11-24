use std::collections::HashMap;

use crate::ecs::world::{self, World, WORLD};

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
    systems: HashMap<SystemTrigger, Vec<Box<dyn System>>>,
}

impl Systems {
    pub fn new() -> Systems {
        Systems {
            systems: HashMap::new(),
        }
    }

    fn get_systems_by_trigger(
        &mut self,
        system_trigger: SystemTrigger,
    ) -> &mut Vec<Box<dyn System>> {
        self.systems.entry(system_trigger).or_insert(Vec::new())
    }

    pub fn add(&mut self, system_trigger: SystemTrigger, system: Box<dyn System>) {
        let system_vec = self.get_systems_by_trigger(system_trigger);
        system_vec.push(system);
    }

    pub fn add_bundle(&mut self, bundle: impl SystemBundle) {
        for system in bundle.systems() {
            let system_vec = self.get_systems_by_trigger(system.0);
            system_vec.push(system.1);
        }
    }

    pub fn execute_type(&mut self, system_type: SystemTrigger) {
        if let Some(systems) = self.systems.get_mut(&system_type) {
            for system in systems.iter_mut() {
                let world = WORLD.read().unwrap();
                system.run(&world);
            }
        }
    }
}

pub trait SystemInput {
    fn borrow<'world>(world: &'world World) -> Self;
}

pub trait System {
    fn run(&mut self, world: &World);
}

pub trait SystemBundle {
    fn systems(self) -> Vec<(SystemTrigger, Box<dyn System>)>;
}
