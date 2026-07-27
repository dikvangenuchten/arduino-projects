use crate::config::{BUTTON_COUNT, DEBOUNCE_SAMPLES};
use crate::debug::ButtonInputs;

fn raw_with(pressed: &[usize]) -> [bool; BUTTON_COUNT] {
    let mut raw = [false; BUTTON_COUNT];
    for &i in pressed {
        raw[i] = true;
    }
    raw
}

fn settle(buttons: &mut ButtonInputs, raw: [bool; BUTTON_COUNT]) -> [bool; BUTTON_COUNT] {
    let mut edges = [false; BUTTON_COUNT];
    for _ in 0..DEBOUNCE_SAMPLES {
        edges = buttons.tick_1ms(raw);
    }
    edges
}

#[test]
fn press_edge_fires_once_when_debounce_accepts() {
    let mut buttons = ButtonInputs::new();
    let edges = settle(&mut buttons, raw_with(&[0]));
    assert_eq!(edges, [true, false, false, false]);
}

#[test]
fn held_button_does_not_repeat_the_edge() {
    let mut buttons = ButtonInputs::new();
    settle(&mut buttons, raw_with(&[0]));

    for _ in 0..20 {
        let edges = buttons.tick_1ms(raw_with(&[0]));
        assert_eq!(edges, [false; BUTTON_COUNT]);
    }
}

#[test]
fn release_then_repress_fires_the_edge_again() {
    let mut buttons = ButtonInputs::new();
    settle(&mut buttons, raw_with(&[0]));
    settle(&mut buttons, raw_with(&[]));

    let edges = settle(&mut buttons, raw_with(&[0]));
    assert_eq!(edges, [true, false, false, false]);
}
