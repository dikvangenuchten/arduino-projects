#!/bin/bash
# Build AVR firmware. Optionally flash after build.
# Usage:
#   ./build.sh           # build only
#   ./build.sh --flash   # build + flash

set -euo pipefail

if [[ $# -gt 1 ]]; then
    echo "Usage: ./build.sh [--flash]"
    exit 1
fi

if [[ $# -eq 1 && "$1" != "--flash" ]]; then
    echo "Unknown option: $1"
    echo "Usage: ./build.sh [--flash]"
    exit 1
fi

echo "Building AVR firmware..."
cargo build -Z build-std=core --release
echo "✓ Firmware build succeeded"

if [[ $# -eq 1 && "$1" == "--flash" ]]; then
    echo "Flashing AVR firmware..."
    cargo run -Z build-std=core --release
    echo "✓ Device programmed successfully"
fi
