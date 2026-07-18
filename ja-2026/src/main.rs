#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

mod board;
mod wrapper;

use board::create_from_dp;
use panic_halt as _;
use wrapper::{ControllerConfig, InputMode, Io22d08Controller, RelayInit};

#[arduino_hal::entry]
fn main() -> ! {
    // Create an abstracted interface to the board peripherals and the display driver.
    let dp = arduino_hal::Peripherals::take().unwrap();
    let (refresher, board) = create_from_dp(dp);

    // Screen needs constant refreshing.
    refresher.enable_interrupts();

    let mut config = ControllerConfig::default();
    config.button_modes[0] = InputMode::Toggle;
    config.relay_init[0] = RelayInit::Blink {
        on_ticks: 250,
        off_ticks: 250,
    };

    let mut ctrl = Io22d08Controller::new_with_config(board, config).unwrap();

    let mut value: u16 = 0;

    loop {
        ctrl.set_display_number(value);
        value = value.wrapping_add(1) % 10_000;

        for _ in 0..100 {
            let pending = refresher.consume_ticks();
            for _ in 0..pending {
                let _ = ctrl.sync_tick();

                // Example policy: button 0 toggles relay 0.
                if let Ok(state) = ctrl.button_state(0) {
                    let _ = ctrl.set_relay_state(0, state);
                }
            }
            arduino_hal::delay_ms(1);
        }
    }
}

