#![allow(dead_code)]

use crate::board::{BoardError, Io22d08Api, BUTTON_COUNT, DIGIT_COUNT, INPUT_COUNT, RELAY_COUNT};

#[derive(Copy, Clone)]
pub enum InputMode {
    Momentary,
    Toggle,
}

#[derive(Copy, Clone)]
pub enum RelayPattern {
    Manual,
    Blink { on_ticks: u16, off_ticks: u16 },
}

#[derive(Copy, Clone)]
pub enum RelayInit {
    Off,
    On,
    Blink { on_ticks: u16, off_ticks: u16 },
}

#[derive(Copy, Clone)]
pub struct ControllerConfig {
    pub button_modes: [InputMode; BUTTON_COUNT],
    pub input_modes: [InputMode; INPUT_COUNT],
    pub relay_init: [RelayInit; RELAY_COUNT],
    pub display_init: [u8; DIGIT_COUNT],
}

impl Default for ControllerConfig {
    fn default() -> Self {
        Self {
            button_modes: [InputMode::Momentary; BUTTON_COUNT],
            input_modes: [InputMode::Momentary; INPUT_COUNT],
            relay_init: [RelayInit::Off; RELAY_COUNT],
            display_init: [0; DIGIT_COUNT],
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ControlError {
    Board(BoardError),
    InvalidRelayIndex,
    InvalidInputIndex,
    InvalidButtonIndex,
    InvalidDigitIndex,
    InvalidBlinkPeriod,
}

impl From<BoardError> for ControlError {
    fn from(value: BoardError) -> Self {
        Self::Board(value)
    }
}

#[derive(Copy, Clone)]
struct ChannelState {
    mode: InputMode,
    state: bool,
    last_raw: bool,
}

impl Default for ChannelState {
    fn default() -> Self {
        Self {
            mode: InputMode::Momentary,
            state: false,
            last_raw: false,
        }
    }
}

#[derive(Copy, Clone)]
struct RelayState {
    desired: bool,
    applied: bool,
    pattern: RelayPattern,
    phase_on: bool,
    remaining_ticks: u16,
}

impl Default for RelayState {
    fn default() -> Self {
        Self {
            desired: false,
            applied: false,
            pattern: RelayPattern::Manual,
            phase_on: false,
            remaining_ticks: 0,
        }
    }
}

pub struct Io22d08Controller<B>
where
    B: Io22d08Api,
{
    board: B,
    buttons: [ChannelState; BUTTON_COUNT],
    inputs: [ChannelState; INPUT_COUNT],
    relays: [RelayState; RELAY_COUNT],
    display: [u8; DIGIT_COUNT],
    display_dirty: bool,
}

impl<B> Io22d08Controller<B>
where
    B: Io22d08Api,
{
    pub fn new(board: B) -> Self {
        Self {
            board,
            buttons: [ChannelState::default(); BUTTON_COUNT],
            inputs: [ChannelState::default(); INPUT_COUNT],
            relays: [RelayState::default(); RELAY_COUNT],
            display: [0; DIGIT_COUNT],
            display_dirty: true,
        }
    }

    pub fn new_with_config(board: B, config: ControllerConfig) -> Result<Self, ControlError> {
        let mut ctrl = Self::new(board);
        ctrl.apply_config(config)?;
        Ok(ctrl)
    }

    pub fn apply_config(&mut self, config: ControllerConfig) -> Result<(), ControlError> {
        for (idx, mode) in config.button_modes.iter().copied().enumerate() {
            self.set_button_mode(idx, mode)?;
        }

        for (idx, mode) in config.input_modes.iter().copied().enumerate() {
            self.set_input_mode(idx, mode)?;
        }

        for (idx, init) in config.relay_init.iter().copied().enumerate() {
            match init {
                RelayInit::Off => self.set_relay_state(idx, false)?,
                RelayInit::On => self.set_relay_state(idx, true)?,
                RelayInit::Blink { on_ticks, off_ticks } => {
                    self.set_relay_blink(idx, on_ticks, off_ticks)?
                }
            }
        }

        for (idx, value) in config.display_init.iter().copied().enumerate() {
            self.set_display_digit(idx, value)?;
        }

        Ok(())
    }

    pub fn set_button_mode(&mut self, index: usize, mode: InputMode) -> Result<(), ControlError> {
        let channel = self
            .buttons
            .get_mut(index)
            .ok_or(ControlError::InvalidButtonIndex)?;
        channel.mode = mode;
        Ok(())
    }

    pub fn set_input_mode(&mut self, index: usize, mode: InputMode) -> Result<(), ControlError> {
        let channel = self
            .inputs
            .get_mut(index)
            .ok_or(ControlError::InvalidInputIndex)?;
        channel.mode = mode;
        Ok(())
    }

    pub fn button_state(&self, index: usize) -> Result<bool, ControlError> {
        let channel = self
            .buttons
            .get(index)
            .ok_or(ControlError::InvalidButtonIndex)?;
        Ok(channel.state)
    }

    pub fn input_state(&self, index: usize) -> Result<bool, ControlError> {
        let channel = self
            .inputs
            .get(index)
            .ok_or(ControlError::InvalidInputIndex)?;
        Ok(channel.state)
    }

    pub fn set_relay_state(&mut self, index: usize, state: bool) -> Result<(), ControlError> {
        let relay = self
            .relays
            .get_mut(index)
            .ok_or(ControlError::InvalidRelayIndex)?;
        relay.pattern = RelayPattern::Manual;
        relay.desired = state;
        Ok(())
    }

    pub fn set_relay_blink(
        &mut self,
        index: usize,
        on_ticks: u16,
        off_ticks: u16,
    ) -> Result<(), ControlError> {
        if on_ticks == 0 || off_ticks == 0 {
            return Err(ControlError::InvalidBlinkPeriod);
        }

        let relay = self
            .relays
            .get_mut(index)
            .ok_or(ControlError::InvalidRelayIndex)?;
        relay.pattern = RelayPattern::Blink { on_ticks, off_ticks };
        relay.phase_on = true;
        relay.remaining_ticks = on_ticks;
        relay.desired = true;
        Ok(())
    }

    pub fn set_display_digit(&mut self, index: usize, value: u8) -> Result<(), ControlError> {
        let digit = self
            .display
            .get_mut(index)
            .ok_or(ControlError::InvalidDigitIndex)?;
        *digit = value % 10;
        self.display_dirty = true;
        Ok(())
    }

    pub fn set_display_number(&mut self, mut value: u16) {
        self.display[0] = (value / 1000) as u8;
        value %= 1000;
        self.display[1] = (value / 100) as u8;
        value %= 100;
        self.display[2] = (value / 10) as u8;
        self.display[3] = (value % 10) as u8;
        self.display_dirty = true;
    }

    pub fn sync_tick(&mut self) -> Result<(), ControlError> {
        self.update_channels()?;
        self.update_patterns();
        self.flush_relays()?;
        self.flush_display()?;
        self.board.tick()?;
        Ok(())
    }

    fn update_channels(&mut self) -> Result<(), ControlError> {
        for idx in 0..BUTTON_COUNT {
            let raw = self.board.read_button(idx)?;
            update_channel_state(&mut self.buttons[idx], raw);
        }

        for idx in 0..INPUT_COUNT {
            let raw = self.board.read_input(idx)?;
            update_channel_state(&mut self.inputs[idx], raw);
        }

        Ok(())
    }

    fn update_patterns(&mut self) {
        for relay in self.relays.iter_mut() {
            match relay.pattern {
                RelayPattern::Manual => {}
                RelayPattern::Blink { on_ticks, off_ticks } => {
                    if relay.remaining_ticks == 0 {
                        relay.phase_on = !relay.phase_on;
                        relay.remaining_ticks = if relay.phase_on { on_ticks } else { off_ticks };
                    }
                    relay.desired = relay.phase_on;
                    relay.remaining_ticks = relay.remaining_ticks.saturating_sub(1);
                }
            }
        }
    }

    fn flush_relays(&mut self) -> Result<(), ControlError> {
        for (idx, relay) in self.relays.iter_mut().enumerate() {
            if relay.desired == relay.applied {
                continue;
            }

            if relay.desired {
                self.board.relay_on(idx)?;
            } else {
                self.board.relay_off(idx)?;
            }
            relay.applied = relay.desired;
        }

        Ok(())
    }

    fn flush_display(&mut self) -> Result<(), ControlError> {
        if !self.display_dirty {
            return Ok(());
        }

        for (idx, value) in self.display.iter().copied().enumerate() {
            self.board.show_digit(idx, value)?;
        }
        self.display_dirty = false;
        Ok(())
    }
}

fn update_channel_state(channel: &mut ChannelState, raw: bool) {
    match channel.mode {
        InputMode::Momentary => {
            channel.state = raw;
        }
        InputMode::Toggle => {
            if raw && !channel.last_raw {
                channel.state = !channel.state;
            }
        }
    }

    channel.last_raw = raw;
}
