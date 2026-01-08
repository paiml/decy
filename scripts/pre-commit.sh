#!/usr/bin/env bash
# Decy Pre-commit Hook
# Install: cp scripts/pre-commit.sh .git/hooks/pre-commit && chmod +x .git/hooks/pre-commit

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

echo "Running pre-commit quality checks..."

# Fast checks only (full quality gates run in CI)

# 1. Format check
echo -n "Checking formatting... "
if cargo fmt --check --quiet 2>/dev/null; then
    echo -e "${GREEN}OK${NC}"
else
    echo -e "${RED}FAIL${NC}"
    echo "Run: cargo fmt"
    exit 1
fi

# 2. Clippy
echo -n "Running clippy... "
if cargo clippy --workspace --quiet -- -D warnings 2>/dev/null; then
    echo -e "${GREEN}OK${NC}"
else
    echo -e "${RED}FAIL${NC}"
    echo "Run: cargo clippy --workspace -- -D warnings"
    exit 1
fi

# 3. Unit tests (fast)
echo -n "Running unit tests... "
if cargo test --workspace --lib --quiet 2>/dev/null; then
    echo -e "${GREEN}OK${NC}"
else
    echo -e "${RED}FAIL${NC}"
    exit 1
fi

# 4. SATD check (zero tolerance)
echo -n "Checking for SATD... "
if grep -rqE "(TODO|FIXME|HACK|XXX)" crates/*/src/*.rs 2>/dev/null; then
    echo -e "${RED}FAIL${NC}"
    echo "SATD comments found. Remove before committing."
    exit 1
else
    echo -e "${GREEN}OK${NC}"
fi

echo -e "${GREEN}Pre-commit checks passed!${NC}"
