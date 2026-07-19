use crate::engine::TickCommand;

use super::{Scene, SceneContext, SceneId};

/// Scene that emits an all-zero [`TickCommand`] every tick, leaving all outputs off.
pub struct IdleScene;

impl Scene for IdleScene {
    fn id(&self) -> SceneId {
        SceneId::Idle
    }

    fn update(&mut self, _ctx: &SceneContext) -> TickCommand {
        TickCommand::default()
    }
}
