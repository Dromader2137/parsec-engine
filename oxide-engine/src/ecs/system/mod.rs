//! Module responsible for systems management.

use std::collections::HashMap;

/// List of possible actions a system can run on.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum SystemTrigger {
    /// Runs when a new frame is being rendered.
    Render,
    /// Runs when app starts.
    Start,
    /// Runs after initializing the event loop.
    LateStart,
    /// Runs every frame before `Update`.
    EarlyUpdate,
    /// Runs every frame.
    Update,
    /// Runs every frame after `Update`.
    LateUpdate,
    /// Runs when app closes.
    End,
    /// Runs when a window is resized.
    WindowResized,
    /// Runs when the cursor leaves a window.
    WindowCursorLeft,
    /// Runs when there is a new keyboard input.
    KeyboardInput,
    /// Runs when there is a new mouse movement.
    MouseMovement,
    /// Runs when there is a new mouse button event.
    MouseButton,
    /// Runs on mouse scroll.
    MouseWheel,
}

/// Stores all systems groped by [`SystemTrigger`].
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

    /// Registers a new system to be executed on `system_trigger`.
    pub fn add(
        &mut self,
        system_trigger: SystemTrigger,
        system: Box<dyn System>,
    ) {
        let system_vec = self.get_systems_by_trigger(system_trigger);
        system_vec.push(system);
    }

    /// Registers an entire [SystemBundle].
    pub fn add_bundle(&mut self, bundle: impl SystemBundle) {
        for system in bundle.systems() {
            let system_vec = self.get_systems_by_trigger(system.0);
            system_vec.push(system.1);
        }
    }

    /// Executes all the systems registered for trigger `system_type`.
    pub fn execute_type(&mut self, system_type: SystemTrigger) {
        if let Some(systems) = self.systems.get_mut(&system_type) {
            for system in systems.iter_mut() {
                system.run();
            }
        }
    }
}

impl Default for Systems {
    fn default() -> Self { Self::new() }
}

/// Marks a type that can be used as an argument inside a [`System`] function.
pub trait SystemInput {
    /// Used to create an instance of `Self` at the beginning of a system.
    fn borrow() -> Self;
}

/// Marks a type that is a system. Implemented using the [`system`] macro.
pub trait System: Send + Sync {
    fn run(&mut self);
}

/// Marks a type used to group systems into interdependent bundles.
pub trait SystemBundle {
    /// Returns a vec of grouped systems and their respective triggers.
    fn systems(self) -> Vec<(SystemTrigger, Box<dyn System>)>;
}

/// Macro used to mark functions as systems.
pub use oxide_engine_macros::system;
