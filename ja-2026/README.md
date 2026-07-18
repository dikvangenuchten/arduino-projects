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

## Development

### Firmware check (AVR target)

Use this to verify the embedded firmware build for the board target:

```bash
cargo check -q
```

### Logic tests (host target)

Use this to run host-side unit tests for engine/scene logic:

```bash
cargo test --target x86_64-unknown-linux-gnu --lib
```

