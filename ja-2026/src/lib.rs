#![cfg_attr(not(test), no_std)]

pub mod board {
    pub const RELAY_COUNT: usize = 8;
    pub const INPUT_COUNT: usize = 8;
    pub const BUTTON_COUNT: usize = 4;
    pub const DIGIT_COUNT: usize = 4;

    #[derive(Copy, Clone, Debug)]
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
}

#[path = "engine.rs"]
pub mod engine;

#[path = "scenes/mod.rs"]
pub mod scenes;

pub fn harness_anchor() -> u8 {
    1
}

#[cfg(test)]
mod tests {
    use super::harness_anchor;

    #[test]
    fn harness_sanity_passes() {
        assert_eq!(harness_anchor(), 1);
    }
}
