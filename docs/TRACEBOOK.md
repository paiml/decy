# Decy Tracebook: Behavioral Equivalence Verification

## Overview

The Decy Tracebook is a multi-layer verification system that proves transpiled Rust code is **behaviorally equivalent** to the original C. It combines four complementary trace systems into a unified workflow:

```
                         TRACEBOOK ARCHITECTURE
    ===================================================================

    Decision Trace (DECY-193)          Golden Trace (DECY-107)
    -------------------------          -------------------------
    Transpilation decisions            C->Rust training pairs
    5 pipeline stages                  3 complexity tiers (P0-P2)
    6 decision types                   9 transformation types
    Confidence scoring                 ML export (JSONL/ChatML/Alpaca)

                          | Quality Gate |

              Trace Verifier (Poka-Yoke)
              --------------------------
              3 verification levels
              Compilation check
              Unsafe block counting
              Clippy linting

                          | Export |

    Renacer Performance Baselines      Corpus Validation (C99/K&R)
    -----------------------------      ----------------------------
    Syscall-level tracing              Per-chapter convergence rates
    JSON trace format                  Semantic equivalence (gcc vs rustc)
    20% regression detection           Deterministic output verification
```

## Philosophy

Following Toyota Way principles:

- **Jidoka (stop on defect)**: Fail immediately when behavioral divergence is detected
- **Genchi Genbutsu (go and see)**: Observe actual execution, not just static analysis
- **Determinism**: Same C input -> same Rust output -> same runtime behavior
- **Zero Defects**: Catch regressions before they reach production

## Quick Start

```bash
# Capture a golden trace (C file -> transpile -> run both -> compare)
make golden-capture TRACE=hello CMD=validation/kr-c/chapter-1/01_hello_world.c

# Compare against existing golden baseline
make golden-compare TRACE=hello CMD=validation/kr-c/chapter-1/01_hello_world.c

# List all captured traces
make golden-list

# Full CI validation (convergence + equivalence + diff-test)
make golden-ci

# Show usage guide
make golden-help
```

## Trace Layers

### Layer 1: Decision Traces (Transpiler Introspection)

Decision traces record **why** the transpiler made each choice during transpilation. This is the internal flight recorder.

**Location**: `crates/decy-core/src/trace.rs`

**Usage**:
```bash
# Emit decision trace as JSON to stderr
decy transpile input.c --trace

# Example output:
# {"stage":"OwnershipInference","decision_type":"PointerClassification",
#  "description":"Classified ptr as owning (malloc pattern detected)",
#  "alternatives":["borrowing","raw_pointer"],"confidence":0.95}
```

**Pipeline Stages**: Parsing, HirConversion, OwnershipInference, LifetimeAnalysis, CodeGeneration

**Decision Types**: PointerClassification, TypeMapping, SafetyTransformation, LifetimeAnnotation, PatternDetection, SignatureTransformation

### Layer 2: Golden Traces (ML Training Dataset)

Golden traces are curated C-to-Rust training pairs with rich metadata for ML model training.

**Location**: `crates/decy-oracle/src/golden_trace.rs`

**Usage**:
```bash
# Generate traces from validation corpus
decy oracle generate-traces

# Export as JSONL for fine-tuning
decy oracle export --format jsonl

# Export as ChatML for instruction tuning
decy oracle export --format chatml
```

**Tiers**:
- **P0 (Simple)**: Basic constructs (variables, arithmetic, control flow)
- **P1 (I/O)**: Programs with stdio (printf/scanf -> println!/read_line)
- **P2 (Complex)**: Ownership patterns (malloc/free -> Box, arrays -> Vec)

**Safety Transformations**: MallocToBox, CallocToVec, NullToOption, PointerToReference, PthreadToMutex, OutputParamToResult, TaggedUnionToEnum, ArrayParamToSlice

### Layer 3: Trace Verifier (Poka-Yoke Quality Gate)

The trace verifier prevents defective traces from entering the training dataset. Named after the Toyota mistake-proofing methodology.

**Location**: `crates/decy-oracle/src/trace_verifier.rs`

**Verification Levels**:
- **Minimal**: Compilation check only
- **Standard**: Compilation + unsafe block counting
- **Strict**: Compilation + unsafe + clippy linting

### Layer 4: Behavioral Equivalence (S5 Differential Testing)

The S5 layer compiles original C with gcc and transpiled Rust with rustc, runs both, and compares stdout + exit codes.

**Location**: `crates/decy-verify/src/diff_test.rs`

**Usage**:
```bash
# Differential test a single file
decy diff-test input.c

# With custom timeout
decy diff-test input.c --timeout 10
```

**Flow**: Write scratch files -> compile with gcc/rustc -> run both -> compare stdout + exit codes -> report divergences.

## Golden Trace Workflow

### Capture

```bash
# Capture behavioral equivalence trace for a C file
make golden-capture TRACE=struct_basics CMD=validation/kr-c/chapter-6/01_struct_basics.c

# What happens:
# 1. Compile C with gcc, run, capture stdout + exit code
# 2. Transpile C with decy
# 3. Compile Rust with rustc, run, capture stdout + exit code
# 4. Compare outputs
# 5. Capture renacer syscall traces for both
# 6. Store everything in golden_traces/<name>/
```

### Compare

```bash
# After making code changes, verify behavior hasn't changed
make golden-compare TRACE=struct_basics CMD=validation/kr-c/chapter-6/01_struct_basics.c

# Reports:
# - stdout match: PASS/FAIL
# - Exit code match: PASS/FAIL
# - Syscall pattern changes: +/- summary
```

### List

```bash
make golden-list

# Output:
# === Golden Traces ===
#   hello_world          stdout:PASS  exit:PASS  captured:2026-02-14
#   struct_basics        stdout:PASS  exit:PASS  captured:2026-02-14
#   array_indexing       stdout:PASS  exit:PASS  captured:2026-02-14
#
# Total: 3 traces
```

### Clean

```bash
# Remove all captured traces
make golden-clean
```

### CI Validation

```bash
# Full CI workflow: convergence + equivalence + diff-test
make golden-ci

# This runs:
# 1. Corpus convergence measurement (transpile + compile rates)
# 2. Semantic equivalence validation (gcc vs rustc output comparison)
# 3. Determinism tests (identical output verification)
```

## Renacer Integration (Performance Baselines)

Renacer captures syscall-level execution traces for performance regression detection.

### Capture Performance Baseline

```bash
# Install renacer
make renacer-install

# Capture baselines for key operations
make renacer-capture
```

### Validate Performance

```bash
# Check for >20% regression against baselines
make renacer-validate
```

### Performance Budget

| Operation | Baseline | Budget (+20%) |
|-----------|----------|---------------|
| transpile_simple | 8.165ms | 9.798ms |
| transpile_moderate | 7.850ms | 9.420ms |
| check_project | 2.902ms | 3.482ms |

Exceeding the budget triggers a CI failure (Jidoka).

## Corpus Validation

### Convergence Measurement

Tracks transpilation success rates across the K&R C validation corpus.

```bash
make convergence

# Output: Per-chapter transpile/compile rates
# | Chapter | Files | Transpile OK | Compile OK | Rate |
# |---------|-------|-------------|-----------|------|
# | ch-1    | 20    | 20          | 18        | 90%  |
# | ch-2    | 15    | 15          | 14        | 93%  |
```

### Semantic Equivalence

Validates that transpiled programs produce identical output to the original C.

```bash
make validate-equivalence

# Output:
# | Metric | Value |
# | Total Files | 45 |
# | Equivalent  | 42 |
# | Divergent   | 3  |
# | Equivalence Rate | 93.3% |
```

### Determinism

Verifies that transpilation is deterministic (same input -> identical output).

```bash
make determinism
```

## Integration with EXTREME TDD

Golden traces integrate naturally with the RED-GREEN-REFACTOR workflow:

### RED Phase
```bash
# Write a failing C test case
echo 'int main() { return 42; }' > test_exit.c

# Capture expected behavior
make golden-capture TRACE=test_exit CMD=test_exit.c
# FAIL: Transpiler can't handle this case yet
```

### GREEN Phase
```bash
# Implement the feature
# ...

# Verify behavioral equivalence
make golden-compare TRACE=test_exit CMD=test_exit.c
# PASS: C and Rust produce identical output
```

### REFACTOR Phase
```bash
# Refactor implementation
# ...

# Verify golden trace still matches (no regression)
make golden-compare TRACE=test_exit CMD=test_exit.c
# PASS: Behavior preserved after refactoring
```

## Trace Storage

```
golden_traces/
  transpile_simple.json          # Renacer syscall trace (performance)
  transpile_simple_summary.txt   # Syscall summary statistics
  transpile_moderate.json        # Moderate complexity trace
  check_project.json             # Project-level operation trace
  ANALYSIS.md                    # Trace interpretation guide
  traces/                        # Behavioral equivalence traces
    hello_world/
      c_stdout.txt               # Original C program output
      c_exit_code.txt            # C program exit code
      rust_stdout.txt            # Transpiled Rust output
      rust_exit_code.txt         # Rust program exit code
      metadata.json              # Capture timestamp, file path, result
    struct_basics/
      ...
```

## Troubleshooting

### Timing Differences in Renacer Traces

**Problem**: Syscall timings vary between runs.

**Solution**: Focus on syscall names and counts, not timing. Use `jq` to extract patterns:
```bash
cat golden_traces/transpile_simple.json | jq '.syscalls[] | .name' | sort | uniq -c
```

### printf vs println! Format Differences

**Problem**: `printf("%f", 3.14)` produces `3.140000` but `println!("{}", 3.14)` produces `3.14`.

**Solution**: S5 focuses on integer programs for V1. Float comparison is a known limitation. Use integer-only test cases or implement output normalization for float values.

### Missing gcc or rustc

**Problem**: `golden_trace.sh` fails with compiler not found.

**Solution**:
```bash
# Verify compilers
gcc --version
rustc --version

# Install if missing
sudo apt install gcc    # Debian/Ubuntu
rustup update stable    # Rust
```

### Transpilation Failure

**Problem**: Golden trace capture fails because decy can't transpile the C file.

**Solution**: This is expected for unsupported constructs. Check the convergence report:
```bash
make convergence
```
Only files that transpile successfully can have behavioral equivalence traces.

## Makefile Targets Reference

| Target | Description | Example |
|--------|-------------|---------|
| `golden-help` | Show usage guide | `make golden-help` |
| `golden-capture` | Capture behavioral trace | `make golden-capture TRACE=name CMD=file.c` |
| `golden-compare` | Compare against baseline | `make golden-compare TRACE=name CMD=file.c` |
| `golden-list` | List all captured traces | `make golden-list` |
| `golden-clean` | Remove all traces | `make golden-clean` |
| `golden-ci` | Full CI validation | `make golden-ci` |
| `convergence` | Corpus convergence rates | `make convergence` |
| `validate-equivalence` | Semantic equivalence | `make validate-equivalence` |
| `determinism` | Deterministic output | `make determinism` |
| `renacer-capture` | Performance baselines | `make renacer-capture` |
| `renacer-validate` | Performance regression | `make renacer-validate` |

## Architecture

```
decy/
  crates/
    decy-core/src/trace.rs              # Decision trace collector (Layer 1)
    decy-oracle/src/golden_trace.rs     # Golden trace dataset (Layer 2)
    decy-oracle/src/trace_verifier.rs   # Poka-yoke quality gate (Layer 3)
    decy-verify/src/diff_test.rs        # S5 differential testing (Layer 4)
  scripts/
    golden_trace.sh                     # Multi-command trace workflow
    capture_golden_traces.sh            # Renacer performance baselines
    convergence.sh                      # Corpus convergence measurement
    validate-equivalence.sh             # Semantic equivalence validation
  golden_traces/                        # Renacer baselines + behavioral traces
  docs/
    TRACEBOOK.md                        # This document
```

## Resources

- [Renacer](https://github.com/paiml/renacer) - Pure Rust syscall tracer
- [EXTREME TDD Guidelines](../CLAUDE.md) - Development methodology
- [C Validation Roadmap](../docs/C-VALIDATION-ROADMAP.yaml) - C99/K&R north star
- [Toyota Way Principles](https://en.wikipedia.org/wiki/The_Toyota_Way)

---

**Status**: Production Ready
**Maturity**: Stable (4 layers operational)
**Testing**: Comprehensive (unit + CLI contract + integration)
**Documentation**: Complete
