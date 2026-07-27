//! Input mapping, debouncing, and edge detection.
//!
//! Phase 3: Debounce all external inputs and built-in buttons (after
//! active-low normalization by the board), detect press edges, resolve
//! Shift, and map to semantic `Action`s via one static table.

use crate::config::DEBOUNCE_SAMPLES;

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
