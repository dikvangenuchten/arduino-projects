use crate::engine::{Snapshot, TickCommand};

/// Identifier for each available scene.
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum SceneId {
    /// No-op scene — all outputs remain at their defaults.
    Idle,
    /// Rotating relay scene — one relay chases around the set with a configurable speed and lag.
    Rotate,
}

/// Input passed to every [`Scene::update`] call.
#[derive(Copy, Clone)]
pub struct SceneContext {
    /// Snapshot produced by the tick that is being processed.
    pub current: Snapshot,
    /// Snapshot from the preceding tick, useful for detecting state changes.
    pub previous: Snapshot,
}

/// Behaviour unit managed by [`SceneManager`].
///
/// Each scene owns its own state and produces a [`TickCommand`] each tick.
pub trait Scene {
    /// Returns the stable identifier for this scene.
    fn id(&self) -> SceneId;
    /// Called once by the manager when this scene becomes active.
    fn on_enter(&mut self, _ctx: &SceneContext) {}
    /// Called once by the manager just before this scene is deactivated.
    fn on_exit(&mut self, _ctx: &SceneContext) {}
    /// Produces the [`TickCommand`] for the current tick.
    fn update(&mut self, ctx: &SceneContext) -> TickCommand;
}

/// Decides which scene should be active based on the current context.
///
/// Called every tick by [`SceneManager`] before delegating to the active scene.
pub trait SceneSelector {
    /// Returns the [`SceneId`] that should be active this tick.
    /// May return `current` to keep the existing scene running.
    fn next_scene(&mut self, ctx: &SceneContext, current: SceneId) -> SceneId;
}
