use crate::action::Action;
use crate::config::{ShiftMode, INPUT_COUNT, SHIFT_MODE};
use crate::input::{map_edges, resolve_shifted};

fn edges_with(pressed: &[usize]) -> [bool; INPUT_COUNT] {
    let mut edges = [false; INPUT_COUNT];
    for &i in pressed {
        edges[i] = true;
    }
    edges
}

#[test]
fn shift_mode_is_momentary_by_default() {
    assert_eq!(SHIFT_MODE, ShiftMode::Momentary);
}

#[test]
fn momentary_shift_resolves_directly_from_the_stable_level() {
    assert!(!resolve_shifted(ShiftMode::Momentary, false));
    assert!(resolve_shifted(ShiftMode::Momentary, true));
}

#[test]
fn relay_keys_unshifted_toggle_power_on_relays_zero_to_three() {
    let edges = edges_with(&[0, 1, 2, 3]);
    let actions = map_edges(edges, false);
    assert_eq!(actions[0], Some(Action::TogglePower(0)));
    assert_eq!(actions[1], Some(Action::TogglePower(1)));
    assert_eq!(actions[2], Some(Action::TogglePower(2)));
    assert_eq!(actions[3], Some(Action::TogglePower(3)));
}

#[test]
fn relay_keys_shifted_toggle_blink_on_relays_four_to_seven() {
    let edges = edges_with(&[0, 1, 2, 3]);
    let actions = map_edges(edges, true);
    assert_eq!(actions[0], Some(Action::ToggleBlink(4)));
    assert_eq!(actions[1], Some(Action::ToggleBlink(5)));
    assert_eq!(actions[2], Some(Action::ToggleBlink(6)));
    assert_eq!(actions[3], Some(Action::ToggleBlink(7)));
}

#[test]
fn global_key_unshifted_emits_global_power() {
    let edges = edges_with(&[5]);
    let actions = map_edges(edges, false);
    assert_eq!(actions[5], Some(Action::GlobalPower));
}

#[test]
fn global_key_shifted_emits_global_blink() {
    let edges = edges_with(&[5]);
    let actions = map_edges(edges, true);
    assert_eq!(actions[5], Some(Action::GlobalBlink));
}

#[test]
fn speed_key_unshifted_emits_speed_all_up() {
    let edges = edges_with(&[6]);
    let actions = map_edges(edges, false);
    assert_eq!(actions[6], Some(Action::SpeedAllUp));
}

#[test]
fn speed_key_shifted_emits_speed_all_down() {
    let edges = edges_with(&[6]);
    let actions = map_edges(edges, true);
    assert_eq!(actions[6], Some(Action::SpeedAllDown));
}

#[test]
fn reserved_key_always_emits_reserved_regardless_of_shift() {
    let edges = edges_with(&[7]);
    assert_eq!(map_edges(edges, false)[7], Some(Action::Reserved));
    assert_eq!(map_edges(edges, true)[7], Some(Action::Reserved));
}

#[test]
fn shift_line_itself_never_emits_an_action() {
    let edges = edges_with(&[4]);
    assert_eq!(map_edges(edges, false)[4], None);
    assert_eq!(map_edges(edges, true)[4], None);
}

#[test]
fn lines_without_a_press_edge_emit_no_action() {
    let edges = [false; INPUT_COUNT];
    let actions = map_edges(edges, false);
    assert_eq!(actions, [None; INPUT_COUNT]);
}

#[test]
fn simultaneous_edges_each_map_independently() {
    let edges = edges_with(&[0, 5, 6]);
    let actions = map_edges(edges, false);
    assert_eq!(actions[0], Some(Action::TogglePower(0)));
    assert_eq!(actions[5], Some(Action::GlobalPower));
    assert_eq!(actions[6], Some(Action::SpeedAllUp));
}
