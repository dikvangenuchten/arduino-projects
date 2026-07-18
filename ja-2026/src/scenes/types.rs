use crate::engine::{Snapshot, TickCommand};

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum SceneId {
    Idle,
    Rotate,
}

#[derive(Copy, Clone)]
pub struct SceneContext {
    pub current: Snapshot,
    pub previous: Snapshot,
}

pub trait Scene {
    fn id(&self) -> SceneId;
    fn on_enter(&mut self, _ctx: &SceneContext) {}
    fn on_exit(&mut self, _ctx: &SceneContext) {}
    fn update(&mut self, ctx: &SceneContext) -> TickCommand;
}

pub trait SceneSelector {
    fn next_scene(&mut self, ctx: &SceneContext, current: SceneId) -> SceneId;
}
