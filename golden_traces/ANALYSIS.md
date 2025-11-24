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
| `transpile_simple` | 8.165ms | 584 | Hello World transpilation (C parse + HIR + codegen) |
| `transpile_moderate` | 7.850ms | 584 | Arrays + pointers + ownership inference |
| `check_project` | 2.902ms | 309 | Dependency analysis (2 C files: helper.c, main.c) |

**Key Insights:**
- **transpile_simple** (8.165ms): Sub-10ms transpilation. Dominated by futex (62.84%, 317 calls) for LLVM/Clang library synchronization. Only 584 total syscalls validates efficient C parser integration.
- **transpile_moderate** (7.850ms): Nearly identical to simple (584 syscalls) despite pointer analysis. Ownership inference happens in-memory with no additional I/O. Futex (65.75%) shows LLVM overhead dominates, not complexity.
- **check_project** (2.902ms): 2.8× faster than transpilation with only 309 syscalls. Balanced I/O: mmap (29.46%), openat (10.20%), read (9.82%). No LLVM overhead, pure dependency graph construction.

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

Generated: 2025-11-24
Renacer Version: 0.6.2
decy Version: 1.0.2
