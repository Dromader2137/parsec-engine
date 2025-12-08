//! Module responsible for systems management.

use std::{collections::HashMap, fmt::Debug, time::SystemTime};

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
    systems: HashMap<SystemTrigger, Vec<(Box<dyn System>, SystemStats)>>,
}

impl Debug for Systems {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut builder = f
            .debug_struct("Systems");
        
        for systems_by_trig in self.systems.iter() {
            for system in systems_by_trig.1.iter() {
                builder
                    .field(system.0.name(), &system.1);
            }
        }

        builder.finish()
    }
}

#[derive(Debug)]
pub struct SystemStats {
    pub times_called: u32,
    pub total_time: f64,
    pub min_time: f64,
    pub max_time: f64,
    pub avg_time: f64,
}

impl Default for SystemStats {
    fn default() -> Self {
        SystemStats { times_called: 0, total_time: 0.0, min_time: 1000.0, max_time: 0.0, avg_time: 0.0 }
    }
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
    ) -> &mut Vec<(Box<dyn System>, SystemStats)> {
        self.systems.entry(system_trigger).or_insert(Vec::new())
    }

    /// Registers a new system to be executed on `system_trigger`.
    pub fn add(
        &mut self,
        system_trigger: SystemTrigger,
        system: Box<dyn System>,
    ) {
        let system_vec = self.get_systems_by_trigger(system_trigger);
        system_vec.push((system, SystemStats::default()));
    }

    /// Registers an entire [SystemBundle].
    pub fn add_bundle(&mut self, bundle: impl SystemBundle) {
        for system in bundle.systems() {
            let system_vec = self.get_systems_by_trigger(system.0);
            system_vec.push((system.1, SystemStats::default()));
        }
    }

    /// Executes all the systems registered for trigger `system_type`.
    pub fn execute_type(&mut self, system_type: SystemTrigger) {
        if let Some(systems) = self.systems.get_mut(&system_type) {
            for (system, stats) in systems.iter_mut() {
                // let timer = SystemTime::now();
                system.run();
                // let duration = (SystemTime::now()
                //     .duration_since(timer)
                //     .unwrap()
                //     .as_micros() as f64)
                //     / 1000.0;
                stats.times_called += 1;
                if stats.times_called < 25 {
                    continue;
                }
                // stats.total_time += duration;
                // stats.min_time = stats.min_time.min(duration);
                // stats.max_time = stats.max_time.max(duration);
                // stats.avg_time = stats.total_time / stats.times_called as f64;
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
    fn name(&self) -> &'static str;
    fn run(&mut self);
}

/// Marks a type used to group systems into interdependent bundles.
pub trait SystemBundle {
    /// Returns a vec of grouped systems and their respective triggers.
    fn systems(self) -> Vec<(SystemTrigger, Box<dyn System>)>;
}

/// Macro used to mark functions as systems.
pub use oxide_engine_macros::system;
