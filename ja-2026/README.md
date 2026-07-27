# ja-2026
=======

Rust project for the _SparkFun ProMini 5v_.

## Hardware
### Board
- Arduino - Pro mini ATMEGA328P 5V/16M
- Eletechsup - IO22D08
   - 8 relays
   - 8 inputs
   - 4 7-segment displays
   - 4 physical buttons

## Behavior

### External inputs (control actions)

| Input | Unshifted | Shifted |
| --- | --- | --- |
| I0-I3 | Toggle power (Off <-> On) on relays 1-4 | Toggle power (Off <-> On) on relays 5-8 |
| I4 | Shift (momentary; held only) | - |
| I5 | Global power: all included relays On if currently all Off, else all Off | Global blink: force all included relays to Blink, preserving speeds |
| I6 | Speed up: move every relay's speed toward shorter half-cycles | Speed down: move every relay's speed toward longer half-cycles |
| I7 | Reserved (no-op) | Reserved (no-op) |

All eight inputs are active-low and debounced, accepting a change only
after 10 consecutive 1 ms samples. Shift is momentary: a Shift press
accepted on the same debounced tick as an action press already applies to
that action. The Speed key changes all eight relays' stored speeds
together (each clamped to its own nearest level independently), even
though each relay is free to diverge afterward via other actions.

### Built-in buttons (diagnostics-only)

Buttons never change relay state; they only select what the 4-digit
display shows.

| Button | Effect |
| --- | --- |
| B0 | Select the Speed view |
| B1 | Select the Inputs view; pressing again while already selected toggles between inputs I0-I3 and I4-I7 |
| B2 | Select the Relay view; pressing again while already selected toggles between relays 1-4 and 5-8 |
| B3 | Reserved (no-op) |

- **Speed view**: shows the most common stored per-relay half-cycle
  duration (`1200`, `0800`, `0500`, `0300`, or `0150`); ties choose the
  largest value.
- **Inputs view**: shows each of the 4 selected input lines as `0`/`1`.
- **Relay view**: shows each of the 4 selected relay modes as `0`=Off,
  `1`=Blink, `2`=On.

The display boots on the Speed view.

### Relay modes and timing

Each relay independently owns its mode (`Off`, `Blink`, `On`) and its
speed, one of `[1200, 800, 500, 300, 150]` ms. `Blink` alternates the
output every half-cycle at that stored speed; there is no shared timing
or shared "current speed" across relays, so relays can blink at
different speeds and different phase offsets simultaneously. Changing a
relay's speed while it is blinking only takes effect once its current
half-cycle finishes.

### Boot defaults

All 8 relays boot `Off` with speed index 2 (500 ms). The diagnostic
display boots on the Speed view. Nothing is persisted across power
cycles (no EEPROM).

## Build & Test

### Run Host-Side Tests
Test the pure domain logic (relay modes, actions, input mapping, etc.) on the host without hardware:

```bash
./test.sh
```

Or manually:
```bash
cargo test --lib --target x86_64-unknown-linux-gnu -- --nocapture
```

### Test Directory Structure

Tests are organized under `tests/`:

- `tests/unit/action_toggle_tests.rs` for toggle action unit tests
- `tests/unit/action_global_tests.rs` for global action unit tests
- `tests/unit/action_speed_tests.rs` for speed action unit tests
- `tests/behavior/blink_state_behavior.rs` for grouped blink state/phase behaviors
- `tests/behavior/blink_speed_behavior.rs` for grouped blink speed behaviors
- `tests/end_to_end/` reserved for larger cross-module scenarios

Nested directories are wired into the library test harness in `src/lib.rs`
using path-based modules.

### Build and Flash Firmware
Compile for AVR and upload to the board via USB:

```bash
./build.sh
```

Or manually:
```bash
cargo build -Z build-std=core --release
```

Build and flash in one command:

```bash
./build.sh --flash
```

The `ravedude` runner will automatically detect the serial device and program it.

### Lint Code with Clippy
Check for common Rust mistakes and style improvements:

```bash
./clippy.sh
```

Or manually:
```bash
cargo clippy -Z build-std=core
```

### Development Workflow
1. Write/update failing tests in `tests/unit/` or `tests/behavior/`
2. Run `./test.sh` to verify tests fail
3. Implement domain logic to pass tests
4. Run `./test.sh` again to verify all tests pass
5. Refactor for clarity (tests stay green)
6. Run `./build.sh --flash` to validate on hardware
7. Commit with clear message

