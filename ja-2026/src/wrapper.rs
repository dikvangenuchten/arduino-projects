//! AVR orchestrator: wires the pure domain (`RelayBank`, `ExternalInputs`,
//! `ButtonInputs`, `DiagnosticState`) to the `Io22d08Api` hardware boundary.
//!
//! Phase 5: read normalized hardware snapshots, debounce/map external
//! inputs, dispatch emitted actions, tick the blink engine once per
//! consumed 1 ms timer tick, flush only changed relay outputs, update the
//! selected diagnostic display, and call the board tick.

use crate::action::{apply_global_action, apply_relay_action, apply_speed_action};
use crate::board_api::{BoardError, Io22d08Api};
use crate::config::{BUTTON_COUNT, INPUT_COUNT, RELAY_COUNT};
use crate::debug::{render, ButtonInputs, DiagnosticState};
use crate::input::ExternalInputs;
use crate::relay::{RelayBank, RelayMode};

/// Thin AVR orchestrator: owns the pure domain state and drives one
/// `Io22d08Api` implementation from it.
pub struct Io22d08Controller<B>
where
    B: Io22d08Api,
{
    board: B,
    relays: RelayBank,
    external_inputs: ExternalInputs,
    buttons: ButtonInputs,
    diagnostics: DiagnosticState,
    applied_relay_outputs: [bool; RELAY_COUNT],
    applied_digits: [u8; 4],
    display_dirty: bool,
}

impl<B> Io22d08Controller<B>
where
    B: Io22d08Api,
{
    /// Construct with all relays On at boot speed and the Speed
    /// diagnostic view selected.
    pub fn new(board: B) -> Self {
        let mut relays = RelayBank::new();
        for idx in 0..RELAY_COUNT {
            relays.set_mode(idx, RelayMode::On);
        }

        Self {
            board,
            relays,
            external_inputs: ExternalInputs::new(),
            buttons: ButtonInputs::new(),
            diagnostics: DiagnosticState::new(),
            applied_relay_outputs: [false; RELAY_COUNT],
            applied_digits: [0; 4],
            display_dirty: true,
        }
    }

    /// Advance the whole system by one consumed 1 ms hardware timer tick:
    /// read normalized hardware snapshots, debounce/map external inputs,
    /// dispatch emitted actions, tick the blink engine, flush only changed
    /// relay outputs, update the selected diagnostic display, and call the
    /// board tick.
    pub fn sync_tick(&mut self) -> Result<(), BoardError> {
        let mut raw_inputs = [false; INPUT_COUNT];
        for (i, raw) in raw_inputs.iter_mut().enumerate() {
            *raw = self.board.read_input(i)?;
        }

        let mut raw_buttons = [false; BUTTON_COUNT];
        for (i, raw) in raw_buttons.iter_mut().enumerate() {
            *raw = self.board.read_button(i)?;
        }

        let actions = self.external_inputs.tick_1ms(raw_inputs);
        for action in actions.into_iter().flatten() {
            apply_relay_action(&mut self.relays, action);
            apply_global_action(&mut self.relays, action);
            apply_speed_action(&mut self.relays, action);
        }

        self.relays.tick_1ms();

        let button_edges = self.buttons.tick_1ms(raw_buttons);
        self.diagnostics.apply_button_edges(button_edges);

        self.flush_relays()?;
        self.flush_display()?;
        self.board.tick()?;

        Ok(())
    }

    fn flush_relays(&mut self) -> Result<(), BoardError> {
        for idx in 0..RELAY_COUNT {
            let output = self.relays.relay_output(idx);
            if output == self.applied_relay_outputs[idx] {
                continue;
            }

            if output {
                self.board.relay_on(idx)?;
            } else {
                self.board.relay_off(idx)?;
            }
            self.applied_relay_outputs[idx] = output;
        }

        Ok(())
    }

    fn flush_display(&mut self) -> Result<(), BoardError> {
        let digits = render(
            &self.diagnostics,
            &self.relays,
            self.external_inputs.stable(),
        );

        if !self.display_dirty && digits == self.applied_digits {
            return Ok(());
        }

        for (position, value) in digits.iter().copied().enumerate() {
            self.board.show_digit(position, value)?;
        }
        self.applied_digits = digits;
        self.display_dirty = false;

        Ok(())
    }
}

