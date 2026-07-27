//! Debug view builder (read-only diagnostics).
//!
//! Phase 4: Immutable diagnostic state/view formatting, modal-speed
//! calculation, and page selection. No function here receives mutable
//! relay/controller state; rendering only ever reads shared snapshots.

use crate::config::{BUTTON_COUNT, INPUT_COUNT, SPEED_LEVELS_MS};
use crate::relay::{RelayBank, RelayMode};

/// Which diagnostic view is currently selected.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DiagnosticView {
    /// Modal stored per-relay speed.
    Speed,
    /// Raw external input states, paged.
    Inputs,
    /// Relay modes, paged.
    Relay,
}

/// Which half of an 8-channel set a paged view is currently showing.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Page {
    /// Channels 0-3.
    First,
    /// Channels 4-7.
    Second,
}

impl Page {
    fn toggled(self) -> Self {
        match self {
            Page::First => Page::Second,
            Page::Second => Page::First,
        }
    }
}

/// Navigation state for the diagnostic display: selected view and each
/// paged view's current page. Boots on the Speed view.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DiagnosticState {
    view: DiagnosticView,
    inputs_page: Page,
    relay_page: Page,
}

impl DiagnosticState {
    /// Construct with the Speed view selected and both pages at First.
    pub fn new() -> Self {
        DiagnosticState {
            view: DiagnosticView::Speed,
            inputs_page: Page::First,
            relay_page: Page::First,
        }
    }

    /// Currently selected view.
    pub fn view(&self) -> DiagnosticView {
        self.view
    }

    /// Current page of the Inputs view.
    pub fn inputs_page(&self) -> Page {
        self.inputs_page
    }

    /// Current page of the Relay view.
    pub fn relay_page(&self) -> Page {
        self.relay_page
    }

    /// Apply this tick's debounced button press edges: B0 selects Speed,
    /// B1 selects/pages Inputs, B2 selects/pages Relay, B3 is reserved
    /// (no-op). Edges are applied in ascending button index order.
    pub fn apply_button_edges(&mut self, edges: [bool; BUTTON_COUNT]) {
        if edges[0] {
            self.view = DiagnosticView::Speed;
        }
        if edges[1] {
            if self.view == DiagnosticView::Inputs {
                self.inputs_page = self.inputs_page.toggled();
            } else {
                self.view = DiagnosticView::Inputs;
                self.inputs_page = Page::First;
            }
        }
        if edges[2] {
            if self.view == DiagnosticView::Relay {
                self.relay_page = self.relay_page.toggled();
            } else {
                self.view = DiagnosticView::Relay;
                self.relay_page = Page::First;
            }
        }
        // edges[3] (B3) is reserved: intentionally ignored.
    }
}

impl Default for DiagnosticState {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute the modal (most frequent) stored per-relay period, in ms.
/// Ties choose the largest millisecond value.
pub fn modal_speed_ms(bank: &RelayBank) -> u16 {
    let mut best_level = SPEED_LEVELS_MS[0];
    let mut best_count = 0usize;

    for &level in SPEED_LEVELS_MS {
        let count = bank.relays().iter().filter(|r| r.speed_ms == level).count();
        if count > best_count || (count == best_count && level > best_level) {
            best_level = level;
            best_count = count;
        }
    }

    best_level
}

/// Render the currently selected diagnostic view into four display digits,
/// ready for one `show_digit` call per position.
pub fn render(
    state: &DiagnosticState,
    bank: &RelayBank,
    stable_inputs: [bool; INPUT_COUNT],
) -> [u8; 4] {
    match state.view {
        DiagnosticView::Speed => digits_from_number(modal_speed_ms(bank)),
        DiagnosticView::Inputs => {
            let offset = page_offset(state.inputs_page);
            let mut digits = [0u8; 4];
            for (i, digit) in digits.iter_mut().enumerate() {
                *digit = stable_inputs[offset + i] as u8;
            }
            digits
        }
        DiagnosticView::Relay => {
            let offset = page_offset(state.relay_page);
            let mut digits = [0u8; 4];
            for (i, digit) in digits.iter_mut().enumerate() {
                *digit = relay_mode_digit(bank.relay(offset + i).mode);
            }
            digits
        }
    }
}

/// Channel offset (into an 8-channel array) for the given page.
fn page_offset(page: Page) -> usize {
    match page {
        Page::First => 0,
        Page::Second => 4,
    }
}

/// Split a value into four decimal digits (thousands..ones), left-to-right.
fn digits_from_number(mut value: u16) -> [u8; 4] {
    let d0 = (value / 1000) as u8;
    value %= 1000;
    let d1 = (value / 100) as u8;
    value %= 100;
    let d2 = (value / 10) as u8;
    let d3 = (value % 10) as u8;
    [d0, d1, d2, d3]
}

/// Map a relay mode to its diagnostic digit: 0=Off, 1=Blink, 2=On.
fn relay_mode_digit(mode: RelayMode) -> u8 {
    match mode {
        RelayMode::Off => 0,
        RelayMode::Blink => 1,
        RelayMode::On => 2,
    }
}
