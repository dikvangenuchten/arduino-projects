#!/bin/bash
# Flash AVR firmware to the board.
# Usage: ./flash.sh

set -e

echo "Building and flashing AVR firmware..."
cargo run -Z build-std=core --release
echo "✓ Device programmed successfully"
