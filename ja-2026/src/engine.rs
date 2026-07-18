use crate::board::{BoardError, Io22d08Api, BUTTON_COUNT, DIGIT_COUNT, INPUT_COUNT, RELAY_COUNT};

pub const INPUTS: usize = BUTTON_COUNT + INPUT_COUNT;

#[derive(Copy, Clone, Default)]
pub struct Display(pub [u8; DIGIT_COUNT]);

impl From<[u8; DIGIT_COUNT]> for Display {
    fn from(value: [u8; DIGIT_COUNT]) -> Self {
        Self(value)
    }
}

impl From<u16> for Display {
    fn from(mut value: u16) -> Self {
        let mut digits = [0; DIGIT_COUNT];
        for idx in (0..DIGIT_COUNT).rev() {
            digits[idx] = (value % 10) as u8;
            value /= 10;
        }
        Self(digits)
    }
}

impl From<Display> for [u8; DIGIT_COUNT] {
    fn from(value: Display) -> Self {
        value.0
    }
}

#[derive(Copy, Clone, Default)]
pub struct TickCommand {
    pub relay_state: [bool; RELAY_COUNT],
    pub display: Display,
    pub input_mode: [Option<InputMode>; INPUTS],
}

#[derive(Copy, Clone, Default)]
pub struct Snapshot {
    pub tick: u32,
    pub input_state: [bool; INPUTS],
    pub relay_state: [bool; RELAY_COUNT],
    pub display: Display,
}

impl Snapshot {
    pub fn identity_command(&self) -> TickCommand {
        TickCommand {
            relay_state: self.relay_state,
            display: self.display,
            input_mode: [None; INPUTS],
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum InputMode {
    Momentary,
    RisingEdgeToggle,
    FallingEdgeToggle,
}

#[derive(Copy, Clone)]
pub struct EngineConfig {
    pub input_modes: [InputMode; INPUTS],
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            input_modes: [InputMode::Momentary; INPUTS],
        }
    }
}

#[derive(Copy, Clone, Default)]
pub struct InputLocal {
    pub last_raw: bool,
    pub logical: bool,
}

pub trait BoardIo {
    fn read_input_raw(&mut self, idx: usize) -> Result<bool, BoardError>;
    fn write_relays(&mut self, relays: [bool; RELAY_COUNT]) -> Result<(), BoardError>;
    fn write_display(&mut self, digits: [u8; DIGIT_COUNT]) -> Result<(), BoardError>;
    fn refresh_tick(&mut self) -> Result<(), BoardError>;
}

impl<T> BoardIo for T
where
    T: Io22d08Api,
{
    fn read_input_raw(&mut self, idx: usize) -> Result<bool, BoardError> {
        if idx < BUTTON_COUNT {
            self.read_button(idx)
        } else if idx < INPUTS {
            self.read_input(idx - BUTTON_COUNT)
        } else {
            Err(BoardError::InvalidInputIndex)
        }
    }

    fn write_relays(&mut self, relays: [bool; RELAY_COUNT]) -> Result<(), BoardError> {
        for (idx, state) in relays.iter().copied().enumerate() {
            if state {
                self.relay_on(idx)?;
            } else {
                self.relay_off(idx)?;
            }
        }
        Ok(())
    }

    fn write_display(&mut self, digits: [u8; DIGIT_COUNT]) -> Result<(), BoardError> {
        for (idx, value) in digits.iter().copied().enumerate() {
            self.show_digit(idx, value)?;
        }
        Ok(())
    }

    fn refresh_tick(&mut self) -> Result<(), BoardError> {
        self.tick()
    }
}

pub struct Engine {
    pub cfg: EngineConfig,
    pub prev: Snapshot,
    pub cur: Snapshot,
    pub input_local: [InputLocal; INPUTS],
}

impl Engine {
    pub fn new(cfg: EngineConfig) -> Self {
        Self {
            cfg,
            prev: Snapshot::default(),
            cur: Snapshot::default(),
            input_local: [InputLocal::default(); INPUTS],
        }
    }

    pub fn tick<B: BoardIo>(
        &mut self,
        board: &mut B,
        command: TickCommand,
    ) -> Result<Snapshot, BoardError> {
        self.prev = self.cur;
        self.cur.tick = self.prev.tick.wrapping_add(1);

        // 1) Inputs: physical -> logical state
        for idx in 0..INPUTS {
            let raw = board.read_input_raw(idx)?;
            let local = &mut self.input_local[idx];

            if let Some(next_mode) = command.input_mode[idx] {
                if self.cfg.input_modes[idx] != next_mode {
                    self.cfg.input_modes[idx] = next_mode;
                    // Prevent an artificial edge on the same tick as mode switch.
                    local.last_raw = raw;
                }
            }

            match self.cfg.input_modes[idx] {
                InputMode::Momentary => {
                    local.logical = raw;
                }
                InputMode::RisingEdgeToggle => {
                    if raw && !local.last_raw {
                        local.logical = !local.logical;
                    }
                }
                InputMode::FallingEdgeToggle => {
                    if !raw && local.last_raw {
                        local.logical = !local.logical;
                    }
                }
            }

            local.last_raw = raw;
            self.cur.input_state[idx] = local.logical;
        }

        // 2) Accept output targets from the caller-side flow.
        self.cur.relay_state = command.relay_state;
        self.cur.display = command.display;

        // 3) Push desired states to board.
        board.write_relays(self.cur.relay_state)?;
        board.write_display(self.cur.display.into())?;

        // 4) Keep multiplex hardware refreshed.
        board.refresh_tick()?;

        Ok(self.cur)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct MockBoard {
        raw_inputs: [bool; INPUTS],
        relays: [bool; RELAY_COUNT],
        digits: [u8; DIGIT_COUNT],
    }

    impl BoardIo for MockBoard {
        fn read_input_raw(&mut self, idx: usize) -> Result<bool, BoardError> {
            Ok(self.raw_inputs[idx])
        }

        fn write_relays(&mut self, relays: [bool; RELAY_COUNT]) -> Result<(), BoardError> {
            self.relays = relays;
            Ok(())
        }

        fn write_display(&mut self, digits: [u8; DIGIT_COUNT]) -> Result<(), BoardError> {
            self.digits = digits;
            Ok(())
        }

        fn refresh_tick(&mut self) -> Result<(), BoardError> {
            Ok(())
        }
    }

    #[test]
    fn identity_command_does_not_request_mode_changes() {
        let snapshot = Snapshot::default();
        let command = snapshot.identity_command();

        assert!(command.input_mode.iter().all(Option::is_none));
    }

    #[test]
    fn mode_override_is_applied_without_false_same_tick_toggle() {
        let mut cfg = EngineConfig::default();
        cfg.input_modes[0] = InputMode::Momentary;

        let mut engine = Engine::new(cfg);
        let mut board = MockBoard::default();
        let mut command = TickCommand::default();

        board.raw_inputs[0] = true;
        command.input_mode[0] = Some(InputMode::RisingEdgeToggle);
        let snapshot = engine.tick(&mut board, command).unwrap();

        assert!(matches!(engine.cfg.input_modes[0], InputMode::RisingEdgeToggle));
        assert!(!snapshot.input_state[0]);
    }

    #[test]
    fn mode_override_persists_across_ticks_until_changed_again() {
        let mut cfg = EngineConfig::default();
        cfg.input_modes[0] = InputMode::Momentary;

        let mut engine = Engine::new(cfg);
        let mut board = MockBoard::default();

        let mut first = TickCommand::default();
        first.input_mode[0] = Some(InputMode::RisingEdgeToggle);

        board.raw_inputs[0] = true;
        let _ = engine.tick(&mut board, first).unwrap();

        board.raw_inputs[0] = false;
        let _ = engine.tick(&mut board, TickCommand::default()).unwrap();

        board.raw_inputs[0] = true;
        let snapshot = engine.tick(&mut board, TickCommand::default()).unwrap();

        assert!(matches!(engine.cfg.input_modes[0], InputMode::RisingEdgeToggle));
        assert!(snapshot.input_state[0]);
    }
}
