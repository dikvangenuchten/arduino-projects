//! Relay domain model and blink engine.
//!
//! Phase 1: RelayMode (Off/Blink/On), RelayState, and concrete RelayBank.
//! Each relay owns its mode and raw speed (ms).
//! Phase 2+: Blink engine with per-relay timing.

use crate::config::{BOOT_SPEED_MS, RELAY_COUNT, SPEED_LEVELS_MS};

/// Per-relay blink timing policy.
///
/// Kept as an enum so a future Random policy can be added without
/// spreading duration selection logic throughout the engine.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum BlinkTimingPolicy {
    FixedLevels,
}

impl BlinkTimingPolicy {
    fn next_half_cycle_ticks(self, speed_ms: u16) -> u16 {
        match self {
            BlinkTimingPolicy::FixedLevels => fixed_half_cycle_ticks(speed_ms),
        }
    }
}

fn fixed_half_cycle_ticks(speed_ms: u16) -> u16 {
    if SPEED_LEVELS_MS.contains(&speed_ms) {
        return speed_ms.max(1);
    }

    // Guard against non-level values by selecting the nearest supported value.
    let mut best = SPEED_LEVELS_MS[0];
    let mut best_delta = speed_ms.abs_diff(best);
    for &candidate in SPEED_LEVELS_MS.iter().skip(1) {
        let delta = speed_ms.abs_diff(candidate);
        if delta < best_delta || (delta == best_delta && candidate > best) {
            best = candidate;
            best_delta = delta;
        }
    }
    best.max(1)
}

/// Operating mode of a single relay.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RelayMode {
    /// Relay is off.
    Off,
    /// Relay is blinking at its stored speed.
    Blink,
    /// Relay is on (steady).
    On,
}

/// State of a single relay: its mode and speed.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct RelayState {
    /// Operating mode (Off, Blink, or On).
    pub mode: RelayMode,
    /// Speed in milliseconds (raw ticks, one of [1200, 800, 500, 300, 150]).
    pub speed_ms: u16,
    output_on: bool,
    ticks_remaining: u16,
    timing_policy: BlinkTimingPolicy,
}

impl RelayState {
    /// Construct a relay in Off mode with the boot speed.
    pub fn new() -> Self {
        RelayState {
            mode: RelayMode::Off,
            speed_ms: BOOT_SPEED_MS,
            output_on: false,
            ticks_remaining: 0,
            timing_policy: BlinkTimingPolicy::FixedLevels,
        }
    }

    /// Construct a relay with a specific mode and speed.
    pub fn with_mode_and_speed(mode: RelayMode, speed_ms: u16) -> Self {
        let mut state = RelayState {
            mode: RelayMode::Off,
            speed_ms,
            output_on: false,
            ticks_remaining: 0,
            timing_policy: BlinkTimingPolicy::FixedLevels,
        };
        state.set_mode(mode);
        state
    }

    fn set_mode(&mut self, mode: RelayMode) {
        if self.mode == mode {
            return;
        }

        match mode {
            RelayMode::Off => {
                self.mode = RelayMode::Off;
                self.output_on = false;
                self.ticks_remaining = 0;
            }
            RelayMode::On => {
                self.mode = RelayMode::On;
                self.output_on = true;
                self.ticks_remaining = 0;
            }
            RelayMode::Blink => {
                self.mode = RelayMode::Blink;
                self.output_on = true;
                self.ticks_remaining = self.timing_policy.next_half_cycle_ticks(self.speed_ms);
            }
        }
    }

    fn tick_1ms(&mut self) {
        if self.mode != RelayMode::Blink {
            return;
        }

        if self.ticks_remaining > 1 {
            self.ticks_remaining -= 1;
            return;
        }

        // For a completed (or malformed zero) half-cycle, transition phase and
        // load a fresh duration from the relay's current stored speed.
        self.output_on = !self.output_on;
        self.ticks_remaining = self.timing_policy.next_half_cycle_ticks(self.speed_ms);
    }
}

impl Default for RelayState {
    fn default() -> Self {
        Self::new()
    }
}

/// Concrete relay bank: array of 8 independently-controlled relays.
/// No shared state; each relay owns its mode and speed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelayBank {
    relays: [RelayState; RELAY_COUNT],
}

impl RelayBank {
    /// Construct a new relay bank with all relays Off and boot speed.
    pub fn new() -> Self {
        RelayBank {
            relays: [RelayState::new(); RELAY_COUNT],
        }
    }

    /// Get the state of relay `idx` (0-indexed).
    /// Panics if `idx >= RELAY_COUNT`.
    pub fn relay(&self, idx: usize) -> RelayState {
        self.relays[idx]
    }

    /// Get a mutable reference to relay `idx` for direct state manipulation.
    /// Panics if `idx >= RELAY_COUNT`.
    /// (Used internally by action dispatchers.)
    fn relay_mut(&mut self, idx: usize) -> &mut RelayState {
        &mut self.relays[idx]
    }

    /// Set relay `idx` to a new mode, preserving its speed.
    /// Panics if `idx >= RELAY_COUNT`.
    pub fn set_mode(&mut self, idx: usize, mode: RelayMode) {
        self.relay_mut(idx).set_mode(mode);
    }

    /// Set relay `idx` to a new speed, preserving its mode.
    /// Panics if `idx >= RELAY_COUNT`.
    pub fn set_speed(&mut self, idx: usize, speed_ms: u16) {
        self.relay_mut(idx).speed_ms = speed_ms;
    }

    /// Current commanded output for relay `idx`.
    /// Panics if `idx >= RELAY_COUNT`.
    pub fn relay_output(&self, idx: usize) -> bool {
        self.relays[idx].output_on
    }

    /// Advance all relay blink engines by one synthetic 1 ms tick.
    pub fn tick_1ms(&mut self) {
        for relay in self.relays.iter_mut() {
            relay.tick_1ms();
        }
    }

    /// Iterate over all relay states.
    pub fn relays(&self) -> &[RelayState; RELAY_COUNT] {
        &self.relays
    }

    /// Get a reference to the internal array as a slice.
    pub fn as_slice(&self) -> &[RelayState] {
        &self.relays
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tick_n(bank: &mut RelayBank, ticks: u16) {
        for _ in 0..ticks {
            bank.tick_1ms();
        }
    }

    #[test]
    fn entering_blink_starts_on_with_full_half_cycle() {
        let mut bank = RelayBank::new();
        bank.set_mode(0, RelayMode::Blink);
        assert!(bank.relay_output(0));

        tick_n(&mut bank, 499);
        assert!(bank.relay_output(0));

        bank.tick_1ms();
        assert!(!bank.relay_output(0));
    }

    #[test]
    fn leaving_blink_immediately_sets_commanded_steady_output() {
        let mut bank = RelayBank::new();
        bank.set_mode(0, RelayMode::Blink);
        tick_n(&mut bank, 123);
        assert!(bank.relay_output(0));

        bank.set_mode(0, RelayMode::Off);
        assert!(!bank.relay_output(0));

        bank.set_mode(0, RelayMode::On);
        assert!(bank.relay_output(0));
    }

    #[test]
    fn reentering_blink_from_non_blink_resets_phase_origin() {
        let mut bank = RelayBank::new();
        bank.set_mode(0, RelayMode::Blink);
        tick_n(&mut bank, 200);

        bank.set_mode(0, RelayMode::On);
        bank.set_mode(0, RelayMode::Blink);
        assert!(bank.relay_output(0));

        tick_n(&mut bank, 499);
        assert!(bank.relay_output(0));

        bank.tick_1ms();
        assert!(!bank.relay_output(0));
    }

    #[test]
    fn reapplying_blink_while_already_blinking_keeps_current_phase() {
        let mut bank = RelayBank::new();
        bank.set_mode(0, RelayMode::Blink);
        tick_n(&mut bank, 200);

        bank.set_mode(0, RelayMode::Blink);
        tick_n(&mut bank, 299);
        assert!(bank.relay_output(0));

        bank.tick_1ms();
        assert!(!bank.relay_output(0));
    }

    #[test]
    fn speed_change_applies_on_next_phase_transition() {
        let mut bank = RelayBank::new();
        bank.set_speed(0, 500);
        bank.set_mode(0, RelayMode::Blink);

        tick_n(&mut bank, 200);
        bank.set_speed(0, 300);

        tick_n(&mut bank, 299);
        assert!(bank.relay_output(0));

        bank.tick_1ms();
        assert!(!bank.relay_output(0));

        tick_n(&mut bank, 299);
        assert!(!bank.relay_output(0));

        bank.tick_1ms();
        assert!(bank.relay_output(0));
    }

    #[test]
    fn relays_keep_independent_phase_offsets() {
        let mut bank = RelayBank::new();
        bank.set_mode(0, RelayMode::Blink);
        tick_n(&mut bank, 200);
        bank.set_mode(1, RelayMode::Blink);

        tick_n(&mut bank, 299);
        assert!(bank.relay_output(0));
        assert!(bank.relay_output(1));

        bank.tick_1ms();
        assert!(!bank.relay_output(0));
        assert!(bank.relay_output(1));

        tick_n(&mut bank, 200);
        assert!(!bank.relay_output(0));
        assert!(!bank.relay_output(1));
    }
}
