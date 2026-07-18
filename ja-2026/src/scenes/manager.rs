use crate::engine::TickCommand;

use super::idle::IdleScene;
use super::rotate::RotateScene;
use super::{Scene, SceneContext, SceneId, SceneSelector};

pub struct SceneManager<S>
where
    S: SceneSelector,
{
    selector: S,
    active: SceneId,
    initialized: bool,
    idle: IdleScene,
    rotate: RotateScene,
}

impl<S> SceneManager<S>
where
    S: SceneSelector,
{
    pub fn new(selector: S, initial: SceneId, rotate_speed: u32, rotate_lag: u32) -> Self {
        Self {
            selector,
            active: initial,
            initialized: false,
            idle: IdleScene,
            rotate: RotateScene::new(rotate_speed, rotate_lag),
        }
    }

    pub fn update(&mut self, ctx: &SceneContext) -> TickCommand {
        if !self.initialized {
            self.with_active_mut(|scene| scene.on_enter(ctx));
            self.initialized = true;
        }

        let next = self.selector.next_scene(ctx, self.active);
        if next != self.active {
            self.with_active_mut(|scene| scene.on_exit(ctx));
            self.active = next;
            self.with_active_mut(|scene| scene.on_enter(ctx));
        }

        self.with_active_mut(|scene| scene.update(ctx))
    }

    pub fn active_scene(&self) -> SceneId {
        self.active
    }

    fn with_active_mut<R>(&mut self, f: impl FnOnce(&mut dyn Scene) -> R) -> R {
        match self.active {
            SceneId::Idle => f(&mut self.idle),
            SceneId::Rotate => f(&mut self.rotate),
        }
    }
}
