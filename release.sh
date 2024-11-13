#!/bin/bash

# if anything fails, we don't release.
set -euo pipefail

# Run all tests
echo "Running tests..."
cargo test

# Run all examples
echo "Running examples..."
cargo run --examples

# Run release script
echo "Running release process..."
cargo run --bin release