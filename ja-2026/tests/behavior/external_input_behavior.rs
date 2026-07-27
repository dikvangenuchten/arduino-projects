use crate::action::Action;
use crate::config::{DEBOUNCE_SAMPLES, INPUT_COUNT};
use crate::input::ExternalInputs;

fn raw_with(pressed: &[usize]) -> [bool; INPUT_COUNT] {
    let mut raw = [false; INPUT_COUNT];
    for &i in pressed {
        raw[i] = true;
    }
    raw
}

/// Feed `raw` for `DEBOUNCE_SAMPLES` ticks, returning only the final tick's
/// actions (the tick on which debounce acceptance completes).
fn settle(inputs: &mut ExternalInputs, raw: [bool; INPUT_COUNT]) -> [Option<Action>; INPUT_COUNT] {
    let mut actions = [None; INPUT_COUNT];
    for _ in 0..DEBOUNCE_SAMPLES {
        actions = inputs.tick_1ms(raw);
    }
    actions
}

#[test]
fn press_edge_emits_action_once_when_debounce_accepts() {
    let mut inputs = ExternalInputs::new();
    let actions = settle(&mut inputs, raw_with(&[0]));
    assert_eq!(actions[0], Some(Action::TogglePower(0)));
}

#[test]
fn held_input_does_not_repeat_the_action() {
    let mut inputs = ExternalInputs::new();
    settle(&mut inputs, raw_with(&[0]));

    // Continue holding I0: no further edges, so no further actions.
    for _ in 0..20 {
        let actions = inputs.tick_1ms(raw_with(&[0]));
        assert_eq!(actions[0], None);
    }
}

#[test]
fn release_then_repress_emits_the_action_again() {
    let mut inputs = ExternalInputs::new();
    settle(&mut inputs, raw_with(&[0]));

    // Release.
    settle(&mut inputs, raw_with(&[]));

    // Repress.
    let actions = settle(&mut inputs, raw_with(&[0]));
    assert_eq!(actions[0], Some(Action::TogglePower(0)));
}

#[test]
fn bounce_before_threshold_delays_the_edge() {
    let mut inputs = ExternalInputs::new();

    // Almost reach threshold, then bounce back to idle.
    for _ in 0..(DEBOUNCE_SAMPLES - 1) {
        let actions = inputs.tick_1ms(raw_with(&[0]));
        assert_eq!(actions[0], None);
    }
    let actions = inputs.tick_1ms(raw_with(&[]));
    assert_eq!(actions[0], None);

    // A fresh full run is required before the edge appears.
    for _ in 0..(DEBOUNCE_SAMPLES - 1) {
        let actions = inputs.tick_1ms(raw_with(&[0]));
        assert_eq!(actions[0], None, "bounce must restart the debounce count");
    }
    let actions = inputs.tick_1ms(raw_with(&[0]));
    assert_eq!(actions[0], Some(Action::TogglePower(0)));
}

#[test]
fn simultaneous_presses_each_emit_their_own_action_on_the_same_tick() {
    let mut inputs = ExternalInputs::new();
    let actions = settle(&mut inputs, raw_with(&[0, 5, 6]));
    assert_eq!(actions[0], Some(Action::TogglePower(0)));
    assert_eq!(actions[5], Some(Action::GlobalPower));
    assert_eq!(actions[6], Some(Action::SpeedAllUp));
}

#[test]
fn shift_and_action_accepted_on_the_same_tick_are_shifted() {
    let mut inputs = ExternalInputs::new();
    // I4 (Shift) and I0 both cross the debounce threshold on the identical tick.
    let actions = settle(&mut inputs, raw_with(&[4, 0]));
    assert_eq!(actions[0], Some(Action::TogglePower(4)));
    // Shift itself never emits an action.
    assert_eq!(actions[4], None);
}

#[test]
fn shift_already_held_applies_to_a_later_action_press() {
    let mut inputs = ExternalInputs::new();
    // Shift is accepted first, held afterward.
    settle(&mut inputs, raw_with(&[4]));

    // I0 is pressed later, on its own debounce run, while Shift is still held.
    let actions = settle(&mut inputs, raw_with(&[4, 0]));
    assert_eq!(actions[0], Some(Action::TogglePower(4)));
}

#[test]
fn releasing_shift_before_the_action_press_is_unshifted() {
    let mut inputs = ExternalInputs::new();
    settle(&mut inputs, raw_with(&[4]));
    settle(&mut inputs, raw_with(&[]));

    let actions = settle(&mut inputs, raw_with(&[0]));
    assert_eq!(actions[0], Some(Action::TogglePower(0)));
}

#[test]
fn stable_reflects_the_last_committed_debounced_snapshot() {
    let mut inputs = ExternalInputs::new();
    assert_eq!(inputs.stable(), [false; INPUT_COUNT]);

    settle(&mut inputs, raw_with(&[0, 2]));
    assert_eq!(inputs.stable(), raw_with(&[0, 2]));
}
