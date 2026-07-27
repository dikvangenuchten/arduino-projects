#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

mod board;

use board::create_from_dp;
use ja_2026::wrapper::Io22d08Controller;
use panic_halt as _;

#[arduino_hal::entry]
fn main() -> ! {
    // Create an abstracted interface to the board peripherals and the display driver.
    let dp = arduino_hal::Peripherals::take().unwrap();
    let (refresher, board) = create_from_dp(dp);

    // Screen needs constant refreshing.
    refresher.enable_interrupts();

    let mut ctrl = Io22d08Controller::new(board);

    loop {
        let pending = refresher.consume_ticks();
        for _ in 0..pending {
            let _ = ctrl.sync_tick();
        }
    }
}


