use crate::board::{BoardError, Io22d08Api, BUTTON_COUNT, DIGIT_COUNT, INPUT_COUNT, RELAY_COUNT};

pub const INPUTS: usize = BUTTON_COUNT + INPUT_COUNT;

#[derive(Copy, Clone, Default)]
pub struct TickCommand {
    pub relay_state: [bool; RELAY_COUNT],
    pub display: [u8; DIGIT_COUNT],
}

#[derive(Copy, Clone, Default)]
pub struct Snapshot {
    pub tick: u32,
    pub input_state: [bool; INPUTS],
    pub relay_state: [bool; RELAY_COUNT],
    pub display: [u8; DIGIT_COUNT],
}

impl Snapshot {
    pub fn identity_command(&self) -> TickCommand {
        TickCommand {
            relay_state: self.relay_state,
            display: self.display,
        }
    }
}

#[derive(Copy, Clone)]
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
        board.write_display(self.cur.display)?;

        // 4) Keep multiplex hardware refreshed.
        board.refresh_tick()?;

        Ok(self.cur)
    }
}
