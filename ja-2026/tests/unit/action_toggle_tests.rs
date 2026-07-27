use crate::action::{apply_relay_action, Action};
use crate::relay::{RelayBank, RelayMode};

#[test]
fn test_toggle_power_off_to_on() {
    let mut bank = RelayBank::new();
    bank.set_mode(0, RelayMode::Off);
    apply_relay_action(&mut bank, Action::TogglePower(0));
    assert_eq!(bank.relay(0).mode, RelayMode::On);
}

#[test]
fn test_toggle_power_on_to_off() {
    let mut bank = RelayBank::new();
    bank.set_mode(0, RelayMode::On);
    apply_relay_action(&mut bank, Action::TogglePower(0));
    assert_eq!(bank.relay(0).mode, RelayMode::Off);
}

#[test]
fn test_toggle_power_blink_to_off() {
    let mut bank = RelayBank::new();
    bank.set_mode(0, RelayMode::Blink);
    apply_relay_action(&mut bank, Action::TogglePower(0));
    assert_eq!(bank.relay(0).mode, RelayMode::Off);
}

#[test]
fn test_toggle_blink_off_to_blink() {
    let mut bank = RelayBank::new();
    bank.set_mode(0, RelayMode::Off);
    apply_relay_action(&mut bank, Action::ToggleBlink(0));
    assert_eq!(bank.relay(0).mode, RelayMode::Blink);
}

#[test]
fn test_toggle_blink_on_to_blink() {
    let mut bank = RelayBank::new();
    bank.set_mode(0, RelayMode::On);
    apply_relay_action(&mut bank, Action::ToggleBlink(0));
    assert_eq!(bank.relay(0).mode, RelayMode::Blink);
}

#[test]
fn test_toggle_blink_blink_to_on() {
    let mut bank = RelayBank::new();
    bank.set_mode(0, RelayMode::Blink);
    apply_relay_action(&mut bank, Action::ToggleBlink(0));
    assert_eq!(bank.relay(0).mode, RelayMode::On);
}
