#!/bin/bash
# Run host-side unit tests for the pure domain logic.
# Usage: ./test.sh

set -e

echo "Running host-side tests..."
cargo test --lib --target x86_64-unknown-linux-gnu -- --nocapture
echo "✓ All tests passed"
