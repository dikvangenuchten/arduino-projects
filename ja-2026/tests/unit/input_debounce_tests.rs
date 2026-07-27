use crate::config::{BUTTON_COUNT, DEBOUNCE_SAMPLES};
use crate::input::Debouncer;

#[test]
fn line_stays_unstable_until_debounce_samples_consecutive() {
    let mut d: Debouncer<1> = Debouncer::new();

    // Idle default is false.
    assert_eq!(d.sample([false]), [false]);

    // Feed DEBOUNCE_SAMPLES - 1 consecutive true samples: not yet accepted.
    for _ in 0..(DEBOUNCE_SAMPLES - 1) {
        let stable = d.sample([true]);
        assert_eq!(stable, [false], "must not accept before threshold");
    }

    // One more consecutive true sample reaches the threshold.
    let stable = d.sample([true]);
    assert_eq!(stable, [true]);
}

#[test]
fn bounce_resets_the_consecutive_count() {
    let mut d: Debouncer<1> = Debouncer::new();

    for _ in 0..(DEBOUNCE_SAMPLES - 1) {
        d.sample([true]);
    }
    // One bounce back to false before reaching the threshold.
    let stable = d.sample([false]);
    assert_eq!(stable, [false]);

    // Needs a fresh run of DEBOUNCE_SAMPLES consecutive true samples now.
    for _ in 0..(DEBOUNCE_SAMPLES - 1) {
        let stable = d.sample([true]);
        assert_eq!(stable, [false], "bounce must restart the count");
    }
    let stable = d.sample([true]);
    assert_eq!(stable, [true]);
}

#[test]
fn release_after_stable_requires_full_debounce_again() {
    let mut d: Debouncer<1> = Debouncer::new();
    for _ in 0..DEBOUNCE_SAMPLES {
        d.sample([true]);
    }

    // Stable true now. Release, but must remain stable until threshold.
    for _ in 0..(DEBOUNCE_SAMPLES - 1) {
        let stable = d.sample([false]);
        assert_eq!(stable, [true], "must remain stable true until threshold reached");
    }
    let stable = d.sample([false]);
    assert_eq!(stable, [false]);
}

#[test]
fn lines_are_independent_and_generic_over_button_count() {
    let mut d: Debouncer<BUTTON_COUNT> = Debouncer::new();
    let mut raw = [false; BUTTON_COUNT];
    raw[0] = true;
    raw[2] = true;

    let mut stable = [false; BUTTON_COUNT];
    for _ in 0..DEBOUNCE_SAMPLES {
        stable = d.sample(raw);
    }
    assert_eq!(stable, [true, false, true, false]);
}

#[test]
fn simultaneous_transitions_are_all_accepted_on_the_same_tick() {
    let mut d: Debouncer<3> = Debouncer::new();
    let raw = [true, true, true];

    let mut stable = [false; 3];
    for _ in 0..DEBOUNCE_SAMPLES {
        stable = d.sample(raw);
    }
    assert_eq!(stable, [true, true, true]);
}
