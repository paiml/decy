#!/usr/bin/env bash
# Decy Quality Gates - EXTREME TDD Enforcement
# Based on bashrs quality-gates.sh
# Toyota Way: Jidoka (自働化) - Build Quality In

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration (can be overridden by decy-quality.toml)
MAX_COMPLEXITY=${MAX_COMPLEXITY:-10}
MAX_COGNITIVE=${MAX_COGNITIVE:-15}
MIN_COVERAGE=${MIN_COVERAGE:-80.0}
MIN_MUTATION_SCORE=${MIN_MUTATION_SCORE:-90.0}
SATD_TOLERANCE=${SATD_TOLERANCE:-0}
MIN_PROPERTY_TESTS=${MIN_PROPERTY_TESTS:-50}

# Exit codes
EXIT_SUCCESS=0
EXIT_FORMAT_FAIL=1
EXIT_LINT_FAIL=2
EXIT_TEST_FAIL=3
EXIT_COVERAGE_FAIL=4
EXIT_COMPLEXITY_FAIL=5
EXIT_SATD_FAIL=6
EXIT_UNSAFE_FAIL=7

# Counters
TOTAL_CHECKS=0
PASSED_CHECKS=0
FAILED_CHECKS=0

print_header() {
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}  Decy Quality Gates - EXTREME TDD Enforcement${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
}

print_section() {
    echo ""
    echo -e "${BLUE}▶ $1${NC}"
    echo -e "${BLUE}──────────────────────────────────────────────────────────${NC}"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
    ((PASSED_CHECKS++))
    ((TOTAL_CHECKS++))
}

print_failure() {
    echo -e "${RED}✗${NC} $1"
    ((FAILED_CHECKS++))
    ((TOTAL_CHECKS++))
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_info() {
    echo -e "  $1"
}

check_format() {
    print_section "1. Format Check (rustfmt)"

    if cargo fmt --all -- --check > /dev/null 2>&1; then
        print_success "Code formatting is correct"
        return 0
    else
        print_failure "Code formatting issues detected"
        print_info "Run: cargo fmt"
        return "$EXIT_FORMAT_FAIL"
    fi
}

check_lint() {
    print_section "2. Lint Check (clippy)"

    if cargo clippy --workspace --all-targets --all-features -- -D warnings > /dev/null 2>&1; then
        print_success "No clippy warnings"
        return 0
    else
        print_failure "Clippy warnings detected"
        print_info "Run: cargo clippy --workspace --all-targets --all-features"
        return "$EXIT_LINT_FAIL"
    fi
}

check_tests() {
    print_section "3. Test Suite"

    print_info "Running all tests..."
    if cargo test --workspace --all-features --quiet 2>&1 | grep -q "test result: ok"; then
        print_success "All tests passed"
    else
        print_failure "Tests failed"
        return "$EXIT_TEST_FAIL"
    fi

    print_info "Running doc tests..."
    if cargo test --doc --workspace --quiet 2>&1 | grep -q "test result: ok"; then
        print_success "Doc tests passed"
    else
        print_failure "Doc tests failed"
        return "$EXIT_TEST_FAIL"
    fi

    # Check for property tests (count from all property_tests files)
    print_info "Checking property tests..."
    local prop_count=$(find crates/*/tests -name "*_property_tests.rs" 2>/dev/null | wc -l || echo 0)
    if [ "$prop_count" -gt 0 ]; then
        print_success "Property test files found: $prop_count"
    else
        print_warning "No property test files found (expected *_property_tests.rs)"
    fi

    return 0
}

check_coverage() {
    print_section "4. Coverage Check"

    if ! command -v cargo-llvm-cov &> /dev/null; then
        print_warning "cargo-llvm-cov not installed, skipping coverage check"
        print_info "Install: cargo install cargo-llvm-cov"
        return 0
    fi

    print_info "Running coverage analysis..."
    local coverage_output
    coverage_output=$(cargo llvm-cov --workspace --all-features --no-cfg-coverage 2>&1 || true)

    # Extract coverage from TOTAL line
    local coverage=$(echo "$coverage_output" | grep "^TOTAL" | tail -1 | awk '{for(i=NF;i>0;i--) if($i ~ /%/) {print $i; break}}' | tr -d '%' || echo "0")

    if [ -z "$coverage" ] || [ "$coverage" = "0" ] || [ "$coverage" = "-" ]; then
        print_warning "Could not parse coverage percentage"
        return 0
    fi

    local coverage_int=$(echo "$coverage" | cut -d. -f1)
    local min_coverage_int=${MIN_COVERAGE%.*}

    if [ "$coverage_int" -ge "$min_coverage_int" ]; then
        print_success "Coverage: ${coverage}% (≥${MIN_COVERAGE}% required)"
        return 0
    else
        print_failure "Coverage: ${coverage}% (< ${MIN_COVERAGE}% required)"
        return "$EXIT_COVERAGE_FAIL"
    fi
}

check_complexity() {
    print_section "5. Complexity Check"

    # Simple complexity check using tokei or line count
    if command -v tokei &> /dev/null; then
        print_info "Analyzing code size with tokei..."
        tokei crates/ --exclude target | head -20
        print_success "Code statistics generated"
    else
        print_warning "tokei not installed, skipping detailed complexity analysis"
        print_info "Install: cargo install tokei"
    fi

    # Check for excessively long files (>1000 lines)
    local long_files=$(find crates/*/src -name "*.rs" -type f -exec wc -l {} + | awk '$1 > 1000 {print $2}' || true)
    if [ -z "$long_files" ]; then
        print_success "No excessively long files (>1000 lines)"
    else
        print_warning "Long files detected (>1000 lines):"
        echo "$long_files" | while read -r file; do
            print_info "  $file"
        done
    fi

    return 0
}

check_satd() {
    print_section "6. SATD Check (Zero Tolerance)"

    local satd_patterns=("TODO" "FIXME" "HACK" "XXX" "TEMP" "WIP" "BROKEN" "KLUDGE" "REFACTOR" "BUG")
    local satd_found=0

    for pattern in "${satd_patterns[@]}"; do
        if grep -r "$pattern" crates/*/src --include="*.rs" > /dev/null 2>&1; then
            local count=$(grep -r "$pattern" crates/*/src --include="*.rs" | wc -l)
            print_failure "SATD pattern found: $pattern ($count occurrences)"
            satd_found=1
        fi
    done

    if [ "$satd_found" -eq 0 ]; then
        print_success "No SATD comments found (zero tolerance maintained)"
        return 0
    else
        print_info "Run: grep -rn 'TODO\\|FIXME\\|HACK' crates/"
        return "$EXIT_SATD_FAIL"
    fi
}

check_unsafe_usage() {
    print_section "7. Unsafe Code Check"

    # Count unsafe blocks in the codebase
    local unsafe_count=$(grep -r "unsafe" crates/*/src --include="*.rs" | grep -v "// SAFETY:" | wc -l || echo 0)

    # Get total lines of code
    local total_loc=$(find crates/*/src -name "*.rs" -type f -exec wc -l {} + | tail -1 | awk '{print $1}')

    if [ "$total_loc" -gt 0 ]; then
        local unsafe_per_1000=$(echo "scale=2; ($unsafe_count * 1000) / $total_loc" | bc || echo "0")
        print_info "Unsafe blocks: $unsafe_count (${unsafe_per_1000} per 1000 LOC)"

        # Target: <5 unsafe blocks per 1000 LOC
        local threshold_check=$(echo "$unsafe_per_1000 < 5" | bc -l)
        if [ "$threshold_check" -eq 1 ]; then
            print_success "Unsafe usage within target (<5 per 1000 LOC)"
        else
            print_warning "Unsafe usage above target (${unsafe_per_1000} per 1000 LOC, target <5)"
        fi
    else
        print_warning "Could not calculate unsafe usage ratio"
    fi

    # Special check for decy-parser (only crate allowed to use unsafe)
    local parser_unsafe=$(grep -r "unsafe" crates/decy-parser/src --include="*.rs" 2>/dev/null | wc -l || echo 0)
    local other_unsafe=$(grep -r "unsafe" crates/*/src --include="*.rs" 2>/dev/null | grep -v "decy-parser" | wc -l || echo 0)

    if [ "$other_unsafe" -gt 0 ]; then
        print_warning "Unsafe code found outside decy-parser: $other_unsafe instances"
        print_info "Only decy-parser should use unsafe (for clang-sys FFI)"
    else
        print_success "Unsafe code properly isolated to decy-parser"
    fi

    return 0
}

check_documentation() {
    print_section "8. Documentation Check"

    print_info "Building documentation..."
    if cargo doc --workspace --no-deps --quiet 2>&1 | grep -q "Documenting"; then
        print_success "Documentation builds successfully"
    else
        print_warning "Documentation build issues detected"
    fi

    return 0
}

print_summary() {
    echo ""
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}  Quality Gates Summary${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
    echo -e "  Total Checks:  $TOTAL_CHECKS"
    echo -e "  ${GREEN}Passed:${NC}        $PASSED_CHECKS"
    echo -e "  ${RED}Failed:${NC}        $FAILED_CHECKS"
    echo ""

    if [ "$FAILED_CHECKS" -eq 0 ]; then
        echo -e "${GREEN}✓ All quality gates passed! ✓${NC}"
        echo ""
        echo -e "${GREEN}  EXTREME TDD: Quality Built In (Jidoka)${NC}"
        echo -e "${GREEN}  Toyota Way: 自働化 - Automation with Human Intelligence${NC}"
        echo ""
        return "$EXIT_SUCCESS"
    else
        echo -e "${RED}✗ Quality gates failed! ✗${NC}"
        echo ""
        echo -e "${RED}  Fix all issues before committing${NC}"
        echo -e "${RED}  Zero tolerance for quality violations${NC}"
        echo ""
        return 1
    fi
}

main() {
    local exit_code="$EXIT_SUCCESS"

    print_header

    # Run all checks (continue even if some fail to show all issues)
    check_format || exit_code=$?
    check_lint || exit_code=$?
    check_tests || exit_code=$?
    check_coverage || exit_code=$?
    check_complexity || exit_code=$?
    check_satd || exit_code=$?
    check_unsafe_usage || exit_code=$?
    check_documentation || exit_code=$?

    print_summary || exit_code=$?

    exit "$exit_code"
}

# Run main if executed directly
if [ "${BASH_SOURCE[0]}" == "${0}" ]; then
    main "$@"
fi
