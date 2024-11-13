#!/bin/bash

# if anything fails, we don't release.
set -euo pipefail

# Run all tests
echo "Running tests: '$> cargo test'"
cargo test &> /dev/null

# Run all examples
echo "Running examples..."
for example in examples/*.rs; do
    example_name=$(basename "$example" .rs)
    echo "Running example: '$> cargo run --example $example_name'"
    cargo run --example "$example_name" &> /dev/null
done

# Run release script
echo "Running release process..."
cargo run --bin release