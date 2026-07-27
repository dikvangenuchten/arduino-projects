use crate::config::{INPUT_COUNT, RELAY_COUNT};
use crate::debug::{render, DiagnosticState};
use crate::relay::{RelayBank, RelayMode};

fn bank_with_speeds(speeds: [u16; RELAY_COUNT]) -> RelayBank {
    let mut bank = RelayBank::new();
    for (i, &speed) in speeds.iter().enumerate() {
        bank.set_mode(i, RelayMode::Blink);
        bank.set_speed(i, speed);
    }
    bank
}

fn inputs_with(set: &[usize]) -> [bool; INPUT_COUNT] {
    let mut inputs = [false; INPUT_COUNT];
    for &i in set {
        inputs[i] = true;
    }
    inputs
}

#[test]
fn speed_view_renders_modal_value_as_four_digits() {
    let state = DiagnosticState::new(); // Speed view by default.
    let bank = bank_with_speeds([800, 800, 800, 800, 800, 800, 800, 800]);
    let digits = render(&state, &bank, [false; INPUT_COUNT]);
    assert_eq!(digits, [0, 8, 0, 0]);
}

#[test]
fn speed_view_renders_1200_without_truncation() {
    let state = DiagnosticState::new();
    let bank = bank_with_speeds([1200, 1200, 1200, 1200, 1200, 1200, 1200, 1200]);
    let digits = render(&state, &bank, [false; INPUT_COUNT]);
    assert_eq!(digits, [1, 2, 0, 0]);
}

#[test]
fn inputs_view_first_page_renders_i0_to_i3() {
    let mut state = DiagnosticState::new();
    state.apply_button_edges([false, true, false, false]); // select Inputs
    let bank = RelayBank::new();

    let digits = render(&state, &bank, inputs_with(&[0, 2]));
    assert_eq!(digits, [1, 0, 1, 0]);
}

#[test]
fn inputs_view_second_page_renders_i4_to_i7() {
    let mut state = DiagnosticState::new();
    state.apply_button_edges([false, true, false, false]); // Inputs, page First
    state.apply_button_edges([false, true, false, false]); // Inputs, page Second
    let bank = RelayBank::new();

    let digits = render(&state, &bank, inputs_with(&[4, 7]));
    assert_eq!(digits, [1, 0, 0, 1]);
}

#[test]
fn relay_view_first_page_renders_modes_as_0_1_2() {
    let mut state = DiagnosticState::new();
    state.apply_button_edges([false, false, true, false]); // select Relay
    let mut bank = RelayBank::new();
    bank.set_mode(0, RelayMode::Off);
    bank.set_mode(1, RelayMode::Blink);
    bank.set_mode(2, RelayMode::On);
    bank.set_mode(3, RelayMode::Off);

    let digits = render(&state, &bank, [false; INPUT_COUNT]);
    assert_eq!(digits, [0, 1, 2, 0]);
}

#[test]
fn relay_view_second_page_renders_relays_four_to_seven() {
    let mut state = DiagnosticState::new();
    state.apply_button_edges([false, false, true, false]); // Relay, page First
    state.apply_button_edges([false, false, true, false]); // Relay, page Second
    let mut bank = RelayBank::new();
    bank.set_mode(4, RelayMode::On);
    bank.set_mode(5, RelayMode::On);
    bank.set_mode(6, RelayMode::Blink);
    bank.set_mode(7, RelayMode::Off);

    let digits = render(&state, &bank, [false; INPUT_COUNT]);
    assert_eq!(digits, [2, 2, 1, 0]);
}
