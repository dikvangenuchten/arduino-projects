use super::{SceneContext, SceneId, SceneSelector};

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
