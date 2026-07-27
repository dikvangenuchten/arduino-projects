//! Action domain and dispatcher.
//!
//! Phase 1: Semantic actions and pure mode/speed transition functions.
//! Every relay independently owns its speed; no shared state.

use crate::config::{GLOBAL_MODE_INCLUSION_MASK, RELAY_COUNT, SPEED_LEVELS_MS};
use crate::relay::{RelayBank, RelayMode};

/// Semantic actions resolved from input mapping.
/// No ActionContext needed; actions are pure and stateless.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Action {
    /// Toggle single relay power (Off <-> On), leaving Blink unchanged.
    TogglePower(usize),
    /// Toggle single relay blink (Off/On <-> Blink).
    ToggleBlink(usize),
    /// Set all included relays On if currently all Off, else all Off.
    GlobalPower,
    /// Force all included relays to Blink, preserving their speeds.
    GlobalBlink,
    /// Increment all relay speeds toward shorter durations.
    SpeedAllUp,
    /// Decrement all relay speeds toward longer durations.
    SpeedAllDown,
    /// Reserved/no-op action.
    Reserved,
}

/// Find the speed level (index into SPEED_LEVELS_MS) after a speed action.
///
/// # Direction semantics (SPEED_LEVELS_MS = [1200, 800, 500, 300, 150], descending):
/// - `up=true`: move toward shorter durations (index increases toward 150).
///   - If between levels, move to the shorter level.
///   - If exactly on a level, move to the next shorter level.
///   - Clamp at 150 (index 4).
/// - `up=false`: move toward longer durations (index decreases toward 1200).
///   - If between levels, move to the longer level.
///   - If exactly on a level, move to the next longer level.
///   - Clamp at 1200 (index 0).
fn nearest_speed_index(current_speed_ms: u16, up: bool) -> usize {
    // SPEED_LEVELS_MS = [1200, 800, 500, 300, 150] (descending)
    // Manual linear search (only 5 elements).
    let levels = SPEED_LEVELS_MS;
    
    if up {
        // Moving toward shorter (faster). Find the next level that is shorter.
        for (i, &level) in levels.iter().enumerate() {
            if current_speed_ms > level {
                // current is between levels[i-1] and levels[i].
                // Return i (the shorter level).
                return i;
            } else if current_speed_ms == level {
                // current is exactly on levels[i].
                // Return the next shorter level (i+1), clamped at the last index.
                return (i + 1).min(levels.len() - 1);
            }
        }
        // current is <= all levels (i.e., <= 150, the shortest).
        // Stay at the shortest.
        levels.len() - 1
    } else {
        // Moving toward longer (slower). Find the next level that is longer.
        for (i, &level) in levels.iter().enumerate().rev() {
            if current_speed_ms < level {
                // current is between levels[i] and levels[i+1] (or below levels[i] if i is last).
                // Return i (the longer level).
                return i;
            } else if current_speed_ms == level {
                // current is exactly on levels[i].
                // Return the next longer level (i-1), clamped at 0.
                return if i == 0 { 0 } else { i - 1 };
            }
        }
        // current >= all levels (i.e., >= 1200, the longest).
        // Stay at the longest.
        0
    }
}

/// Apply a relay action: single relay mode changes.
pub fn apply_relay_action(bank: &mut RelayBank, action: Action) {
    match action {
        Action::TogglePower(relay_idx) => {
            if relay_idx < RELAY_COUNT {
                let relay = bank.relay(relay_idx);
                let new_mode = match relay.mode {
                    RelayMode::Off => RelayMode::On,
                    RelayMode::On | RelayMode::Blink => RelayMode::Off,
                };
                bank.set_mode(relay_idx, new_mode);
            }
        }
        Action::ToggleBlink(relay_idx) => {
            if relay_idx < RELAY_COUNT {
                let relay = bank.relay(relay_idx);
                let new_mode = match relay.mode {
                    RelayMode::Off | RelayMode::On => RelayMode::Blink,
                    RelayMode::Blink => RelayMode::On,
                };
                bank.set_mode(relay_idx, new_mode);
            }
        }
        _ => {}
    }
}

/// Apply a global mode action: affects all included relays.
pub fn apply_global_action(bank: &mut RelayBank, action: Action) {
    match action {
        Action::GlobalPower => {
            // Check if all included relays are Off.
            let all_off = (0..RELAY_COUNT).all(|i| {
                if (GLOBAL_MODE_INCLUSION_MASK & (1 << i as u8)) != 0 {
                    bank.relay(i).mode == RelayMode::Off
                } else {
                    true
                }
            });

            let new_mode = if all_off { RelayMode::On } else { RelayMode::Off };
            for i in 0..RELAY_COUNT {
                if (GLOBAL_MODE_INCLUSION_MASK & (1 << i as u8)) != 0 {
                    bank.set_mode(i, new_mode);
                }
            }
        }
        Action::GlobalBlink => {
            // Force all included relays to Blink, preserving their speeds.
            for i in 0..RELAY_COUNT {
                if (GLOBAL_MODE_INCLUSION_MASK & (1 << i as u8)) != 0 {
                    bank.set_mode(i, RelayMode::Blink);
                }
            }
        }
        _ => {}
    }
}

/// Apply a speed action: adjust all relay speeds independently.
/// SpeedAllUp moves each relay toward shorter durations.
/// SpeedAllDown moves each relay toward longer durations.
/// Each relay is clamped to its nearest level independently.
pub fn apply_speed_action(bank: &mut RelayBank, action: Action) {
    match action {
        Action::SpeedAllUp => {
            for i in 0..RELAY_COUNT {
                let relay = bank.relay(i);
                let new_idx = nearest_speed_index(relay.speed_ms, true);
                let new_speed_ms = SPEED_LEVELS_MS[new_idx];
                bank.set_speed(i, new_speed_ms);
            }
        }
        Action::SpeedAllDown => {
            for i in 0..RELAY_COUNT {
                let relay = bank.relay(i);
                let new_idx = nearest_speed_index(relay.speed_ms, false);
                let new_speed_ms = SPEED_LEVELS_MS[new_idx];
                bank.set_speed(i, new_speed_ms);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_global_power_all_off_to_on() {
        let mut bank = RelayBank::new();
        // All relays are Off by default.
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
    fn test_speed_all_up_from_boot_speeds() {
        let mut bank = RelayBank::new();
        // All relays boot with 500 ms.
        apply_speed_action(&mut bank, Action::SpeedAllUp);
        for i in 0..RELAY_COUNT {
            assert_eq!(bank.relay(i).speed_ms, 300); // Next level up
        }
    }

    #[test]
    fn test_speed_all_up_mixed_speeds() {
        let mut bank = RelayBank::new();
        bank.set_speed(0, 1200);
        bank.set_speed(1, 500);
        bank.set_speed(2, 150);
        apply_speed_action(&mut bank, Action::SpeedAllUp);
        assert_eq!(bank.relay(0).speed_ms, 800); // 1200 -> 800
        assert_eq!(bank.relay(1).speed_ms, 300); // 500 -> 300
        assert_eq!(bank.relay(2).speed_ms, 150); // 150 stays at 150
    }

    #[test]
    fn test_speed_all_down_from_boot_speeds() {
        let mut bank = RelayBank::new();
        // All relays boot with 500 ms.
        apply_speed_action(&mut bank, Action::SpeedAllDown);
        for i in 0..RELAY_COUNT {
            assert_eq!(bank.relay(i).speed_ms, 800); // Next level down
        }
    }

    #[test]
    fn test_speed_all_down_mixed_speeds() {
        let mut bank = RelayBank::new();
        bank.set_speed(0, 1200);
        bank.set_speed(1, 500);
        bank.set_speed(2, 150);
        apply_speed_action(&mut bank, Action::SpeedAllDown);
        assert_eq!(bank.relay(0).speed_ms, 1200); // 1200 stays at 1200
        assert_eq!(bank.relay(1).speed_ms, 800); // 500 -> 800
        assert_eq!(bank.relay(2).speed_ms, 300); // 150 -> 300
    }

    #[test]
    fn test_global_inclusion_mask_exclusion() {
        let original_mask = GLOBAL_MODE_INCLUSION_MASK;
        // This test documents that GLOBAL_MODE_INCLUSION_MASK is 0xFF (all relays).
        // A future test with a different mask would verify exclusion behavior.
        assert_eq!(original_mask, 0xFF);
    }

    #[test]
    fn test_speed_action_with_non_level_values() {
        let mut bank = RelayBank::new();
        bank.set_speed(0, 301); // Between 300 and 500
        bank.set_speed(1, 499);
        apply_speed_action(&mut bank, Action::SpeedAllUp);
        assert_eq!(bank.relay(0).speed_ms, 300); // 301 up -> 300
        assert_eq!(bank.relay(1).speed_ms, 300); // 499 up -> 300
    }

    #[test]
    fn test_speed_down_with_non_level_values() {
        let mut bank = RelayBank::new();
        bank.set_speed(0, 301); // Between 300 and 500
        bank.set_speed(1, 499);
        apply_speed_action(&mut bank, Action::SpeedAllDown);
        assert_eq!(bank.relay(0).speed_ms, 500); // 301 down -> 500
        assert_eq!(bank.relay(1).speed_ms, 500); // 499 down -> 500
    }
}
