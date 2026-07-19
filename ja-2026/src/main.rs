#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

mod board;
mod engine;
mod scenes;

use board::create_from_dp;
use engine::{Engine, EngineConfig, InputMode, TickCommand};
use panic_halt as _;
use scenes::{ButtonCycleSelector, SceneContext, SceneId, SceneManager};

/// Firmware entry point.
///
/// # Tick loop
///
/// Timer1 fires at 1 kHz and increments a pending-tick counter. The main loop
/// drains the counter and calls [`Engine::tick`] once per pending tick, then
/// feeds the resulting [`Snapshot`] to the [`SceneManager`] which produces the
/// next [`TickCommand`].
///
/// # Input layout (logical indices)
///
/// | Index | Source   | Mode               | Role                        |
/// |-------|----------|--------------------|-----------------------------|
/// | 0–3   | Buttons  | See config below   | B0: rotate dir, B3: scene   |
/// | 4–11  | Opto ins | Momentary          | Available for scene logic   |
#[arduino_hal::entry]
fn main() -> ! {
    // Create an abstracted interface to the board peripherals and the display driver.
    let dp = arduino_hal::Peripherals::take().unwrap();
    let (refresher, mut board) = create_from_dp(dp);

    // Screen needs constant refreshing.
    refresher.enable_interrupts();

    let mut config = EngineConfig::default();
    for i in 0..12 {
        config.input_modes[i] = InputMode::Momentary;
    }
    let mut engine = Engine::new(config);
    let mut command = TickCommand::default();
    let selector = ButtonCycleSelector::new(3);
    let mut scene_manager = SceneManager::new(selector, SceneId::Rotate, 100, 200);

    loop {
        let pending = refresher.consume_ticks();
        if pending == 0 {
            arduino_hal::delay_ms(1);
            continue;
        }

        for _ in 0..pending {
            if let Ok(snapshot) = engine.tick(&mut board, command) {
                let ctx = SceneContext {
                    current: snapshot,
                    previous: engine.prev,
                };
                command = scene_manager.update(&ctx);
            }
        }
    }
}
