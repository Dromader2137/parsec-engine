//! Module responsible for timing.

use std::time::SystemTime;

use crate::{ecs::system::{system, System, SystemBundle, SystemTrigger}, resources::{Resource, Resources}};

/// Stores timing information like delta_time and current_time.
pub struct Time {
    start_time: SystemTime,
    previous_time: SystemTime,
    current_time: SystemTime,
    delta_time: f32,
}

impl Time {
    fn new() -> Time {
        let start_time = SystemTime::now();
        Time {
            start_time,
            previous_time: start_time,
            current_time: start_time,
            delta_time: 0.0,
        }
    }

    fn update_time(&mut self) {
        self.previous_time = self.current_time;
        self.current_time = SystemTime::now();
        self.delta_time = (self
            .current_time
            .duration_since(self.previous_time)
            .unwrap()
            .as_nanos() as f64
            / 1000000000.0) as f32;
    }

    /// Gets frame duration in seconds.
    pub fn delta_time(&self) -> f32 {
        self.delta_time
    }
    
    /// Gets application start time.
    pub fn start_time(&self) -> SystemTime {
        self.start_time
    }

    /// Gets current frame time. May differ slightly from [`SystemTime::now()`].
    pub fn current_time(&self) -> SystemTime {
        self.current_time
    }
}

#[system]
fn time_init() {
    Resources::add(Time::new()).unwrap();
}

#[system]
fn time_update(
    mut time: Resource<Time>
) {
    time.update_time();
}

/// Bundle containing systems responsible for time calculations.
#[derive(Default)]
pub struct TimeBundle;
impl SystemBundle for TimeBundle {
    fn systems(self) -> Vec<(SystemTrigger, Box<dyn System>)> {
        vec![
            (SystemTrigger::Start, TimeInit::new()),
            (SystemTrigger::EarlyUpdate, TimeUpdate::new()),
        ]
    }
}
