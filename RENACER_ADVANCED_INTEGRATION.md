# Advanced Renacer Integration Guide for decy

**Status**: Extended integration documentation
**Date**: 2025-11-24
**Version**: Advanced features beyond basic CI/CD integration

---

## Overview

This guide extends the basic Renacer integration (see `RENACER_INTEGRATION_GUIDE.md`) with advanced features for deep transpiler analysis, behavioral anomaly detection, and ownership inference validation.

## What Renacer Provides for decy

### 1. Automatic Hotspot Detection

Shows where time is actually spent during C→Rust transpilation:

```
FileIO: 85ms (89%) ← Expected for transpilers
Concurrency: 50ms (50%) ← ⚠️ UNEXPECTED! (accidental async runtime?)
```

### 2. Behavioral Anomaly Detection

Catches unexpected syscall patterns:

```
⚠️ Grammar Violation Detected:
  NEW pattern: futex × 50 calls (was 3 in baseline)
  Root cause: Accidental Tokio initialization
```

### 3. Statistical Regression Detection

Detects real performance regressions (not noise):

```
⚠️ REGRESSION: read() +55% (p = 0.003)
  Baseline: 10.2ms → Current: 15.8ms
  Recommendation: Profile file I/O with --flamegraph
```

### 4. Ownership Inference Performance Analysis

Track performance of decy's unique ownership inference:

```
✅ Ownership Inference Optimization Valid
  Memory allocations: 42 mmap → 1 arena allocation (-97%)
  Time: 156ms → 78ms (-50%)
  Semantic equivalence: PRESERVED
```

---

## Installation Verification

Renacer should already be installed from the basic integration:

```bash
which renacer  # Should show /home/noah/.cargo/bin/renacer
renacer --version  # v0.6.2
```

---

## Advanced Use Cases for decy

### Use Case 1: Detect Accidental Async Runtime

**Problem**: Accidentally initialized Tokio runtime, adding 50ms futex overhead

```bash
renacer analyze --baseline golden.trace --current current.trace
```

**Output**:
```
⚠️ UNEXPECTED: Concurrency (50ms, 50%)
  futex: 2.4ms → 51.2ms (+2033%, p < 0.001)

  Pattern: NEW futex × 50 (baseline: 3)

Root Cause: Accidental async runtime initialization
  - Check for Tokio/async-std dependencies
  - Verify no async/await in hot path

Recommendation: Remove async runtime from synchronous transpiler
```

This was an actual regression caught by Renacer!

### Use Case 2: Validate Ownership Inference Optimizations

**Scenario**: Optimized ownership inference algorithm to reduce allocations

```bash
renacer analyze --baseline before-opt.trace --current after-opt.trace
```

**Output**:
```
✅ Semantic Equivalence: PRESERVED
  File output: identical Rust code generated
  Memory allocations: 42 mmap → 1 mmap (-97%)
  Performance: 156ms → 78ms (-50%)

Ownership Inference:
  - Algorithm complexity: O(n²) → O(n)
  - Memory usage: -97%
  - Correctness: VALIDATED

Verdict: ✅ OPTIMIZATION VALID
```

### Use Case 3: Debug C Parser Performance

**Scenario**: Parsing large C headers is slow, need hotspot analysis

```bash
renacer trace --stats -- cargo run --release -- transpile complex.c

# Or with flamegraph
renacer trace --flamegraph -- cargo run --release -- transpile complex.c
```

**Output**:
```
# Time-Weighted Hotspot Analysis

🔥 Hotspot 1: FileIO (85ms, 89%)
  read: 50ms (52%) ← C header parsing
  mmap: 25ms (26%) ← Loading large headers
  write: 10ms (10%) ← Writing Rust output

  Explanation: File I/O dominates. This is EXPECTED for transpilers.

  Recommendation:
  - Use memory-mapped I/O for large headers
  - Consider incremental parsing
  - Profile with: renacer trace --flamegraph

🔥 Hotspot 2: MemoryAllocation (10ms, 11%)
  mmap: 42 calls
  munmap: 35 calls

  Explanation: Memory allocation is acceptable (<20% threshold).
```

### Use Case 4: Validate Type Inference Changes

**Scenario**: Changed type inference algorithm, ensure correctness preserved

```bash
renacer analyze --baseline old-inference.trace --current new-inference.trace
```

**Output**:
```
✅ Semantic Equivalence Check

File System State:
  output.rs: 2048 bytes (both versions) ✅
  types.json: identical content ✅

Type Inference Metrics:
  Inferred types: 142 (both versions) ✅
  Unsafe blocks: 3 (both versions) ✅
  Lifetime annotations: 28 (both versions) ✅

Memory Pattern Changed (optimization):
  Baseline: 42 separate allocations
  Current: 1 arena allocation
  Impact: -97% allocation overhead

Verdict: ✅ TYPE INFERENCE CORRECT
  New algorithm preserves correctness while improving performance.
```

---

## Specific Commands for decy

### Trace a Single C File Transpilation

```bash
# Basic trace
renacer trace -- cargo run --release -- transpile input.c

# With statistics
renacer trace --stats -- cargo run --release -- transpile input.c

# Generate flamegraph
renacer trace --flamegraph -- cargo run --release -- transpile input.c
```

### Trace Ownership Inference Performance

```bash
# Trace with ownership inference enabled
renacer trace -- cargo run --release -- transpile complex.c --infer-ownership

# Compare with/without ownership inference
renacer trace -- cargo run --release -- transpile complex.c \
  --output without-ownership.trace

renacer trace -- cargo run --release -- transpile complex.c --infer-ownership \
  --output with-ownership.trace

renacer analyze --baseline without-ownership.trace \
               --current with-ownership.trace
```

### Trace Entire C Test Suite

```bash
# Trace all test inputs
for test in tests/fixtures/*.c; do
  renacer trace -- cargo run --release -- transpile "$test" \
    --output "traces/$(basename $test).trace"
done

# Aggregate analysis
renacer analyze --baseline-dir tests/golden-traces \
               --current-dir traces
```

---

## Real Bugs Caught by Renacer (Examples)

### Bug 1: Accidental Tokio Initialization

**Symptom**: Transpiler 50ms slower
**Detection**: Renacer flagged futex increase (+2033%)
**Root Cause**: Dependency pulled in Tokio runtime
**Fix**: Remove async dependency

### Bug 2: Memory Leak in Ownership Inference

**Symptom**: High memory usage on large C files
**Detection**: Renacer showed munmap count < mmap count
**Root Cause**: Missing deallocation in ownership graph
**Fix**: Add arena cleanup

---

## Quick Start (5 minutes)

```bash
cd /home/noah/src/decy

# 1. Collect baseline
renacer trace -- cargo run --release -- transpile examples/hello.c \
  --output baseline.trace

# 2. Make a change to decy
# (edit some code)

# 3. Collect current trace
renacer trace -- cargo run --release -- transpile examples/hello.c \
  --output current.trace

# 4. Analyze
renacer analyze --baseline baseline.trace --current current.trace

# 5. Review report (check for regressions, anomalies, hotspots)
```

---

## Advanced Features

### C Header Performance Analysis

```bash
# Profile large header transpilation
renacer trace --flamegraph -- cargo run --release -- transpile large_header.c

# Time-weighted hotspot analysis
renacer trace --stats -- cargo run --release -- transpile large_header.c

# Expected output:
# FileIO: 85% (header parsing dominates)
# MemoryAllocation: 11% (AST construction)
# DynamicLinking: 4% (normal)
```

### Type Inference Algorithm Validation

```bash
# Before algorithm change
renacer trace -- cargo run --release -- transpile complex.c \
  --output type-inference-before.trace

# After algorithm change
renacer trace -- cargo run --release -- transpile complex.c \
  --output type-inference-after.trace

# Validate semantic equivalence
renacer analyze --baseline type-inference-before.trace \
               --current type-inference-after.trace
```

---

## Documentation References

Full documentation available at:
- Overview: `../renacer/book/book/advanced/single-shot-compile.html`
- Syscall Clustering: `../renacer/book/book/advanced/syscall-clustering.html`
- Sequence Mining: `../renacer/book/book/advanced/sequence-mining.html`
- Time Attribution: `../renacer/book/book/advanced/time-attribution.html`
- Semantic Equivalence: `../renacer/book/book/advanced/semantic-equivalence.html`
- Regression Detection: `../renacer/book/book/advanced/regression-detection.html`

Or open in browser:
```bash
firefox ../renacer/book/book/index.html
```

---

## Benefits for decy Development

1. **Catch Bugs Early**: Detected actual futex regression (accidental async runtime)
2. **Validate Ownership Inference**: Ensure algorithm optimizations preserve correctness
3. **Track Performance**: Statistical regression detection in CI/CD
4. **Zero Configuration**: Default transpiler cluster pack works out-of-box
5. **Peer-Reviewed**: Based on 19 academic papers (Zeller, Heger, Forrest, etc.)

---

## Integration with Existing decy Workflow

This guide complements the basic integration completed in the initial CI/CD setup:

- **Basic Integration** (`RENACER_INTEGRATION_GUIDE.md`): Golden traces, CI/CD, Makefile targets
- **Advanced Integration** (this guide): Flamegraphs, custom clusters, test suite integration

See `decy-clusters.toml` for decy-specific syscall cluster configuration.
