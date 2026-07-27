# Architecture

## Purpose

This project separates pure control-domain logic from AVR-specific hardware orchestration.
The domain lives in the library crate and is host-testable. Hardware access stays behind board and wrapper layers.

## Design Choices

### 1. Domain-Hardware Separation

- Pure domain modules:
  - `src/relay.rs`
  - `src/action.rs`
  - `src/input.rs`
  - `src/debug.rs`
- Hardware-facing modules:
  - `src/board.rs` (hardware API + pin wiring)
  - `src/wrapper.rs` (controller orchestration)
  - `src/main.rs` (startup and loop)

Why:
- Enables fast host-side testing without flashing hardware.
- Keeps side effects localized and easier to reason about.

### 2. Per-Relay Ownership Model

Each relay independently owns:
- mode (`Off`, `Blink`, `On`)
- stored speed in milliseconds
- blink phase/output state
- half-cycle countdown state

Why:
- Avoids hidden coupling and shared-timing bugs.
- Allows relays to run with independent offsets and transitions.

### 3. Semantic Action Layer

Input mapping resolves button/input events into semantic actions (`TogglePower`, `GlobalBlink`, `SpeedAllUp`, etc.).
Action handlers update relay state without hardware dependencies.

Why:
- Decouples UI/input policy from state transition logic.
- Makes transition rules easy to test with table-like scenarios.

### 4. Localized Blink Timing Policy

Blink half-cycle selection is localized behind a small policy boundary in `src/relay.rs`.
Current policy uses fixed levels. Future policy variants can be introduced without rewriting engine flow.

Why:
- Prevents scattered timing rules.
- Supports extension (for example, random per-transition timing) with lower risk.

### 5. Test Topology

Tests are organized under `tests/`:
- `tests/unit/` for module-focused tests
- `tests/behavior/` for grouped behavior tests across module surfaces
- `tests/end_to_end/` reserved for larger comprehensive scenarios

Nested directories are loaded via `#[path = "..."]` modules in the
library test harness (`src/lib.rs`) so host test runs avoid AVR binary
compilation.

Why:
- Keeps test intent visible from path names.
- Scales better as phase coverage grows.

### 6. TDD Execution Contract

- Red tests must compile.
- Compile failures are setup failures, not valid red state.
- If a new API is referenced by tests, add a minimal compiling stub first.

Why:
- Keeps TDD signals trustworthy.
- Reduces wasted cycles chasing unrelated compile errors.

## Build and Verification

Host-side test gate:
- `cargo test --target x86_64-unknown-linux-gnu`

AVR firmware build:
- `cargo build`
