use super::{SceneContext, SceneId, SceneSelector};

/// Cycles through all [`SceneId`] variants in order on every rising edge of one button.
pub struct ButtonCycleSelector {
    index: usize,
    last_state: u8,
}

impl ButtonCycleSelector {
    /// Creates a selector driven by the logical input at `index`.
    pub fn new(index: usize) -> Self {
        Self {
            index,
            last_state: 0,
        }
    }
}

impl SceneSelector for ButtonCycleSelector {
    fn next_scene(&mut self, ctx: &SceneContext, current: SceneId) -> SceneId {
        let now = ctx.current.input_state[self.index];

        let next = if now != self.last_state {
            match now {
                0 => SceneId::Idle,
                1 => SceneId::Rotate,
                _ => current,
            }
        } else {
            current
        };

        self.last_state = now;
        next
    }
}
