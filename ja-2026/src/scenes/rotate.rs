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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::Snapshot;

    fn count_on(relays: &[bool]) -> usize {
        relays.iter().copied().filter(|s| *s).count()
    }

    #[test]
    fn lag_zero_keeps_single_active_relay() {
        let mut scene = RotateScene::new(1, 0);
        let mut current = Snapshot::default();
        current.relay_state[0] = true;

        let enter_ctx = SceneContext {
            current,
            previous: Snapshot::default(),
        };
        scene.on_enter(&enter_ctx);

        for _ in 0..16 {
            let ctx = SceneContext {
                current,
                previous: current,
            };
            let command = scene.update(&ctx);
            current.relay_state = command.relay_state;
            current.tick = current.tick.wrapping_add(1);

            assert_eq!(count_on(&current.relay_state), 1);
        }
    }

    #[test]
    fn lag_keeps_previous_relay_on_for_configured_ticks() {
        let mut scene = RotateScene::new(1, 2);
        let mut current = Snapshot::default();
        current.relay_state[0] = true;

        let enter_ctx = SceneContext {
            current,
            previous: Snapshot::default(),
        };
        scene.on_enter(&enter_ctx);

        let ctx0 = SceneContext {
            current,
            previous: current,
        };
        let cmd0 = scene.update(&ctx0);
        current.relay_state = cmd0.relay_state;
        current.tick = current.tick.wrapping_add(1);
        assert!(current.relay_state[0]);

        let ctx1 = SceneContext {
            current,
            previous: current,
        };
        let cmd1 = scene.update(&ctx1);
        current.relay_state = cmd1.relay_state;
        current.tick = current.tick.wrapping_add(1);
        assert!(current.relay_state[0]);

        let ctx2 = SceneContext {
            current,
            previous: current,
        };
        let cmd2 = scene.update(&ctx2);
        current.relay_state = cmd2.relay_state;
        current.tick = current.tick.wrapping_add(1);
        assert!(!current.relay_state[0]);
    }
}
