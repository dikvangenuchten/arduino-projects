#![allow(dead_code)]

use arduino_hal::Peripherals;
use avr_device::interrupt::{self, Mutex};
use core::cell::Cell;
use embedded_hal::digital::{InputPin, OutputPin};

/// Number of relay outputs on the IO22D08 board.
pub const RELAY_COUNT: usize = 8;
/// Number of opto-coupled digital inputs on the IO22D08 board.
pub const INPUT_COUNT: usize = 8;
/// Number of physical push-buttons on the IO22D08 board.
pub const BUTTON_COUNT: usize = 4;
/// Number of 7-segment display digits on the IO22D08 board.
pub const DIGIT_COUNT: usize = 4;

static DISPLAY_TICK_PENDING: Mutex<Cell<u8>> = Mutex::new(Cell::new(0));

/// Drives the display multiplex via a 1 kHz Timer1 interrupt.
///
/// The interrupt increments a pending-tick counter; the main loop calls
/// [`DisplayRefresher::consume_ticks`] to drain it and tick the engine accordingly.
pub struct DisplayRefresher;

impl DisplayRefresher {
    pub fn new(tc1: arduino_hal::pac::TC1) -> Self {
        tc1.tccr1a().write(|w| unsafe { w.bits(0) });
        // CTC mode (WGM12=1) and prescaler 64 (CS11|CS10=1).
        tc1.tccr1b()
            .write(|w| unsafe { w.bits((1 << 3) | (1 << 1) | (1 << 0)) });
        tc1.tcnt1().write(|w| unsafe { w.bits(0) });
        // 16MHz / 64 / (249 + 1) = 1000Hz -> 1ms interrupt period.
        tc1.ocr1a().write(|w| unsafe { w.bits(249) });
        // Enable Timer1 Compare Match A interrupt (OCIE1A).
        tc1.timsk1().write(|w| unsafe { w.bits(1 << 1) });

        Self
    }

    /// Globally enables AVR interrupts so the Timer1 ISR can fire.
    pub fn enable_interrupts(&self) {
        unsafe { avr_device::interrupt::enable() };
    }

    /// Atomically reads and resets the pending-tick counter.
    ///
    /// Returns the number of ticks that fired since the last call. The main loop
    /// should call [`Engine::tick`] this many times to stay in sync.
    pub fn consume_ticks(&self) -> u8 {
        interrupt::free(|cs| {
            let pending = DISPLAY_TICK_PENDING.borrow(cs).get();
            DISPLAY_TICK_PENDING.borrow(cs).set(0);
            pending
        })
    }
}

#[avr_device::interrupt(atmega328p)]
fn TIMER1_COMPA() {
    interrupt::free(|cs| {
        let pending = DISPLAY_TICK_PENDING.borrow(cs).get();
        DISPLAY_TICK_PENDING.borrow(cs).set(pending.wrapping_add(1));
    });
}

/// Full hardware API for the Eletechsup IO22D08 expansion board.
///
/// Implemented by [`Io22d08Board`]. [`BoardIo`][crate::engine::BoardIo] is a
/// blanket-implemented subset used by the engine to avoid taking on hardware
/// dependencies in scene logic tests.
pub trait Io22d08Api {
    /// Write a 4-digit decimal number to the display buffer (applied on the next `tick`).
    fn set_number(&mut self, value: u16);
    /// Set a single display digit by position (`0` = leftmost).
    fn show_digit(&mut self, position: usize, value: u8) -> Result<(), BoardError>;
    /// Turn a relay on by index (`0`-based).
    fn relay_on(&mut self, relay: usize) -> Result<(), BoardError>;
    /// Turn a relay off by index.
    fn relay_off(&mut self, relay: usize) -> Result<(), BoardError>;
    /// Toggle a relay by index.
    fn relay_toggle(&mut self, relay: usize) -> Result<(), BoardError>;
    /// Read one of the 4 physical buttons (low-active, returns logical `true` when pressed).
    fn read_button(&mut self, button: usize) -> Result<bool, BoardError>;
    /// Read one of the 8 opto-coupled inputs (low-active, returns logical `true` when active).
    fn read_input(&mut self, input: usize) -> Result<bool, BoardError>;
    /// Advance the display multiplex by one digit. Must be called at ~1 kHz.
    fn tick(&mut self) -> Result<(), BoardError>;
}

pub fn create_from_dp(dp: Peripherals) -> (DisplayRefresher, impl Io22d08Api) {
    let refresher = DisplayRefresher::new(dp.TC1);
    let pins = arduino_hal::pins!(dp);

    let data = pins.d13.into_output();
    let oe_595 = pins.a1.into_output();
    let latch = pins.a2.into_output();
    let clock = pins.a3.into_output();

    let buttons = (
        pins.d7.into_pull_up_input(),
        pins.d8.into_pull_up_input(),
        pins.d9.into_pull_up_input(),
        pins.d10.into_pull_up_input(),
    );

    let inputs = (
        pins.d2.into_pull_up_input(),
        pins.d3.into_pull_up_input(),
        pins.d4.into_pull_up_input(),
        pins.d5.into_pull_up_input(),
        pins.d6.into_pull_up_input(),
        pins.a0.into_pull_up_input(),
        pins.d12.into_pull_up_input(),
        pins.d11.into_pull_up_input(),
    );

    let board = Io22d08Board::new(data, oe_595, latch, clock, buttons, inputs);
    (refresher, board)
}

/// Errors returned by board hardware operations.
#[derive(Copy, Clone, Debug)]
pub enum BoardError {
    /// Relay index is out of range (`>= RELAY_COUNT`).
    InvalidRelayIndex,
    /// Input index is out of range (`>= INPUT_COUNT`).
    InvalidInputIndex,
    /// Button index is out of range (`>= BUTTON_COUNT`).
    InvalidButtonIndex,
    /// Digit position is out of range (`>= DIGIT_COUNT`).
    InvalidDigitIndex,
    /// A GPIO read or write operation failed.
    Pin,
}

struct DigitalInput<P>
where
    P: InputPin,
{
    pin: P,
}

impl<P> DigitalInput<P>
where
    P: InputPin,
{
    fn new(pin: P) -> Self {
        Self { pin }
    }

    fn read_low_active(&mut self) -> Result<bool, BoardError> {
        let high = self.pin.is_high().map_err(|_| BoardError::Pin)?;
        Ok(!high)
    }
}

pub struct Io22d08Board<DATA, LATCH, CLOCK, B0, B1, B2, B3, I0, I1, I2, I3, I4, I5, I6, I7>
where
    DATA: OutputPin,
    LATCH: OutputPin,
    CLOCK: OutputPin,
    B0: InputPin,
    B1: InputPin,
    B2: InputPin,
    B3: InputPin,
    I0: InputPin,
    I1: InputPin,
    I2: InputPin,
    I3: InputPin,
    I4: InputPin,
    I5: InputPin,
    I6: InputPin,
    I7: InputPin,
{
    data: DATA,
    latch: LATCH,
    clock: CLOCK,
    buttons: (
        DigitalInput<B0>,
        DigitalInput<B1>,
        DigitalInput<B2>,
        DigitalInput<B3>,
    ),
    inputs: (
        DigitalInput<I0>,
        DigitalInput<I1>,
        DigitalInput<I2>,
        DigitalInput<I3>,
        DigitalInput<I4>,
        DigitalInput<I5>,
        DigitalInput<I6>,
        DigitalInput<I7>,
    ),
    dat_buf: [u8; DIGIT_COUNT],
    relay_port: u8,
    com_num: usize,
}

impl<DATA, LATCH, CLOCK, B0, B1, B2, B3, I0, I1, I2, I3, I4, I5, I6, I7>
    Io22d08Board<DATA, LATCH, CLOCK, B0, B1, B2, B3, I0, I1, I2, I3, I4, I5, I6, I7>
where
    DATA: OutputPin,
    LATCH: OutputPin,
    CLOCK: OutputPin,
    B0: InputPin,
    B1: InputPin,
    B2: InputPin,
    B3: InputPin,
    I0: InputPin,
    I1: InputPin,
    I2: InputPin,
    I3: InputPin,
    I4: InputPin,
    I5: InputPin,
    I6: InputPin,
    I7: InputPin,
{
    /// Creates a new `Io22d08Board` using the board's fixed hardware mapping.
    ///
    /// Signature:
    /// `pub fn new<OE>(data: DATA, oe_595: OE, latch: LATCH, clock: CLOCK, button_pins: (B0, B1, B2, B3), input_pins: (I0, I1, I2, I3, I4, I5, I6, I7)) -> Self where OE: OutputPin`
    ///
    /// Goal:
    /// Initialize the 74HC595 control pins, enable outputs, and store button/input pins
    /// so the board API can control relays, read inputs/buttons, and refresh the display.
    ///
    /// Parameters:
    /// - `data`: serial data line for the daisy-chained shift registers.
    /// - `oe_595`: output-enable pin for the first 74HC595 (active low).
    /// - `latch`: latch/storage clock pin.
    /// - `clock`: shift clock pin.
    /// - `button_pins`: 4 low-active button input pins.
    /// - `input_pins`: 8 low-active opto input pins.
    pub fn new<OE>(
        mut data: DATA,
        mut oe_595: OE,
        mut latch: LATCH,
        mut clock: CLOCK,
        button_pins: (B0, B1, B2, B3),
        input_pins: (I0, I1, I2, I3, I4, I5, I6, I7),
    ) -> Self
    where
        OE: OutputPin,
    {
        let _ = data.set_low();
        let _ = latch.set_high();
        let _ = clock.set_low();
        let _ = oe_595.set_low();

        Self {
            data,
            latch,
            clock,
            buttons: (
                DigitalInput::new(button_pins.0),
                DigitalInput::new(button_pins.1),
                DigitalInput::new(button_pins.2),
                DigitalInput::new(button_pins.3),
            ),
            inputs: (
                DigitalInput::new(input_pins.0),
                DigitalInput::new(input_pins.1),
                DigitalInput::new(input_pins.2),
                DigitalInput::new(input_pins.3),
                DigitalInput::new(input_pins.4),
                DigitalInput::new(input_pins.5),
                DigitalInput::new(input_pins.6),
                DigitalInput::new(input_pins.7),
            ),
            dat_buf: [0, 0, 0, 0],
            relay_port: 0,
            com_num: 0,
        }
    }

    /// Writes a full 4-digit decimal value into the display buffer.
    ///
    /// Signature:
    /// `pub fn set_number(&mut self, value: u16)`
    ///
    /// Goal:
    /// Split `value` into thousands/hundreds/tens/ones and store it for multiplex display.
    /// The update becomes visible when `tick` is called repeatedly.
    ///
    /// Parameters:
    /// - `value`: number to show, expected in range `0..=9999` for meaningful output.
    pub fn set_number(&mut self, mut value: u16) {
        self.dat_buf[0] = (value / 1000) as u8;
        value %= 1000;
        self.dat_buf[1] = (value / 100) as u8;
        value %= 100;
        self.dat_buf[2] = (value / 10) as u8;
        self.dat_buf[3] = (value % 10) as u8;
    }

    /// Sets a single digit in the display buffer.
    ///
    /// Signature:
    /// `pub fn show_digit(&mut self, position: usize, value: u8) -> Result<(), BoardError>`
    ///
    /// Goal:
    /// Update one display position without touching other digits.
    ///
    /// Parameters:
    /// - `position`: digit index `0..4` (left-to-right according to hardware mapping).
    /// - `value`: decimal digit value; values above `9` are reduced with `% 10`.
    ///
    /// Returns:
    /// - `Ok(())` when the buffer is updated.
    /// - `Err(BoardError::InvalidDigitIndex)` when `position` is out of range.
    pub fn show_digit(&mut self, position: usize, value: u8) -> Result<(), BoardError> {
        if position >= DIGIT_COUNT {
            return Err(BoardError::InvalidDigitIndex);
        }

        self.dat_buf[position] = value % 10;
        Ok(())
    }

    /// Turns on one relay output.
    ///
    /// Signature:
    /// `pub fn relay_on(&mut self, relay: usize) -> Result<(), BoardError>`
    ///
    /// Goal:
    /// Set one relay bit in the relay shadow register.
    ///
    /// The physical relay output is applied on the next `tick`.
    ///
    /// Parameters:
    /// - `relay`: relay index `0..8`.
    ///
    /// Returns:
    /// - `Ok(())` on success.
    /// - `Err(BoardError::InvalidRelayIndex)` when `relay` is out of range.
    /// - `Err(BoardError::Pin)` on GPIO write failures.
    pub fn relay_on(&mut self, relay: usize) -> Result<(), BoardError> {
        if relay >= RELAY_COUNT {
            return Err(BoardError::InvalidRelayIndex);
        }

        self.relay_port |= 1 << relay;
        Ok(())
    }

    /// Turns off one relay output.
    ///
    /// Signature:
    /// `pub fn relay_off(&mut self, relay: usize) -> Result<(), BoardError>`
    ///
    /// Goal:
    /// Clear one relay bit in the relay shadow register.
    ///
    /// The physical relay output is applied on the next `tick`.
    ///
    /// Parameters:
    /// - `relay`: relay index `0..8`.
    ///
    /// Returns:
    /// - `Ok(())` on success.
    /// - `Err(BoardError::InvalidRelayIndex)` when `relay` is out of range.
    /// - `Err(BoardError::Pin)` on GPIO write failures.
    pub fn relay_off(&mut self, relay: usize) -> Result<(), BoardError> {
        if relay >= RELAY_COUNT {
            return Err(BoardError::InvalidRelayIndex);
        }

        self.relay_port &= !(1 << relay);
        Ok(())
    }

    /// Toggles one relay output.
    ///
    /// Signature:
    /// `pub fn relay_toggle(&mut self, relay: usize) -> Result<(), BoardError>`
    ///
    /// Goal:
    /// Flip one relay bit in the relay shadow register.
    ///
    /// The physical relay output is applied on the next `tick`.
    ///
    /// Parameters:
    /// - `relay`: relay index `0..8`.
    ///
    /// Returns:
    /// - `Ok(())` on success.
    /// - `Err(BoardError::InvalidRelayIndex)` when `relay` is out of range.
    /// - `Err(BoardError::Pin)` on GPIO write failures.
    pub fn relay_toggle(&mut self, relay: usize) -> Result<(), BoardError> {
        if relay >= RELAY_COUNT {
            return Err(BoardError::InvalidRelayIndex);
        }

        self.relay_port ^= 1 << relay;
        Ok(())
    }

    /// Reads one physical button (low-active).
    ///
    /// Signature:
    /// `pub fn read_button(&mut self, button: usize) -> Result<bool, BoardError>`
    ///
    /// Goal:
    /// Return whether the selected button is currently pressed.
    ///
    /// Parameters:
    /// - `button`: button index `0..4`.
    ///
    /// Returns:
    /// - `Ok(true)` when pressed.
    /// - `Ok(false)` when released.
    /// - `Err(BoardError::InvalidButtonIndex)` when `button` is out of range.
    /// - `Err(BoardError::Pin)` on GPIO read failures.
    pub fn read_button(&mut self, button: usize) -> Result<bool, BoardError> {
        match button {
            0 => self.buttons.0.read_low_active(),
            1 => self.buttons.1.read_low_active(),
            2 => self.buttons.2.read_low_active(),
            3 => self.buttons.3.read_low_active(),
            _ => Err(BoardError::InvalidButtonIndex),
        }
    }

    /// Reads one opto-isolated input (low-active).
    ///
    /// Signature:
    /// `pub fn read_input(&mut self, input: usize) -> Result<bool, BoardError>`
    ///
    /// Goal:
    /// Return whether the selected external input channel is active.
    ///
    /// Parameters:
    /// - `input`: input index `0..8`.
    ///
    /// Returns:
    /// - `Ok(true)` when active.
    /// - `Ok(false)` when inactive.
    /// - `Err(BoardError::InvalidInputIndex)` when `input` is out of range.
    /// - `Err(BoardError::Pin)` on GPIO read failures.
    pub fn read_input(&mut self, input: usize) -> Result<bool, BoardError> {
        match input {
            0 => self.inputs.0.read_low_active(),
            1 => self.inputs.1.read_low_active(),
            2 => self.inputs.2.read_low_active(),
            3 => self.inputs.3.read_low_active(),
            4 => self.inputs.4.read_low_active(),
            5 => self.inputs.5.read_low_active(),
            6 => self.inputs.6.read_low_active(),
            7 => self.inputs.7.read_low_active(),
            _ => Err(BoardError::InvalidInputIndex),
        }
    }

    /// Performs one multiplex refresh/update cycle.
    ///
    /// Signature:
    /// `pub fn tick(&mut self) -> Result<(), BoardError>`
    ///
    /// Goal:
    /// Advance to the next display digit, combine display + relay state, and shift out
    /// the three payload bytes (`display_h`, `display_l`, `relay_dat`) to the daisy-chained
    /// 74HC595 registers.
    ///
    /// Returns:
    /// - `Ok(())` on successful hardware update.
    /// - `Err(BoardError::Pin)` on GPIO write failures.
    pub fn tick(&mut self) -> Result<(), BoardError> {
        const TUBE_NUM: [u8; 4] = [0xfe, 0xfd, 0xfb, 0xf7];
        const TUBE_SEG: [u8; 10] = [0xc0, 0xf9, 0xa4, 0xb0, 0x99, 0x92, 0x82, 0xf8, 0x80, 0x90];

        self.com_num = (self.com_num + 1) % 4;
        let dat = self.dat_buf[self.com_num] as usize;
        let tube_dat = TUBE_SEG[dat];
        let bit_num = !TUBE_NUM[self.com_num];

        let mut display_l = (tube_dat & 0x10) >> 3;
        display_l |= (bit_num & 0x01) << 2;
        display_l |= tube_dat & 0x08;
        display_l |= (tube_dat & 0x01) << 4;
        display_l |= (tube_dat & 0x80) >> 2;
        display_l |= (tube_dat & 0x20) << 1;
        display_l |= (tube_dat & 0x04) << 5;

        let mut display_h = bit_num & 0x02;
        display_h |= bit_num & 0x04;
        display_h |= (tube_dat & 0x40) >> 3;
        display_h |= (tube_dat & 0x02) << 3;
        display_h |= (bit_num & 0x08) << 2;

        let relay_dat = ((self.relay_port & 0x7f) << 1) | ((self.relay_port & 0x80) >> 7);

        self.latch.set_low().map_err(|_| BoardError::Pin)?;
        self.shift_out(display_h)?;
        self.shift_out(display_l)?;
        self.shift_out(relay_dat)?;
        self.latch.set_high().map_err(|_| BoardError::Pin)?;

        Ok(())
    }

    fn shift_out(&mut self, mut value: u8) -> Result<(), BoardError> {
        for _ in 0..8 {
            if (value & 0x80) != 0 {
                self.data.set_high().map_err(|_| BoardError::Pin)?;
            } else {
                self.data.set_low().map_err(|_| BoardError::Pin)?;
            }

            self.clock.set_high().map_err(|_| BoardError::Pin)?;
            self.clock.set_low().map_err(|_| BoardError::Pin)?;
            value <<= 1;
        }

        Ok(())
    }
}

impl<DATA, LATCH, CLOCK, B0, B1, B2, B3, I0, I1, I2, I3, I4, I5, I6, I7> Io22d08Api
    for Io22d08Board<DATA, LATCH, CLOCK, B0, B1, B2, B3, I0, I1, I2, I3, I4, I5, I6, I7>
where
    DATA: OutputPin,
    LATCH: OutputPin,
    CLOCK: OutputPin,
    B0: InputPin,
    B1: InputPin,
    B2: InputPin,
    B3: InputPin,
    I0: InputPin,
    I1: InputPin,
    I2: InputPin,
    I3: InputPin,
    I4: InputPin,
    I5: InputPin,
    I6: InputPin,
    I7: InputPin,
{
    fn set_number(&mut self, value: u16) {
        Io22d08Board::set_number(self, value);
    }

    fn show_digit(&mut self, position: usize, value: u8) -> Result<(), BoardError> {
        Io22d08Board::show_digit(self, position, value)
    }

    fn relay_on(&mut self, relay: usize) -> Result<(), BoardError> {
        Io22d08Board::relay_on(self, relay)
    }

    fn relay_off(&mut self, relay: usize) -> Result<(), BoardError> {
        Io22d08Board::relay_off(self, relay)
    }

    fn relay_toggle(&mut self, relay: usize) -> Result<(), BoardError> {
        Io22d08Board::relay_toggle(self, relay)
    }

    fn read_button(&mut self, button: usize) -> Result<bool, BoardError> {
        Io22d08Board::read_button(self, button)
    }

    fn read_input(&mut self, input: usize) -> Result<bool, BoardError> {
        Io22d08Board::read_input(self, input)
    }

    fn tick(&mut self) -> Result<(), BoardError> {
        Io22d08Board::tick(self)
    }
}
