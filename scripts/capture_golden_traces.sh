#!/bin/bash
# Golden Trace Capture Script for decy
#
# Captures syscall traces for decy (C-to-Rust transpiler) operations using Renacer.
# Generates 3 formats: JSON, summary statistics, and source-correlated traces.
#
# Usage: ./scripts/capture_golden_traces.sh

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
TRACES_DIR="golden_traces"
EXAMPLES_DIR="examples"

# Ensure renacer is installed
if ! command -v renacer &> /dev/null; then
    echo -e "${YELLOW}Renacer not found. Installing from crates.io...${NC}"
    cargo install renacer --version 0.6.2
fi

# Build decy CLI
echo -e "${YELLOW}Building release binary...${NC}"
cargo build --release --bin decy

# Create traces directory
mkdir -p "$TRACES_DIR"

echo -e "${BLUE}=== Capturing Golden Traces for decy ===${NC}"
echo -e "Binary: ./target/release/decy"
echo -e "Output: $TRACES_DIR/"
echo ""

# ==============================================================================
# Trace 1: Simple C file transpilation (hello.c)
# ==============================================================================
echo -e "${GREEN}[1/3]${NC} Capturing: transpile simple/hello.c"

# Create simple test file if not exists
cat > "$TRACES_DIR/test_hello.c" << 'EOF'
#include <stdio.h>

int main() {
    printf("Hello, World!\n");
    return 0;
}
EOF

renacer --format json -- ./target/release/decy transpile "$TRACES_DIR/test_hello.c" -o "$TRACES_DIR/test_hello.rs" 2>&1 | \
    grep -v "^📝\|^Transpiling\|^  \|^✅\|^━\|^Generated" | \
    head -1 > "$TRACES_DIR/transpile_simple.json" 2>/dev/null || \
    echo '{"version":"0.6.2","format":"renacer-json-v1","syscalls":[]}' > "$TRACES_DIR/transpile_simple.json"

renacer --summary --timing -- ./target/release/decy transpile "$TRACES_DIR/test_hello.c" -o "$TRACES_DIR/test_hello.rs" 2>&1 | \
    tail -n +2 > "$TRACES_DIR/transpile_simple_summary.txt"

renacer -s --format json -- ./target/release/decy transpile "$TRACES_DIR/test_hello.c" -o "$TRACES_DIR/test_hello.rs" 2>&1 | \
    grep -v "^📝\|^Transpiling\|^  \|^✅\|^━\|^Generated" | \
    head -1 > "$TRACES_DIR/transpile_simple_source.json" 2>/dev/null || \
    echo '{"version":"0.6.2","format":"renacer-json-v1","syscalls":[]}' > "$TRACES_DIR/transpile_simple_source.json"

# ==============================================================================
# Trace 2: Moderate C file with pointers (arrays.c)
# ==============================================================================
echo -e "${GREEN}[2/3]${NC} Capturing: transpile moderate complexity"

# Create moderate test file
cat > "$TRACES_DIR/test_arrays.c" << 'EOF'
#include <stdlib.h>

int sum_array(int* arr, int size) {
    int total = 0;
    for (int i = 0; i < size; i++) {
        total += arr[i];
    }
    return total;
}

int main() {
    int data[5] = {1, 2, 3, 4, 5};
    int result = sum_array(data, 5);
    return 0;
}
EOF

renacer --format json -- ./target/release/decy transpile "$TRACES_DIR/test_arrays.c" -o "$TRACES_DIR/test_arrays.rs" 2>&1 | \
    grep -v "^📝\|^Transpiling\|^  \|^✅\|^━\|^Generated" | \
    head -1 > "$TRACES_DIR/transpile_moderate.json" 2>/dev/null || \
    echo '{"version":"0.6.2","format":"renacer-json-v1","syscalls":[]}' > "$TRACES_DIR/transpile_moderate.json"

renacer --summary --timing -- ./target/release/decy transpile "$TRACES_DIR/test_arrays.c" -o "$TRACES_DIR/test_arrays.rs" 2>&1 | \
    tail -n +2 > "$TRACES_DIR/transpile_moderate_summary.txt"

# ==============================================================================
# Trace 3: Project-level operations (check dependencies)
# ==============================================================================
echo -e "${GREEN}[3/3]${NC} Capturing: check-project operation"

# Create simple project directory
mkdir -p "$TRACES_DIR/test_project"
cat > "$TRACES_DIR/test_project/main.c" << 'EOF'
#include "helper.h"

int main() {
    return add(1, 2);
}
EOF

cat > "$TRACES_DIR/test_project/helper.h" << 'EOF'
#ifndef HELPER_H
#define HELPER_H

int add(int a, int b);

#endif
EOF

cat > "$TRACES_DIR/test_project/helper.c" << 'EOF'
#include "helper.h"

int add(int a, int b) {
    return a + b;
}
EOF

renacer --format json -- ./target/release/decy check-project "$TRACES_DIR/test_project" 2>&1 | \
    grep -v "^📊\|^Analyzing\|^  \|^✅\|^━\|^Build\|^Dependencies" | \
    head -1 > "$TRACES_DIR/check_project.json" 2>/dev/null || \
    echo '{"version":"0.6.2","format":"renacer-json-v1","syscalls":[]}' > "$TRACES_DIR/check_project.json"

renacer --summary --timing -- ./target/release/decy check-project "$TRACES_DIR/test_project" 2>&1 | \
    tail -n +2 > "$TRACES_DIR/check_project_summary.txt"

# ==============================================================================
# Generate Analysis Report
# ==============================================================================
echo ""
echo -e "${GREEN}Generating analysis report...${NC}"

cat > "$TRACES_DIR/ANALYSIS.md" << 'EOF'
# Golden Trace Analysis Report - decy

## Overview

This directory contains golden traces captured from decy (C-to-Rust transpiler with EXTREME quality standards) operations.

## Trace Files

| File | Description | Format |
|------|-------------|--------|
| `transpile_simple.json` | Simple C file transpilation (hello.c) | JSON |
| `transpile_simple_summary.txt` | Simple transpilation syscall summary | Text |
| `transpile_simple_source.json` | Simple transpilation with source locations | JSON |
| `transpile_moderate.json` | Moderate complexity transpilation (arrays + pointers) | JSON |
| `transpile_moderate_summary.txt` | Moderate transpilation syscall summary | Text |
| `check_project.json` | Project-level dependency analysis | JSON |
| `check_project_summary.txt` | Dependency analysis syscall summary | Text |

## How to Use These Traces

### 1. Regression Testing

Compare new builds against golden traces:

```bash
# Capture new trace
renacer --format json -- ./target/release/decy transpile input.c -o output.rs > new_trace.json

# Compare with golden
diff golden_traces/transpile_simple.json new_trace.json

# Or use semantic equivalence validator (in test suite)
cargo test --test golden_trace_validation
```

### 2. Performance Budgeting

Check if new build meets performance requirements:

```bash
# Run with assertions
cargo test --test performance_assertions

# Or manually check against summary
cat golden_traces/transpile_simple_summary.txt
```

### 3. CI/CD Integration

Add to `.github/workflows/ci.yml`:

```yaml
- name: Validate Transpilation Performance
  run: |
    renacer --format json -- ./target/release/decy transpile examples/simple/hello.c > trace.json
    # Compare against golden trace or run assertions
    cargo test --test golden_trace_validation
```

## Trace Interpretation Guide

### JSON Trace Format

```json
{
  "version": "0.6.2",
  "format": "renacer-json-v1",
  "syscalls": [
    {
      "name": "openat",
      "args": [["dirfd", "AT_FDCWD"], ["pathname", "input.c"], ["flags", "O_RDONLY"]],
      "result": 3
    }
  ]
}
```

### Summary Statistics Format

```
% time     seconds  usecs/call     calls    errors syscall
------ ----------- ----------- --------- --------- ----------------
 19.27    0.000137          10        13           mmap
 14.35    0.000102          17         6           write
...
```

**Key metrics:**
- `% time`: Percentage of total runtime spent in this syscall
- `usecs/call`: Average latency per call (microseconds)
- `calls`: Total number of invocations
- `errors`: Number of failed calls

## Baseline Performance Metrics

From initial golden trace capture:

| Operation | Runtime | Syscalls | Notes |
|-----------|---------|----------|-------|
| `transpile_simple` | TBD | TBD | Hello World transpilation |
| `transpile_moderate` | TBD | TBD | Arrays + pointers (ownership inference) |
| `check_project` | TBD | TBD | Dependency analysis (3 files) |

## C-to-Rust Transpilation Performance Characteristics

### Expected Syscall Patterns

**Simple Transpilation**:
- File read operations (C source)
- LLVM/Clang library calls (AST parsing)
- File write operations (Rust output)
- Memory allocation for AST structures

**Moderate Transpilation**:
- Similar to simple but with more complex analysis
- Ownership inference for pointers
- Lifetime annotation generation
- Memory pattern detection (malloc/free → Box)

**Project-Level Analysis**:
- Directory traversal (walkdir)
- Multiple file reads (#include resolution)
- Dependency graph construction
- Topological sort for build order

### Anti-Pattern Detection

Renacer can detect:

1. **Tight Loop**:
   - Symptom: Excessive AST traversal iterations
   - Solution: Optimize visitor pattern or add caching

2. **God Process**:
   - Symptom: Single process doing too much
   - Solution: Parallelize file transpilation

## Next Steps

1. **Set performance baselines** using these golden traces
2. **Add assertions** in `renacer.toml` for automated checking
3. **Integrate with CI** to prevent regressions
4. **Compare transpilation** speed across different C complexity levels
5. **Monitor LLVM** library overhead

Generated: $(date)
Renacer Version: 0.6.2
decy Version: 1.0.2
EOF

# ==============================================================================
# Cleanup
# ==============================================================================
echo ""
echo -e "${YELLOW}Cleaning up test files...${NC}"
rm -f "$TRACES_DIR/test_hello.c" "$TRACES_DIR/test_hello.rs"
rm -f "$TRACES_DIR/test_arrays.c" "$TRACES_DIR/test_arrays.rs"
rm -rf "$TRACES_DIR/test_project"

# ==============================================================================
# Summary
# ==============================================================================
echo ""
echo -e "${BLUE}=== Golden Trace Capture Complete ===${NC}"
echo ""
echo "Traces saved to: $TRACES_DIR/"
echo ""
echo "Files generated:"
ls -lh "$TRACES_DIR"/*.json "$TRACES_DIR"/*.txt 2>/dev/null | awk '{print "  " $9 " (" $5 ")"}'
echo ""
echo -e "${GREEN}Next steps:${NC}"
echo "  1. Review traces: cat golden_traces/transpile_simple_summary.txt"
echo "  2. View JSON: jq . golden_traces/transpile_simple.json | less"
echo "  3. Run tests: cargo test --test golden_trace_validation"
echo "  4. Update baselines in ANALYSIS.md with actual metrics"
