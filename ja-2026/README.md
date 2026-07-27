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

## Build & Test

### Run Host-Side Unit Tests
Test the pure domain logic (relay modes, actions, input mapping, etc.) on the host without hardware:

```bash
./test.sh
```

Or manually:
```bash
cargo test --lib --target x86_64-unknown-linux-gnu -- --nocapture
```

### Build and Flash Firmware
Compile for AVR and upload to the board via USB:

```bash
./flash.sh
```

Or manually:
```bash
cargo run -Z build-std=core --release
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
1. Write/update failing tests in the relevant module (phase-specific test modules)
2. Run `./test.sh` to verify tests fail
3. Implement domain logic to pass tests
4. Run `./test.sh` again to verify all tests pass
5. Refactor for clarity (tests stay green)
6. Run `./flash.sh` to validate on hardware
7. Commit with clear message

