use crate::relay::{RelayBank, RelayMode};

fn tick_n(bank: &mut RelayBank, ticks: u16) {
    // Advance synthetic time by `ticks` milliseconds.
    for _ in 0..ticks {
        bank.tick_1ms();
    }
}

#[test]
fn entering_blink_starts_on_with_full_half_cycle() {
    // Step 1: Start from default state (all relays Off).
    let mut bank = RelayBank::new();

    // Step 2: Enter Blink mode for relay 0.
    bank.set_mode(0, RelayMode::Blink);

    // Step 3: Entering Blink must immediately command output On.
    assert!(bank.relay_output(0));

    // Step 4: Advance to one millisecond before first half-cycle boundary
    // (default stored speed is 500 ms, so boundary is at tick 500).
    tick_n(&mut bank, 499);

    // Step 5: Output must still be On before the boundary is crossed.
    assert!(bank.relay_output(0));

    // Step 6: Advance exactly one more tick to reach boundary.
    bank.tick_1ms();

    // Step 7: Output must toggle to Off at the boundary.
    assert!(!bank.relay_output(0));
}

#[test]
fn leaving_blink_immediately_sets_commanded_steady_output() {
    // Step 1: Start from default relay bank.
    let mut bank = RelayBank::new();

    // Step 2: Enter Blink for relay 0 and advance part-way into half-cycle.
    bank.set_mode(0, RelayMode::Blink);
    tick_n(&mut bank, 123);

    // Step 3: While still inside first half-cycle, output should be On.
    assert!(bank.relay_output(0));

    // Step 4: Exit Blink into Off.
    bank.set_mode(0, RelayMode::Off);

    // Step 5: Leaving Blink must immediately command steady Off.
    assert!(!bank.relay_output(0));

    // Step 6: Switch to steady On.
    bank.set_mode(0, RelayMode::On);

    // Step 7: Output must immediately reflect steady On.
    assert!(bank.relay_output(0));
}

#[test]
fn reentering_blink_from_non_blink_resets_phase_origin() {
    // Step 1: Enter Blink and consume part of first half-cycle.
    let mut bank = RelayBank::new();
    bank.set_mode(0, RelayMode::Blink);
    tick_n(&mut bank, 200);

    // Step 2: Leave Blink, then re-enter Blink.
    bank.set_mode(0, RelayMode::On);
    bank.set_mode(0, RelayMode::Blink);

    // Step 3: Re-entering Blink must restart phase at commanded On.
    assert!(bank.relay_output(0));

    // Step 4: Advance to one tick before new half-cycle boundary.
    tick_n(&mut bank, 499);

    // Step 5: Output should still be On before boundary.
    assert!(bank.relay_output(0));

    // Step 6: Cross the boundary by one tick.
    bank.tick_1ms();

    // Step 7: Output should toggle Off at the new boundary.
    assert!(!bank.relay_output(0));
}

#[test]
fn reapplying_blink_while_already_blinking_keeps_current_phase() {
    // Step 1: Enter Blink and consume part of first half-cycle.
    let mut bank = RelayBank::new();
    bank.set_mode(0, RelayMode::Blink);
    tick_n(&mut bank, 200);

    // Step 2: Re-apply Blink while already blinking.
    bank.set_mode(0, RelayMode::Blink);

    // Step 3: Advance to one tick before original boundary (not reset).
    tick_n(&mut bank, 299);

    // Step 4: Output should still be On if phase was preserved.
    assert!(bank.relay_output(0));

    // Step 5: Cross original boundary.
    bank.tick_1ms();

    // Step 6: Output should toggle Off exactly at preserved boundary.
    assert!(!bank.relay_output(0));
}

#[test]
fn relays_keep_independent_phase_offsets() {
    // Step 1: Start relay 0 blinking.
    let mut bank = RelayBank::new();
    bank.set_mode(0, RelayMode::Blink);

    // Step 2: Create an offset by waiting 200 ms before starting relay 1.
    tick_n(&mut bank, 200);
    bank.set_mode(1, RelayMode::Blink);

    // Step 3: Advance to one tick before relay 0 boundary.
    tick_n(&mut bank, 299);

    // Step 4: Both relays should still be On at this point.
    assert!(bank.relay_output(0));
    assert!(bank.relay_output(1));

    // Step 5: Cross relay 0 boundary only.
    bank.tick_1ms();

    // Step 6: Relay 0 toggles Off, relay 1 remains On due to offset.
    assert!(!bank.relay_output(0));
    assert!(bank.relay_output(1));

    // Step 7: Advance 200 ms to reach relay 1 boundary.
    tick_n(&mut bank, 200);

    // Step 8: Relay 1 now toggles Off; relay 0 stays Off.
    assert!(!bank.relay_output(0));
    assert!(!bank.relay_output(1));
}
