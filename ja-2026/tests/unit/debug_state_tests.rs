use crate::config::BUTTON_COUNT;
use crate::debug::{DiagnosticState, DiagnosticView, Page};

fn edges_with(pressed: &[usize]) -> [bool; BUTTON_COUNT] {
    let mut edges = [false; BUTTON_COUNT];
    for &i in pressed {
        edges[i] = true;
    }
    edges
}

#[test]
fn boots_on_the_speed_view() {
    let state = DiagnosticState::new();
    assert_eq!(state.view(), DiagnosticView::Speed);
}

#[test]
fn b1_first_press_selects_inputs_view_at_first_page() {
    let mut state = DiagnosticState::new();
    state.apply_button_edges(edges_with(&[1]));
    assert_eq!(state.view(), DiagnosticView::Inputs);
    assert_eq!(state.inputs_page(), Page::First);
}

#[test]
fn repeated_b1_press_toggles_the_inputs_page() {
    let mut state = DiagnosticState::new();
    state.apply_button_edges(edges_with(&[1]));
    state.apply_button_edges(edges_with(&[1]));
    assert_eq!(state.inputs_page(), Page::Second);

    state.apply_button_edges(edges_with(&[1]));
    assert_eq!(state.inputs_page(), Page::First);
}

#[test]
fn b2_first_press_selects_relay_view_at_first_page() {
    let mut state = DiagnosticState::new();
    state.apply_button_edges(edges_with(&[2]));
    assert_eq!(state.view(), DiagnosticView::Relay);
    assert_eq!(state.relay_page(), Page::First);
}

#[test]
fn repeated_b2_press_toggles_the_relay_page() {
    let mut state = DiagnosticState::new();
    state.apply_button_edges(edges_with(&[2]));
    state.apply_button_edges(edges_with(&[2]));
    assert_eq!(state.relay_page(), Page::Second);

    state.apply_button_edges(edges_with(&[2]));
    assert_eq!(state.relay_page(), Page::First);
}

#[test]
fn switching_away_and_back_to_inputs_resets_to_first_page() {
    let mut state = DiagnosticState::new();
    state.apply_button_edges(edges_with(&[1])); // Inputs, page First
    state.apply_button_edges(edges_with(&[1])); // Inputs, page Second
    state.apply_button_edges(edges_with(&[2])); // Relay, page First
    state.apply_button_edges(edges_with(&[1])); // back to Inputs

    assert_eq!(state.view(), DiagnosticView::Inputs);
    assert_eq!(state.inputs_page(), Page::First);
}

#[test]
fn b0_press_returns_to_speed_view_from_any_view() {
    let mut state = DiagnosticState::new();
    state.apply_button_edges(edges_with(&[2]));
    assert_eq!(state.view(), DiagnosticView::Relay);

    state.apply_button_edges(edges_with(&[0]));
    assert_eq!(state.view(), DiagnosticView::Speed);
}

#[test]
fn b3_press_is_a_no_op() {
    let mut state = DiagnosticState::new();
    state.apply_button_edges(edges_with(&[1]));
    let before = state;

    state.apply_button_edges(edges_with(&[3]));
    assert_eq!(state, before);
}

#[test]
fn simultaneous_b0_and_b2_edges_apply_in_ascending_button_order() {
    let mut state = DiagnosticState::new();
    // B0 (Speed) and B2 (Relay) both accepted on the same tick: B2 is
    // processed after B0, so Relay wins.
    state.apply_button_edges(edges_with(&[0, 2]));
    assert_eq!(state.view(), DiagnosticView::Relay);
}
