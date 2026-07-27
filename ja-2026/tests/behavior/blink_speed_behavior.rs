use crate::relay::{RelayBank, RelayMode};

fn tick_n(bank: &mut RelayBank, ticks: u16) {
    // Advance synthetic time by `ticks` milliseconds.
    for _ in 0..ticks {
        bank.tick_1ms();
    }
}

#[test]
fn speed_change_applies_on_next_phase_transition() {
    // Step 1: Start with a new relay bank.
    let mut bank = RelayBank::new();

    // Step 2: Set relay 0 speed to 500 ms and enter Blink.
    bank.set_speed(0, 500);
    bank.set_mode(0, RelayMode::Blink);

    // Step 3: Consume part of current 500 ms half-cycle.
    tick_n(&mut bank, 200);

    // Step 4: Change stored speed to 300 ms while still in active phase.
    bank.set_speed(0, 300);

    // Step 5: Advance to one tick before old (500 ms) boundary.
    tick_n(&mut bank, 299);

    // Step 6: Output must remain On until current phase completes.
    assert!(bank.relay_output(0));

    // Step 7: Cross old boundary; relay toggles Off.
    bank.tick_1ms();
    assert!(!bank.relay_output(0));

    // Step 8: New speed (300 ms) should be active for this Off half-cycle.
    // Advance to one tick before new boundary.
    tick_n(&mut bank, 299);

    // Step 9: Output should still be Off before new boundary.
    assert!(!bank.relay_output(0));

    // Step 10: Cross new boundary; relay toggles back On.
    bank.tick_1ms();

    // Step 11: On confirms the new 300 ms duration was adopted at transition.
    assert!(bank.relay_output(0));
}
