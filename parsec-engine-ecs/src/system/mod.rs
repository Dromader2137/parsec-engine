//! Module responsible for systems management.

use std::{any::type_name, collections::HashMap, fmt::Debug, time::Instant};

use parsec_engine_error::ParsecError;

use crate::world::World;

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
    /// Runs when the cursor leaves the window.
    WindowCursorLeft,
    /// Runs when the cursor enters the window.
    WindowCursorEntered,
    /// Runs when there is a new keyboard input.
    KeyboardInput,
    /// Runs when there is a new mouse movement.
    MouseMovement,
    /// Runs when there is a new mouse button event.
    MouseButton,
    /// Runs on mouse scroll.
    MouseWheel,
}

/// Stores all systems grouped by [`SystemTrigger`].
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
        self.systems.entry(system_trigger).or_default()
    }

    /// Registers a new system to be executed on `system_trigger`.
    pub fn add(&mut self, system_trigger: SystemTrigger, system: impl System) {
        let trigger_vec = self.get_systems_by_trigger(system_trigger);
        trigger_vec.push(Box::new(system));
    }

    /// Registers an entire [SystemBundle].
    pub fn add_bundle(&mut self, bundle: impl SystemBundle) {
        bundle.insert(self);
    }

    /// Executes all the systems registered for trigger `system_type`.
    pub fn fire_trigger(
        &mut self,
        system_type: SystemTrigger,
        world: &mut World,
    ) -> Result<(), ParsecError> {
        if let Some(systems) = self.systems.get_mut(&system_type) {
            for system in systems.iter_mut() {
                system.run(world)?;
            }
        }
        Ok(())
    }
}

impl Default for Systems {
    fn default() -> Self { Self::new() }
}

/// Marks a type that is a system.
pub trait System: Send + Sync + 'static {
    fn run<'b>(&mut self, world: &'b mut World) -> Result<(), ParsecError>;
}

impl System for fn(&World) {
    fn run<'b>(&mut self, world: &'b mut World) -> Result<(), ParsecError> {
        (*self)(world);
        Ok(())
    }
}

impl<'a> System<'a> for fn(&mut World) {
    fn run<'b: 'a>(&mut self, world: &'b mut World) -> Result<(), ParsecError> {
        (*self)(world);
        Ok(())
    }
}

impl<'a> System<'a> for fn(&World) -> Result<(), ParsecError> {
    fn run<'b: 'a>(&mut self, world: &'b mut World) -> Result<(), ParsecError> {
        (*self)(world)
    }
}

impl<'a> System<'a> for fn(&mut World) -> Result<(), ParsecError> {
    fn run<'b: 'a>(&mut self, world: &'b mut World) -> Result<(), ParsecError> {
        (*self)(world)
    }
}

/// Marks a type used to group systems into interdependent bundles.
pub trait SystemBundle {
    /// Returns a vec of grouped systems and their respective triggers.
    fn insert(self, systems: &mut Systems);
}
