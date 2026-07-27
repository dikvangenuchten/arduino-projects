use crate::action::{apply_global_action, Action};
use crate::config::{GLOBAL_MODE_INCLUSION_MASK, RELAY_COUNT};
use crate::relay::{RelayBank, RelayMode};

#[test]
fn test_global_power_all_off_to_on() {
    let mut bank = RelayBank::new();
    apply_global_action(&mut bank, Action::GlobalPower);
    for i in 0..RELAY_COUNT {
        assert_eq!(bank.relay(i).mode, RelayMode::On);
    }
}

#[test]
fn test_global_power_mixed_to_off() {
    let mut bank = RelayBank::new();
    bank.set_mode(0, RelayMode::On);
    bank.set_mode(1, RelayMode::Off);
    apply_global_action(&mut bank, Action::GlobalPower);
    for i in 0..RELAY_COUNT {
        assert_eq!(bank.relay(i).mode, RelayMode::Off);
    }
}

#[test]
fn test_global_blink_all_relays() {
    let mut bank = RelayBank::new();
    bank.set_mode(0, RelayMode::On);
    bank.set_mode(1, RelayMode::Off);
    apply_global_action(&mut bank, Action::GlobalBlink);
    for i in 0..RELAY_COUNT {
        assert_eq!(bank.relay(i).mode, RelayMode::Blink);
    }
}

#[test]
fn test_global_blink_preserves_speeds() {
    let mut bank = RelayBank::new();
    bank.set_speed(0, 300);
    bank.set_speed(1, 150);
    apply_global_action(&mut bank, Action::GlobalBlink);
    assert_eq!(bank.relay(0).speed_ms, 300);
    assert_eq!(bank.relay(1).speed_ms, 150);
}

#[test]
fn test_global_inclusion_mask_exclusion() {
    assert_eq!(GLOBAL_MODE_INCLUSION_MASK, 0xFF);
}
