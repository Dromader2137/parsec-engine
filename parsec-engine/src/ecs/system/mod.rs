//! Module responsible for systems management.

use std::{collections::HashMap, fmt::Debug, marker::PhantomData};

use crate::{
    assets::AssetLibrary,
    ctx::Ctx,
    ecs::{resources::Resources, world::World},
    error::ParsecError,
};

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
    pub fn add<M>(
        &mut self,
        system_trigger: SystemTrigger,
        system: impl IntoSystem<M>,
    ) {
        let trigger_vec = self.get_systems_by_trigger(system_trigger);
        trigger_vec.push(Box::new(system.into_system()));
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
        resources: &mut Resources,
        assets: &mut AssetLibrary,
    ) -> Result<(), ParsecError> {
        if let Some(systems) = self.systems.get_mut(&system_type) {
            for system in systems.iter_mut() {
                system.run(Ctx {
                    world: &mut *world,
                    resources: &mut *resources,
                    assets: &mut *assets,
                })?;
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
    fn run<'a>(&mut self, ctx: Ctx<'a>) -> Result<(), ParsecError>;
}

pub trait IntoSystem<Marker> {
    type ResultingSystem: System;
    fn into_system(self) -> Self::ResultingSystem;
}

pub struct FunctionSystem<F, Marker> {
    function: F,
    _marker: PhantomData<Marker>,
}

pub struct CtxMarker;
pub struct CtxResultMarker;

impl<F> IntoSystem<CtxMarker> for F
where
    F: for<'a> FnMut(Ctx<'a>) + Send + Sync + 'static,
{
    type ResultingSystem = FunctionSystem<F, CtxMarker>;
    fn into_system(self) -> Self::ResultingSystem {
        FunctionSystem {
            function: self,
            _marker: PhantomData,
        }
    }
}
impl<F> System for FunctionSystem<F, CtxMarker>
where
    F: for<'a> FnMut(Ctx<'a>) + Send + Sync + 'static,
{
    fn run<'a>(&mut self, ctx: Ctx<'a>) -> Result<(), ParsecError> {
        (self.function)(ctx);
        Ok(())
    }
}

impl<F> IntoSystem<CtxResultMarker> for F
where
    F: for<'a> FnMut(Ctx<'a>) -> Result<(), ParsecError>
        + Send
        + Sync
        + 'static,
{
    type ResultingSystem = FunctionSystem<F, CtxResultMarker>;
    fn into_system(self) -> Self::ResultingSystem {
        FunctionSystem {
            function: self,
            _marker: PhantomData,
        }
    }
}
impl<F> System for FunctionSystem<F, CtxResultMarker>
where
    F: for<'a> FnMut(Ctx<'a>) -> Result<(), ParsecError>
        + Send
        + Sync
        + 'static,
{
    fn run<'a>(&mut self, ctx: Ctx<'a>) -> Result<(), ParsecError> {
        (self.function)(ctx)
    }
}

/// Marks a type used to group systems into interdependent bundles.
pub trait SystemBundle {
    /// Inserts the bundle's systems into `systems`.
    fn insert(self, systems: &mut Systems);
}
