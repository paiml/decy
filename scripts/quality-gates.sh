#!/usr/bin/env bash
# Decy Quality Gates Pre-Commit Hook
# Enforces EXTREME quality standards before allowing commits
# Based on: decy-quality.toml

set -e  # Exit on first error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "🔍 Running Decy Quality Gates..."
echo ""

FAILED=0

# 1. Check Formatting
echo "📝 Checking code formatting..."
if cargo fmt -- --check; then
    echo -e "${GREEN}✅ Formatting check passed${NC}"
else
    echo -e "${RED}❌ Formatting check failed${NC}"
    echo "Run: cargo fmt"
    FAILED=1
fi
echo ""

# 2. Run Clippy (Zero Warnings Policy)
echo "🔍 Running clippy (zero warnings policy)..."
if cargo clippy --workspace --all-targets --all-features -- -D warnings; then
    echo -e "${GREEN}✅ Clippy passed (0 warnings)${NC}"
else
    echo -e "${RED}❌ Clippy failed (warnings or errors found)${NC}"
    echo "Fix all clippy warnings before committing"
    FAILED=1
fi
echo ""

# 3. Check for SATD Comments (Self-Admitted Technical Debt)
echo "🚫 Checking for SATD comments (TODO, FIXME, HACK, XXX)..."
SATD_FOUND=$(git diff --cached --name-only | grep -E '\.rs$' | xargs grep -nE '(TODO|FIXME|HACK|XXX|TEMP|WIP|BROKEN)' || true)
if [ -z "$SATD_FOUND" ]; then
    echo -e "${GREEN}✅ No SATD comments found${NC}"
else
    echo -e "${RED}❌ SATD comments detected:${NC}"
    echo "$SATD_FOUND"
    echo ""
    echo "Remove all TODO/FIXME/HACK/XXX comments before committing"
    FAILED=1
fi
echo ""

# 4. Run Tests
echo "🧪 Running all tests..."
if cargo test --workspace --all-features; then
    echo -e "${GREEN}✅ All tests passed${NC}"
else
    echo -e "${RED}❌ Tests failed${NC}"
    FAILED=1
fi
echo ""

# 5. Check Coverage (≥80% required)
echo "📊 Checking test coverage (≥80% required)..."
if command -v cargo-llvm-cov &> /dev/null; then
    COVERAGE=$(cargo llvm-cov --workspace --all-features --summary-only 2>&1 | grep -oP 'lines:\s+\K[\d.]+' | head -1 || echo "0")
    COVERAGE_INT=${COVERAGE%.*}

    if [ "$COVERAGE_INT" -ge 80 ]; then
        echo -e "${GREEN}✅ Coverage: ${COVERAGE}% (≥80%)${NC}"
    else
        echo -e "${RED}❌ Coverage: ${COVERAGE}% (<80%)${NC}"
        echo "Add more tests to reach 80% minimum coverage"
        FAILED=1
    fi
else
    echo -e "${YELLOW}⚠️  cargo-llvm-cov not installed, skipping coverage check${NC}"
    echo "Install with: cargo install cargo-llvm-cov"
fi
echo ""

# 6. Check Complexity (if tool available)
echo "🧮 Checking code complexity..."
if command -v cargo-geiger &> /dev/null; then
    # Check for unsafe code usage
    UNSAFE_COUNT=$(cargo geiger --output-format json 2>/dev/null | grep -o '"unsafe"' | wc -l || echo "0")
    echo "Unsafe blocks found: $UNSAFE_COUNT"
    echo -e "${GREEN}ℹ️  Unsafe usage will be tracked per sprint${NC}"
else
    echo -e "${YELLOW}⚠️  cargo-geiger not installed, skipping unsafe check${NC}"
fi
echo ""

# 7. Build Check
echo "🔨 Running build check..."
if cargo build --workspace --all-features; then
    echo -e "${GREEN}✅ Build successful${NC}"
else
    echo -e "${RED}❌ Build failed${NC}"
    FAILED=1
fi
echo ""

# 8. Doc Check
echo "📚 Checking documentation..."
if RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --document-private-items; then
    echo -e "${GREEN}✅ Documentation builds without warnings${NC}"
else
    echo -e "${RED}❌ Documentation has warnings or errors${NC}"
    FAILED=1
fi
echo ""

# Final Result
echo "═══════════════════════════════════════════"
if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✅ ALL QUALITY GATES PASSED${NC}"
    echo "Commit approved ✅"
    exit 0
else
    echo -e "${RED}❌ QUALITY GATES FAILED${NC}"
    echo ""
    echo "Fix the issues above before committing."
    echo ""
    echo "Quality Standards:"
    echo "  • Coverage ≥ 80%"
    echo "  • 0 clippy warnings"
    echo "  • 0 SATD comments (TODO/FIXME/HACK)"
    echo "  • All tests passing"
    echo "  • Code formatted"
    echo "  • Documentation complete"
    echo ""
    exit 1
fi
