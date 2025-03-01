#!/bin/bash

# if anything fails, we don't release.
set -euo pipefail

# Check for uncommited changes
if [ -n "$(git status --porcelain)" ]; then
  echo "There are uncommited files. Please commit your changes before proceeding."
  exit 1
fi

# Check if gh is installed
if ! command -v gh &> /dev/null; then
    echo "Error: GitHub CLI (gh) is not installed. Please install it first:"
    echo "  brew install gh"
    exit 1
fi

# Check if we're on main or master branch
CURRENT_BRANCH=$(git branch --show-current)
if [ "$CURRENT_BRANCH" != "main" ] && [ "$CURRENT_BRANCH" != "master" ]; then
    echo "Error: Must be on 'main' or 'master' branch to release. Currently on '$CURRENT_BRANCH'"
    exit 1
fi

# Run cargo clippy
echo "Running clippy: '&> cargo clippy -- -D warnings'"
cargo clippy -- -D warnings

# Run cargo fmt
echo "Running fmt: '$> cargo fmt'"
cargo fmt

# Run all tests
echo "Running tests: '$> cargo test'"
cargo test

# Check if examples directory exists
if [ -d "examples" ]; then
    # Run all examples
    echo "Running examples..."
    for example in examples/*.rs; do
        if [ -f "$example" ]; then
            example_name=$(basename "$example" .rs)
            echo "Running example: '$> cargo run --example $example_name'"
            cargo run --example "$example_name"
        fi
    done
else
    echo "No examples directory found, skipping examples."
fi

# Add all changed files to the staging area
if git add .; then
  echo "Changes added to staging area."
else
  echo "No changes to add."
fi

# Commit the changes with the specified message
if git commit -m "fmt, test & example validation"; then
  echo "Changes committed."
else
  echo "No changes to commit."
fi

# Run release script
echo "Running release process..."
cargo run --bin release