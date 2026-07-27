//! Relay domain model and blink engine.
//!
//! Phase 1: RelayMode (Off/Blink/On), RelayState, and concrete RelayBank.
//! Each relay owns its mode and raw speed (ms).
//! Phase 2+: Blink engine with per-relay timing.

use crate::config::{BOOT_SPEED_MS, RELAY_COUNT};

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
}

impl RelayState {
    /// Construct a relay in Off mode with the boot speed.
    pub fn new() -> Self {
        RelayState {
            mode: RelayMode::Off,
            speed_ms: BOOT_SPEED_MS,
        }
    }

    /// Construct a relay with a specific mode and speed.
    pub fn with_mode_and_speed(mode: RelayMode, speed_ms: u16) -> Self {
        RelayState { mode, speed_ms }
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
        self.relay_mut(idx).mode = mode;
    }

    /// Set relay `idx` to a new speed, preserving its mode.
    /// Panics if `idx >= RELAY_COUNT`.
    pub fn set_speed(&mut self, idx: usize, speed_ms: u16) {
        self.relay_mut(idx).speed_ms = speed_ms;
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
