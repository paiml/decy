#!/bin/bash
# Tier 1: ON-SAVE Testing (Sub-Second Feedback)
# Goal: <1 second execution to maintain flow state
# Reference: docs/specifications/improve-testing-quality-using-certeza-concepts.md

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT"

echo -e "${YELLOW}🔍 Tier 1: ON-SAVE (Target: <1s)${NC}"

START_TIME=$(date +%s%3N)

# 1. Fast unit tests only (lib tests, no integration tests)
echo -n "  ⚡ Unit tests (fast)... "
if cargo test --lib --quiet 2>&1 | grep -q "test result: ok"; then
    echo -e "${GREEN}✓${NC}"
else
    echo -e "${RED}✗${NC}"
    echo -e "${RED}Unit tests failed - run 'cargo test --lib' for details${NC}"
    exit 1
fi

# 2. Quick clippy check (just errors, not all lints)
echo -n "  📎 Clippy (quick)... "
if cargo clippy --quiet --all-targets -- -D warnings 2>&1 | head -1 | grep -q "Checking\|Finished"; then
    echo -e "${GREEN}✓${NC}"
else
    echo -e "${RED}✗${NC}"
    echo -e "${RED}Clippy warnings found - run 'cargo clippy' for details${NC}"
    exit 1
fi

# 3. Format check
echo -n "  🎨 Format check... "
if cargo fmt --check --quiet 2>&1; then
    echo -e "${GREEN}✓${NC}"
else
    echo -e "${RED}✗${NC}"
    echo -e "${RED}Format issues - run 'cargo fmt'${NC}"
    exit 1
fi

# 4. SATD detection (fast grep)
echo -n "  🚫 SATD comments... "
SATD_COUNT=$(grep -r "TODO\|FIXME\|HACK\|XXX\|TEMP\|WIP\|BROKEN" \
    --include="*.rs" \
    --exclude-dir=target \
    --exclude-dir=.git \
    crates/ 2>/dev/null | \
    grep -v "RED-GREEN-REFACTOR\|TDD.*REFACTOR\|REFACTOR phase\|REFACTOR)" | \
    grep -v "// BUG:\|// ERROR:\|C: .*BUG\|\".*BUG.*\"" | \
    wc -l || echo "0")

if [ "$SATD_COUNT" -eq 0 ]; then
    echo -e "${GREEN}✓${NC}"
else
    echo -e "${RED}✗ Found $SATD_COUNT SATD comments${NC}"
    exit 1
fi

END_TIME=$(date +%s%3N)
ELAPSED=$((END_TIME - START_TIME))

if [ $ELAPSED -lt 1000 ]; then
    echo -e "${GREEN}✅ Tier 1 passed in ${ELAPSED}ms (<1s target)${NC}"
else
    echo -e "${YELLOW}⚠️  Tier 1 passed in ${ELAPSED}ms (>1s, optimize!)${NC}"
fi

exit 0
