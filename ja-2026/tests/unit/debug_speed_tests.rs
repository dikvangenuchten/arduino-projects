use crate::config::RELAY_COUNT;
use crate::debug::modal_speed_ms;
use crate::relay::{RelayBank, RelayMode};

fn bank_with_speeds(speeds: [u16; RELAY_COUNT]) -> RelayBank {
    let mut bank = RelayBank::new();
    for (i, &speed) in speeds.iter().enumerate() {
        bank.set_mode(i, RelayMode::Blink);
        bank.set_speed(i, speed);
    }
    bank
}

#[test]
fn uniform_periods_returns_that_period() {
    let bank = bank_with_speeds([500, 500, 500, 500, 500, 500, 500, 500]);
    assert_eq!(modal_speed_ms(&bank), 500);
}

#[test]
fn unique_mode_returns_most_frequent_period() {
    let bank = bank_with_speeds([1200, 1200, 1200, 800, 800, 500, 300, 150]);
    assert_eq!(modal_speed_ms(&bank), 1200);
}

#[test]
fn two_way_tie_returns_the_largest_tied_value() {
    // 800 and 300 both occur 3 times; 800 is the larger value.
    let bank = bank_with_speeds([800, 800, 800, 300, 300, 300, 1200, 150]);
    assert_eq!(modal_speed_ms(&bank), 800);
}

#[test]
fn distinct_frequencies_select_the_most_frequent_value_regardless_of_magnitude() {
    // 300 occurs 3 times (the most), even though it is neither the
    // smallest nor the largest configured level.
    let bank = bank_with_speeds([300, 300, 300, 1200, 1200, 800, 500, 150]);
    assert_eq!(modal_speed_ms(&bank), 300);
}
