#!/bin/bash
# Run mutation testing on a specific crate
# Part of Certeza Tier 3 verification
# Reference: docs/specifications/improve-testing-quality-using-certeza-concepts.md
#
# Usage:
#   ./scripts/run-mutation-test.sh decy-ownership
#   ./scripts/run-mutation-test.sh --all

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

usage() {
    echo "Usage: $0 [CRATE_NAME | --all]"
    echo ""
    echo "Examples:"
    echo "  $0 decy-ownership    # Test single crate"
    echo "  $0 --all             # Test entire workspace (slow!)"
    echo ""
    echo "Note: Mutation testing is SLOW (Tier 3)"
    echo "  - Single crate: 10-30 minutes"
    echo "  - Full workspace: 1-4 hours"
    exit 1
}

if [ $# -eq 0 ]; then
    usage
fi

CRATE="$1"

cd "$PROJECT_ROOT"

echo -e "${BLUE}=== Mutation Testing: $CRATE ===${NC}"
echo ""

# Check if cargo-mutants is installed
if ! command -v cargo-mutants &> /dev/null; then
    echo -e "${YELLOW}Installing cargo-mutants...${NC}"
    cargo install cargo-mutants --locked
fi

# Run mutation testing
if [ "$CRATE" = "--all" ]; then
    echo -e "${YELLOW}Running mutation tests on entire workspace...${NC}"
    echo -e "${YELLOW}This will take 1-4 hours. Consider running in background.${NC}"
    echo ""

    cargo mutants --workspace \
        --timeout 300 \
        --output target/mutants.json \
        --output target/mutants.txt \
        --jobs 2

else
    echo -e "${YELLOW}Running mutation tests on $CRATE...${NC}"
    echo -e "${YELLOW}This may take 10-30 minutes.${NC}"
    echo ""

    cargo mutants -p "$CRATE" \
        --timeout 300 \
        --output "target/${CRATE}_mutants.json" \
        --output "target/${CRATE}_mutants.txt" \
        --jobs 2
fi

# Analyze results
echo ""
echo -e "${BLUE}=== Analyzing Results ===${NC}"

if [ "$CRATE" = "--all" ]; then
    python3 "$SCRIPT_DIR/analyze_mutations.py" target/mutants.json
else
    python3 "$SCRIPT_DIR/analyze_mutations.py" "target/${CRATE}_mutants.json"
fi

# Save results
if [ "$CRATE" = "--all" ]; then
    echo ""
    echo -e "${GREEN}Results saved to:${NC}"
    echo "  - target/mutants.json (machine-readable)"
    echo "  - target/mutants.txt (human-readable)"
else
    echo ""
    echo -e "${GREEN}Results saved to:${NC}"
    echo "  - target/${CRATE}_mutants.json"
    echo "  - target/${CRATE}_mutants.txt"
fi
