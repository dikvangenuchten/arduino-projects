//! Hardware-facing API boundary and its error type.
//!
//! `Io22d08Api` is the only surface the AVR orchestrator
//! (`wrapper::Io22d08Controller`) uses to talk to real hardware. The trait
//! has no dependency on any specific HAL, which keeps it host-testable
//! via a fake implementation.

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BoardError {
    InvalidRelayIndex,
    InvalidInputIndex,
    InvalidButtonIndex,
    InvalidDigitIndex,
    Pin,
}

pub trait Io22d08Api {
    fn set_number(&mut self, value: u16);
    fn show_digit(&mut self, position: usize, value: u8) -> Result<(), BoardError>;
    fn relay_on(&mut self, relay: usize) -> Result<(), BoardError>;
    fn relay_off(&mut self, relay: usize) -> Result<(), BoardError>;
    fn relay_toggle(&mut self, relay: usize) -> Result<(), BoardError>;
    fn read_button(&mut self, button: usize) -> Result<bool, BoardError>;
    fn read_input(&mut self, input: usize) -> Result<bool, BoardError>;
    fn tick(&mut self) -> Result<(), BoardError>;
}
