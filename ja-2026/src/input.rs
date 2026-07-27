//! Input mapping, debouncing, and edge detection.
//!
//! Phase 3: Debounce all external inputs and built-in buttons (after
//! active-low normalization by the board), detect press edges, resolve
//! Shift, and map to semantic `Action`s via one static table.

use crate::action::Action;
use crate::config::{ShiftMode, DEBOUNCE_SAMPLES, INPUT_COUNT, SHIFT_MODE};

/// Debounces `N` independent logical input lines.
///
/// Accepts a candidate value once it has been sampled for
/// `DEBOUNCE_SAMPLES` consecutive ticks; any mismatched sample resets that
/// line's consecutive count (bounce reset). Generic over `N` so the same
/// primitive debounces both the external inputs and the built-in buttons.
pub struct Debouncer<const N: usize> {
    stable: [bool; N],
    candidate: [bool; N],
    count: [u32; N],
}

impl<const N: usize> Debouncer<N> {
    /// Construct a debouncer with all lines idle (`false`).
    pub fn new() -> Self {
        Debouncer {
            stable: [false; N],
            candidate: [false; N],
            count: [0; N],
        }
    }

    /// Feed one 1 ms raw sample for all `N` lines; return the updated
    /// stable snapshot.
    pub fn sample(&mut self, raw: [bool; N]) -> [bool; N] {
        for (((stable, candidate), count), raw) in self
            .stable
            .iter_mut()
            .zip(self.candidate.iter_mut())
            .zip(self.count.iter_mut())
            .zip(raw.iter())
        {
            if *raw == *candidate {
                if *count < DEBOUNCE_SAMPLES {
                    *count += 1;
                }
                if *count >= DEBOUNCE_SAMPLES {
                    *stable = *candidate;
                }
            } else {
                *candidate = *raw;
                *count = 1;
            }
        }
        self.stable
    }
}

impl<const N: usize> Default for Debouncer<N> {
    fn default() -> Self {
        Self::new()
    }
}

/// Index of the Shift line among the external inputs.
const IDX_SHIFT: usize = 4;
/// Index of the Global mode line among the external inputs.
const IDX_GLOBAL: usize = 5;
/// Index of the Speed line among the external inputs.
const IDX_SPEED: usize = 6;
/// Index of the Reserved (no-op) line among the external inputs.
const IDX_RESERVED: usize = 7;

/// Resolve whether the mapper should treat this tick as shifted, given the
/// configured `ShiftMode` and the Shift line's current stable level.
pub fn resolve_shifted(mode: ShiftMode, shift_line_stable: bool) -> bool {
    match mode {
        ShiftMode::Momentary => shift_line_stable,
    }
}

/// Map press edges (rising transitions of the debounced stable snapshot) to
/// semantic actions using the static default external mapping:
/// I0-I3 relay keys, I4 Shift, I5 Global, I6 Speed, I7 Reserved.
pub fn map_edges(edges: [bool; INPUT_COUNT], shifted: bool) -> [Option<Action>; INPUT_COUNT] {
    let mut actions = [None; INPUT_COUNT];

    for i in 0..IDX_SHIFT {
        if edges[i] {
            actions[i] = Some(if shifted {
                Action::ToggleBlink(i + 4)
            } else {
                Action::TogglePower(i)
            });
        }
    }

    if edges[IDX_GLOBAL] {
        actions[IDX_GLOBAL] = Some(if shifted {
            Action::GlobalBlink
        } else {
            Action::GlobalPower
        });
    }

    if edges[IDX_SPEED] {
        actions[IDX_SPEED] = Some(if shifted {
            Action::SpeedAllDown
        } else {
            Action::SpeedAllUp
        });
    }

    if edges[IDX_RESERVED] {
        actions[IDX_RESERVED] = Some(Action::Reserved);
    }

    actions
}

/// Combines debouncing, press-edge detection, and Shift-aware mapping for
/// the eight external inputs into one per-tick orchestrator.
pub struct ExternalInputs {
    debouncer: Debouncer<INPUT_COUNT>,
    prev_stable: [bool; INPUT_COUNT],
}

impl ExternalInputs {
    /// Construct with all lines idle.
    pub fn new() -> Self {
        ExternalInputs {
            debouncer: Debouncer::new(),
            prev_stable: [false; INPUT_COUNT],
        }
    }

    /// Feed one 1 ms raw sample for all eight external inputs (already
    /// active-low normalized by the board) and return this tick's emitted
    /// actions, keyed by input index.
    pub fn tick_1ms(&mut self, raw: [bool; INPUT_COUNT]) -> [Option<Action>; INPUT_COUNT] {
        // Commit one stable snapshot first, atomically, for all lines.
        let stable = self.debouncer.sample(raw);

        let mut edges = [false; INPUT_COUNT];
        for i in 0..INPUT_COUNT {
            edges[i] = stable[i] && !self.prev_stable[i];
        }

        // Resolve Shift from the just-committed snapshot so a Shift press
        // accepted on the same tick as an action press is already shifted.
        let shifted = resolve_shifted(SHIFT_MODE, stable[IDX_SHIFT]);
        let actions = map_edges(edges, shifted);

        self.prev_stable = stable;
        actions
    }
}

impl Default for ExternalInputs {
    fn default() -> Self {
        Self::new()
    }
}
