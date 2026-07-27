use crate::action::{apply_speed_action, Action};
use crate::config::RELAY_COUNT;
use crate::relay::RelayBank;

#[test]
fn test_speed_all_up_from_boot_speeds() {
    let mut bank = RelayBank::new();
    apply_speed_action(&mut bank, Action::SpeedAllUp);
    for i in 0..RELAY_COUNT {
        assert_eq!(bank.relay(i).speed_ms, 300);
    }
}

#[test]
fn test_speed_all_up_mixed_speeds() {
    let mut bank = RelayBank::new();
    bank.set_speed(0, 1200);
    bank.set_speed(1, 500);
    bank.set_speed(2, 150);
    apply_speed_action(&mut bank, Action::SpeedAllUp);
    assert_eq!(bank.relay(0).speed_ms, 800);
    assert_eq!(bank.relay(1).speed_ms, 300);
    assert_eq!(bank.relay(2).speed_ms, 150);
}

#[test]
fn test_speed_all_down_from_boot_speeds() {
    let mut bank = RelayBank::new();
    apply_speed_action(&mut bank, Action::SpeedAllDown);
    for i in 0..RELAY_COUNT {
        assert_eq!(bank.relay(i).speed_ms, 800);
    }
}

#[test]
fn test_speed_all_down_mixed_speeds() {
    let mut bank = RelayBank::new();
    bank.set_speed(0, 1200);
    bank.set_speed(1, 500);
    bank.set_speed(2, 150);
    apply_speed_action(&mut bank, Action::SpeedAllDown);
    assert_eq!(bank.relay(0).speed_ms, 1200);
    assert_eq!(bank.relay(1).speed_ms, 800);
    assert_eq!(bank.relay(2).speed_ms, 300);
}

#[test]
fn test_speed_action_with_non_level_values() {
    let mut bank = RelayBank::new();
    bank.set_speed(0, 301);
    bank.set_speed(1, 499);
    apply_speed_action(&mut bank, Action::SpeedAllUp);
    assert_eq!(bank.relay(0).speed_ms, 300);
    assert_eq!(bank.relay(1).speed_ms, 300);
}

#[test]
fn test_speed_down_with_non_level_values() {
    let mut bank = RelayBank::new();
    bank.set_speed(0, 301);
    bank.set_speed(1, 499);
    apply_speed_action(&mut bank, Action::SpeedAllDown);
    assert_eq!(bank.relay(0).speed_ms, 500);
    assert_eq!(bank.relay(1).speed_ms, 500);
}
