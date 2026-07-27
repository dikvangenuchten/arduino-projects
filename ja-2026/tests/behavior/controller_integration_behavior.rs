//! Phase 5: end-to-end orchestration behavior for `Io22d08Controller`,
//! exercised against a fake `Io22d08Api` so the whole read -> debounce/map
//! -> dispatch -> blink -> flush pipeline is host-testable.

use std::cell::RefCell;
use std::rc::Rc;

use crate::board_api::{BoardError, Io22d08Api};
use crate::config::{BUTTON_COUNT, DEBOUNCE_SAMPLES, INPUT_COUNT, RELAY_COUNT};
use crate::wrapper::Io22d08Controller;

struct FakeBoardState {
    inputs: [bool; INPUT_COUNT],
    buttons: [bool; BUTTON_COUNT],
    relay_state: [bool; RELAY_COUNT],
    relay_on_calls: [u32; RELAY_COUNT],
    relay_off_calls: [u32; RELAY_COUNT],
    digits: [u8; 4],
    show_digit_calls: u32,
    tick_calls: u32,
}

impl Default for FakeBoardState {
    fn default() -> Self {
        Self {
            inputs: [false; INPUT_COUNT],
            buttons: [false; BUTTON_COUNT],
            relay_state: [false; RELAY_COUNT],
            relay_on_calls: [0; RELAY_COUNT],
            relay_off_calls: [0; RELAY_COUNT],
            digits: [0; 4],
            show_digit_calls: 0,
            tick_calls: 0,
        }
    }
}

struct FakeBoard {
    state: Rc<RefCell<FakeBoardState>>,
}

impl Io22d08Api for FakeBoard {
    fn set_number(&mut self, _value: u16) {}

    fn show_digit(&mut self, position: usize, value: u8) -> Result<(), BoardError> {
        let mut state = self.state.borrow_mut();
        state.digits[position] = value;
        state.show_digit_calls += 1;
        Ok(())
    }

    fn relay_on(&mut self, relay: usize) -> Result<(), BoardError> {
        let mut state = self.state.borrow_mut();
        state.relay_state[relay] = true;
        state.relay_on_calls[relay] += 1;
        Ok(())
    }

    fn relay_off(&mut self, relay: usize) -> Result<(), BoardError> {
        let mut state = self.state.borrow_mut();
        state.relay_state[relay] = false;
        state.relay_off_calls[relay] += 1;
        Ok(())
    }

    fn relay_toggle(&mut self, relay: usize) -> Result<(), BoardError> {
        let mut state = self.state.borrow_mut();
        state.relay_state[relay] = !state.relay_state[relay];
        Ok(())
    }

    fn read_button(&mut self, button: usize) -> Result<bool, BoardError> {
        Ok(self.state.borrow().buttons[button])
    }

    fn read_input(&mut self, input: usize) -> Result<bool, BoardError> {
        Ok(self.state.borrow().inputs[input])
    }

    fn tick(&mut self) -> Result<(), BoardError> {
        self.state.borrow_mut().tick_calls += 1;
        Ok(())
    }
}

fn new_controller() -> (Io22d08Controller<FakeBoard>, Rc<RefCell<FakeBoardState>>) {
    let state = Rc::new(RefCell::new(FakeBoardState::default()));
    let board = FakeBoard {
        state: state.clone(),
    };
    (Io22d08Controller::new(board), state)
}

fn tick_n(ctrl: &mut Io22d08Controller<FakeBoard>, n: u32) {
    for _ in 0..n {
        ctrl.sync_tick().unwrap();
    }
}

#[test]
fn boot_shows_the_speed_view_with_all_relays_off() {
    let (mut ctrl, state) = new_controller();
    ctrl.sync_tick().unwrap();

    let state = state.borrow();
    assert_eq!(state.tick_calls, 1);
    // Boot speed is 500ms -> "0500".
    assert_eq!(state.digits, [0, 5, 0, 0]);
    assert_eq!(state.relay_on_calls, [0; RELAY_COUNT]);
    assert_eq!(state.relay_off_calls, [0; RELAY_COUNT]);
}

#[test]
fn pressing_an_external_relay_key_turns_that_relay_on_once() {
    let (mut ctrl, state) = new_controller();
    state.borrow_mut().inputs[0] = true;
    tick_n(&mut ctrl, DEBOUNCE_SAMPLES);

    assert_eq!(state.borrow().relay_on_calls[0], 1);
    assert!(state.borrow().relay_state[0]);

    // Holding the key steady must not repeat the action.
    tick_n(&mut ctrl, 20);
    assert_eq!(state.borrow().relay_on_calls[0], 1);
}

#[test]
fn release_then_repress_toggles_the_relay_off_again() {
    let (mut ctrl, state) = new_controller();
    state.borrow_mut().inputs[0] = true;
    tick_n(&mut ctrl, DEBOUNCE_SAMPLES);
    assert!(state.borrow().relay_state[0]);

    state.borrow_mut().inputs[0] = false;
    tick_n(&mut ctrl, DEBOUNCE_SAMPLES);

    state.borrow_mut().inputs[0] = true;
    tick_n(&mut ctrl, DEBOUNCE_SAMPLES);

    assert!(!state.borrow().relay_state[0]);
    assert_eq!(state.borrow().relay_off_calls[0], 1);
}

#[test]
fn global_action_turns_on_all_included_relays_when_all_off() {
    let (mut ctrl, state) = new_controller();
    state.borrow_mut().inputs[5] = true; // I5 = Global, unshifted -> GlobalPower.
    tick_n(&mut ctrl, DEBOUNCE_SAMPLES);

    let state = state.borrow();
    assert_eq!(state.relay_on_calls, [1; RELAY_COUNT]);
    assert_eq!(state.relay_state, [true; RELAY_COUNT]);
}

#[test]
fn button_b1_selects_inputs_view_showing_the_stable_input_snapshot() {
    let (mut ctrl, state) = new_controller();
    state.borrow_mut().inputs[0] = true;
    tick_n(&mut ctrl, DEBOUNCE_SAMPLES);

    state.borrow_mut().buttons[1] = true;
    tick_n(&mut ctrl, DEBOUNCE_SAMPLES);

    assert_eq!(state.borrow().digits, [1, 0, 0, 0]);
}

#[test]
fn button_b3_is_a_reserved_no_op() {
    let (mut ctrl, state) = new_controller();
    ctrl.sync_tick().unwrap();
    let boot_digits = state.borrow().digits;

    state.borrow_mut().buttons[3] = true;
    tick_n(&mut ctrl, DEBOUNCE_SAMPLES);

    assert_eq!(state.borrow().digits, boot_digits);
}

#[test]
fn a_relay_in_blink_mode_independently_completes_its_half_cycle() {
    let (mut ctrl, state) = new_controller();
    // Shift (I4) + relay key I0 accepted on the same debounce window ->
    // shifted ToggleBlink targeting relay 4.
    {
        let mut state = state.borrow_mut();
        state.inputs[4] = true;
        state.inputs[0] = true;
    }
    tick_n(&mut ctrl, DEBOUNCE_SAMPLES);

    assert!(state.borrow().relay_state[4], "entering Blink turns the relay on immediately");
    assert_eq!(state.borrow().relay_on_calls[4], 1);

    // Keep holding steady; the relay must flip off on its own after one
    // independent half-cycle, with no further input edges involved.
    for _ in 0..600 {
        ctrl.sync_tick().unwrap();
        if !state.borrow().relay_state[4] {
            break;
        }
    }

    assert!(!state.borrow().relay_state[4]);
    assert_eq!(state.borrow().relay_off_calls[4], 1);
}
