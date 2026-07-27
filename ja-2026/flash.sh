#!/bin/bash
# Flash AVR firmware to the board.
# Usage: ./flash.sh

set -e

echo "Building and flashing AVR firmware..."
cargo build -Z build-std=core --release
echo "✓ Firmware built successfully"
echo "✓ Device programmed via ravedude"
