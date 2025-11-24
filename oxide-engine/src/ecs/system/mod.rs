use std::{collections::HashMap, thread::scope};

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
            if matches!(system_type, SystemTrigger::Update) {
                scope(|x| {
                    let mut handles = Vec::new();
                    for system in systems.iter_mut() {
                        handles.push(x.spawn(move || system.run()));
                    }
                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            } else {
                for system in systems.iter_mut() {
                    system.run();
                }
            }
        }
    }
}

pub trait SystemInput {
    fn borrow() -> Self;
}

pub trait System: Send + Sync {
    fn run(&mut self);
}

pub trait SystemBundle {
    fn systems(self) -> Vec<(SystemTrigger, Box<dyn System>)>;
}
