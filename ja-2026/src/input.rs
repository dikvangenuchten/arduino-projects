//! Input mapping, debouncing, and edge detection.
//!
//! Phase 3: Debounce all external inputs and built-in buttons (after
//! active-low normalization by the board), detect press edges, resolve
//! Shift, and map to semantic `Action`s via one static table.

use crate::config::{ShiftMode, DEBOUNCE_SAMPLES, INPUT_COUNT};
use crate::action::Action;

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
        for i in 0..N {
            if raw[i] == self.candidate[i] {
                if self.count[i] < DEBOUNCE_SAMPLES {
                    self.count[i] += 1;
                }
                if self.count[i] >= DEBOUNCE_SAMPLES {
                    self.stable[i] = self.candidate[i];
                }
            } else {
                self.candidate[i] = raw[i];
                self.count[i] = 1;
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
