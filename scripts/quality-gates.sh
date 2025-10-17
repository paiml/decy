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
    # Run coverage and extract percentage from TOTAL line (Lines Cover column)
    COVERAGE_OUTPUT=$(cargo llvm-cov --workspace --all-features --no-cfg-coverage 2>&1)
    # Extract the last TOTAL line and get the last "Cover" column value (89.60%)
    COVERAGE=$(echo "$COVERAGE_OUTPUT" | grep "^TOTAL" | tail -1 | awk '{for(i=NF;i>0;i--) if($i ~ /%/) {print $i; break}}' | tr -d '%' || echo "0")

    # Handle empty coverage value
    if [ -z "$COVERAGE" ] || [ "$COVERAGE" = "0" ] || [ "$COVERAGE" = "-" ]; then
        echo -e "${YELLOW}⚠️  Could not parse coverage percentage${NC}"
        echo "Coverage output:"
        echo "$COVERAGE_OUTPUT" | tail -10
    else
        # Extract integer part
        COVERAGE_INT=$(echo "$COVERAGE" | cut -d'.' -f1)

        if [ "$COVERAGE_INT" -ge 80 ]; then
            echo -e "${GREEN}✅ Coverage: ${COVERAGE}% (≥80%)${NC}"
        else
            echo -e "${RED}❌ Coverage: ${COVERAGE}% (<80%)${NC}"
            echo "Add more tests to reach 80% minimum coverage"
            FAILED=1
        fi
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

# 6a. Check PMAT Entropy
echo "🎲 Checking PMAT entropy..."
if command -v pmat &> /dev/null; then
    # Check entropy for all staged Rust files
    STAGED_FILES=$(git diff --cached --name-only --diff-filter=ACM | grep -E '\.rs$' || true)
    if [ -n "$STAGED_FILES" ]; then
        ENTROPY_FAILED=0
        for file in $STAGED_FILES; do
            if [ -f "$file" ]; then
                # pmat entropy returns 0 if entropy is acceptable
                if ! pmat entropy "$file" --threshold 0.8 --quiet; then
                    echo -e "${RED}❌ High entropy detected in: $file${NC}"
                    ENTROPY_FAILED=1
                fi
            fi
        done

        if [ $ENTROPY_FAILED -eq 0 ]; then
            echo -e "${GREEN}✅ Entropy check passed (all files <80% threshold)${NC}"
        else
            echo -e "${RED}❌ Entropy check failed${NC}"
            echo "Files have high entropy (randomness). Consider:"
            echo "  - Breaking down complex logic"
            echo "  - Adding meaningful comments"
            echo "  - Improving code structure"
            FAILED=1
        fi
    else
        echo -e "${GREEN}✅ No Rust files to check${NC}"
    fi
else
    echo -e "${YELLOW}⚠️  pmat not installed, skipping entropy check${NC}"
    echo "Install with: cargo install pmat"
fi
echo ""

# 6b. Check PMAT Complexity (Cyclomatic <10)
echo "🔢 Checking PMAT complexity (cyclomatic <10)..."
if command -v pmat &> /dev/null; then
    # Check complexity for all staged Rust files
    STAGED_FILES=$(git diff --cached --name-only --diff-filter=ACM | grep -E '\.rs$' || true)
    if [ -n "$STAGED_FILES" ]; then
        COMPLEXITY_FAILED=0
        for file in $STAGED_FILES; do
            if [ -f "$file" ]; then
                # pmat complexity --max-cyclomatic 10 returns non-zero if any function exceeds threshold
                if ! pmat complexity "$file" --max-cyclomatic 10 --quiet; then
                    echo -e "${RED}❌ High complexity detected in: $file${NC}"
                    COMPLEXITY_FAILED=1
                fi
            fi
        done

        if [ $COMPLEXITY_FAILED -eq 0 ]; then
            echo -e "${GREEN}✅ Complexity check passed (all functions ≤10)${NC}"
        else
            echo -e "${RED}❌ Complexity check failed (functions >10 cyclomatic complexity)${NC}"
            echo "Refactor complex functions to reduce cyclomatic complexity:"
            echo "  - Extract helper functions"
            echo "  - Simplify control flow"
            echo "  - Break down large functions"
            echo "  - Use early returns"
            FAILED=1
        fi
    else
        echo -e "${GREEN}✅ No Rust files to check${NC}"
    fi
else
    echo -e "${YELLOW}⚠️  pmat not installed, skipping complexity check${NC}"
    echo "Install with: cargo install pmat"
fi
echo ""

# 6c. Check PMAT TDG (Technical Debt Grade)
echo "📈 Checking PMAT TDG (Technical Debt Grade)..."
if command -v pmat &> /dev/null; then
    # Check TDG for all staged Rust files
    STAGED_FILES=$(git diff --cached --name-only --diff-filter=ACM | grep -E '\.rs$' || true)
    if [ -n "$STAGED_FILES" ]; then
        TDG_FAILED=0
        for file in $STAGED_FILES; do
            if [ -f "$file" ]; then
                # pmat tdg returns grade A-F, fail if below threshold
                TDG_OUTPUT=$(pmat tdg "$file" 2>&1 || true)
                # Extract grade (assumes format like "Grade: A" or just "A")
                GRADE=$(echo "$TDG_OUTPUT" | grep -oE '[A-F]' | head -1 || echo "F")

                # Fail if grade is D, E, or F
                if [[ "$GRADE" == "D" || "$GRADE" == "E" || "$GRADE" == "F" ]]; then
                    echo -e "${RED}❌ Poor TDG in: $file (Grade: $GRADE)${NC}"
                    TDG_FAILED=1
                fi
            fi
        done

        if [ $TDG_FAILED -eq 0 ]; then
            echo -e "${GREEN}✅ TDG check passed (all files grade A-C)${NC}"
        else
            echo -e "${RED}❌ TDG check failed (files with grade D-F)${NC}"
            echo "Improve technical debt grade by:"
            echo "  - Reducing complexity"
            echo "  - Lowering entropy"
            echo "  - Adding documentation"
            echo "  - Removing SATD comments"
            echo "  - Improving code structure"
            FAILED=1
        fi
    else
        echo -e "${GREEN}✅ No Rust files to check${NC}"
    fi
else
    echo -e "${YELLOW}⚠️  pmat not installed, skipping TDG check${NC}"
    echo "Install with: cargo install pmat"
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

# 8. Validate Documentation Links (using PMAT)
echo "🔗 Validating documentation links..."
if command -v pmat &> /dev/null; then
    if pmat validate-docs --fail-on-error --quiet; then
        echo -e "${GREEN}✅ Documentation links valid${NC}"
    else
        echo -e "${RED}❌ Broken documentation links found${NC}"
        FAILED=1
    fi
else
    echo -e "${YELLOW}⚠️  pmat not installed, skipping link validation${NC}"
    echo "Install with: cargo install pmat"
fi
echo ""

# 9. Doc Check
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
    echo "  • PMAT entropy < 80%"
    echo "  • PMAT cyclomatic complexity ≤ 10"
    echo "  • PMAT TDG grade ≥ C (no D, E, F)"
    echo "  • Documentation complete"
    echo ""
    exit 1
fi
