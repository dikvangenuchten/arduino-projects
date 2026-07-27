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
pub mod board_api;
pub mod wrapper;

#[cfg(test)]
#[path = "../tests/mod.rs"]
mod tests;
