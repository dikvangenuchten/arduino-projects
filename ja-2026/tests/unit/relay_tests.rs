use crate::config::BOOT_SPEED_MS;
use crate::relay::{RelayMode, RelayState};

#[test]
fn relay_state_new_defaults_to_off_and_boot_speed() {
    let relay = RelayState::new();
    assert_eq!(relay.mode, RelayMode::Off);
    assert_eq!(relay.speed_ms, BOOT_SPEED_MS);
}

#[test]
fn relay_state_with_mode_and_speed_sets_requested_state() {
    let relay = RelayState::with_mode_and_speed(RelayMode::On, 300);
    assert_eq!(relay.mode, RelayMode::On);
    assert_eq!(relay.speed_ms, 300);
}

#[test]
fn relay_state_default_matches_new() {
    let relay = RelayState::default();
    assert_eq!(relay.mode, RelayMode::Off);
    assert_eq!(relay.speed_ms, BOOT_SPEED_MS);
}
