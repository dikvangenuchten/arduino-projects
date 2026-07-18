use crate::engine::TickCommand;

use super::{Scene, SceneContext, SceneId};

pub struct RotateScene {
    speed: u32,
    lag: u32,
    active_idx: usize,
    pending_off: [Option<u32>; crate::board::RELAY_COUNT],
}

impl RotateScene {
    pub fn new(speed: u32, lag: u32) -> Self {
        Self {
            speed: speed.max(1),
            lag,
            active_idx: 0,
            pending_off: [None; crate::board::RELAY_COUNT],
        }
    }
}

impl Scene for RotateScene {
    fn id(&self) -> SceneId {
        SceneId::Rotate
    }

    fn on_enter(&mut self, ctx: &SceneContext) {
        self.active_idx = ctx.current.relay_state.iter().position(|&s| s).unwrap_or(0);
        self.pending_off = [None; crate::board::RELAY_COUNT];
    }

    fn update(&mut self, ctx: &SceneContext) -> TickCommand {
        let mut command = ctx.current.identity_command();

        if ctx.current.input_state[1] {
            self.speed = (self.speed + 1).min(5000);
        } else if ctx.current.input_state[2] {
            self.speed = self.speed.saturating_sub(1).max(1);
        }

        if ctx.current.tick % self.speed == 0 {
            let idx = self.active_idx;
            let next_idx = if ctx.current.input_state[0] {
                (idx + 1) % ctx.current.relay_state.len()
            } else {
                (idx + ctx.current.relay_state.len() - 1) % ctx.current.relay_state.len()
            };

            command.relay_state[next_idx] = true;
            self.pending_off[next_idx] = None;

            if next_idx != idx {
                if self.lag == 0 {
                    command.relay_state[idx] = false;
                    self.pending_off[idx] = None;
                } else {
                    self.pending_off[idx] = Some(ctx.current.tick.wrapping_add(self.lag));
                }
            }

            self.active_idx = next_idx;
        }

        for relay_idx in 0..command.relay_state.len() {
            if let Some(off_at) = self.pending_off[relay_idx] {
                if ctx.current.tick >= off_at {
                    command.relay_state[relay_idx] = false;
                    self.pending_off[relay_idx] = None;
                }
            }
        }

        command.display = ((self.speed % 10_000) as u16).into();
        command
    }
}
