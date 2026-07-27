#![cfg_attr(not(test), no_std)]

//! Pure, hardware-independent control core for the relay/blink system.
//!
//! This library implements the relay control architecture as pure functions
//! and data structures, decoupled from hardware specifics (AVR, GPIO, etc.).
//! This allows comprehensive unit testing via host-side `cargo test` without
//! requiring an embedded target.
//!
//! **Test-Driven Development**: Each phase (1-8) defines its behavior contract
//! as failing tests first, then implements until green, then refactors while
//! keeping tests passing.

pub mod config;
pub mod relay;
pub mod action;
pub mod input;
pub mod debug;

#[cfg(test)]
mod tests {
    #[test]
    fn test_harness_loads() {
        // Minimal test to verify the test harness runs successfully.
        // Phase 1+ will add substantive tests here and in dedicated test modules.
        assert_eq!(2 + 2, 4);
    }
}
