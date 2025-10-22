#!/bin/bash
# Test all code examples in the mdBook
# This is run in CI and can be run locally
# FAILS CI if any code example doesn't compile or test fails

set -e

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Book TDD Verification"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo

# Check if mdbook is installed
if ! command -v mdbook &> /dev/null; then
    echo "❌ mdbook is not installed"
    echo
    echo "Install with:"
    echo "  cargo install mdbook"
    echo
    exit 1
fi

cd book

echo "▶ Testing all Rust code blocks in book..."
echo

# This command tests ALL ```rust code blocks in the book
# - Compiles each code block
# - Runs any #[test] functions
# - FAILS if any don't compile or tests fail
if mdbook test; then
    echo
    echo "✓ All book examples compile and tests pass!"
    echo
else
    echo
    echo "❌ Book examples failed to compile or tests failed"
    echo
    echo "This is a BLOCKING error. Fix the book examples before release."
    exit 1
fi

# Build the book to catch any other issues
echo "▶ Building book HTML..."
if mdbook build; then
    echo "✓ Book built successfully"
    echo
else
    echo "❌ Book build failed"
    exit 1
fi

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  ✓ Book TDD Verification Complete"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
