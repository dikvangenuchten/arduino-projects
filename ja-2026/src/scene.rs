use crate::board::DIGIT_COUNT;
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

pub struct ButtonCycleSelector {
    index: usize,
    last_state: bool,
}

impl ButtonCycleSelector {
    pub fn new(index: usize) -> Self {
        Self {
            index,
            last_state: false,
        }
    }
}

impl SceneSelector for ButtonCycleSelector {
    fn next_scene(&mut self, ctx: &SceneContext, current: SceneId) -> SceneId {
        let now = ctx.current.input_state[self.index];
        let mut next = current;

        if now && !self.last_state {
            next = match current {
                SceneId::Idle => SceneId::Rotate,
                SceneId::Rotate => SceneId::Idle,
            };
        }

        self.last_state = now;
        next
    }
}

pub struct IdleScene;

impl Scene for IdleScene {
    fn id(&self) -> SceneId {
        SceneId::Idle
    }

    fn update(&mut self, _ctx: &SceneContext) -> TickCommand {
        TickCommand::default()
    }
}

pub struct RotateScene {
    speed: u32,
}

impl RotateScene {
    pub fn new(speed: u32) -> Self {
        Self {
            speed: speed.max(1),
        }
    }
}

impl Scene for RotateScene {
    fn id(&self) -> SceneId {
        SceneId::Rotate
    }

    fn update(&mut self, ctx: &SceneContext) -> TickCommand {
        let mut command = ctx.current.identity_command();

        if ctx.current.input_state[1] {
            self.speed = (self.speed + 1).min(5000);
        } else if ctx.current.input_state[2] {
            self.speed = self.speed.saturating_sub(1).max(1);
        }

        if ctx.current.tick % self.speed == 0 {
            let idx = ctx.current.relay_state.iter().position(|&s| s).unwrap_or(0);
            let next_idx = if ctx.current.input_state[0] {
                (idx + 1) % ctx.current.relay_state.len()
            } else {
                (idx + ctx.current.relay_state.len() - 1) % ctx.current.relay_state.len()
            };
            command.relay_state.fill(false);
            command.relay_state[next_idx] = true;
        }

        command.display = digits_from_u16((self.speed % 10_000) as u16);
        command
    }
}

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
    pub fn new(selector: S, initial: SceneId, rotate_speed: u32) -> Self {
        Self {
            selector,
            active: initial,
            initialized: false,
            idle: IdleScene,
            rotate: RotateScene::new(rotate_speed),
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

fn digits_from_u16(mut value: u16) -> [u8; DIGIT_COUNT] {
    let mut digits = [0; DIGIT_COUNT];
    digits[0] = (value / 1000) as u8;
    value %= 1000;
    digits[1] = (value / 100) as u8;
    value %= 100;
    digits[2] = (value / 10) as u8;
    digits[3] = (value % 10) as u8;
    digits
}
