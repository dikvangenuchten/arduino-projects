//! Configuration: single source of truth for indices and constants.
//!
//! Phase 1: Behavioral constants and fixed-size array dimensions.
//! Relay/button/input counts, speed levels, default speed, global inclusion mask.

/// Total number of relays controlled by the system.
pub const RELAY_COUNT: usize = 8;

/// Total number of built-in buttons (diagnostics-only).
pub const BUTTON_COUNT: usize = 4;

/// Total number of external inputs (control actions).
pub const INPUT_COUNT: usize = 8;

/// Half-cycle duration table for relay blinking, in milliseconds.
/// Indexed by speed level [0..4], where level 2 is the boot default.
pub const SPEED_LEVELS_MS: &[u16] = &[1200, 800, 500, 300, 150];

/// Boot speed level for all relays (index into SPEED_LEVELS_MS).
/// Level 2 = 500 ms.
pub const BOOT_SPEED_LEVEL: usize = 2;

/// Boot speed value in milliseconds for all relays.
pub const BOOT_SPEED_MS: u16 = SPEED_LEVELS_MS[BOOT_SPEED_LEVEL];

/// Bitmask of relays included in global mode actions (GlobalPower, GlobalBlink).
/// Bit N represents relay N (0-indexed).
/// Default includes all 8 relays.
pub const GLOBAL_MODE_INCLUSION_MASK: u8 = 0xFF; // binary 11111111

/// Debounce threshold: accept a change after this many consecutive 1ms samples.
/// Phase 1: placeholder; used in Phase 3.
pub const DEBOUNCE_SAMPLES: u32 = 10;
