#!/bin/bash
# Run Rust linter (clippy) on AVR firmware.
# Usage: ./clippy.sh

set -e

echo "Running clippy linter..."
cargo clippy -Z build-std=core
echo "✓ Clippy check passed"
