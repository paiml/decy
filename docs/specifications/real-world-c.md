# Real-World C Transpilation Specification

**Status**: Active (S1/S2 Implemented)
**Created**: 2026-02-10
**Updated**: 2026-02-10
**Version**: 1.1

## Problem Statement

Decy transpiles toy C functions but fails on real-world code. Three critical gaps:

1. **No output verification**: Generated Rust is never compiled — silent type errors, missing imports, and broken lifetimes ship undetected.
2. **Panics on legal C**: `for(;;)` (infinite loop) panics the transpiler despite being idiomatic C99 (§6.8.5.3).
3. **Unusable for real codebases**: Missing `unsafe` escape hatch, no incremental adoption path, no feedback loop.

This specification defines five strategies to close these gaps, grounded in Popperian falsification (concrete testable predictions), Toyota Way principles (Jidoka, Genchi Genbutsu, Kaizen), and peer-reviewed transpilation research.

## Strategy 1: Compile-the-Output Verification

### Motivation

A transpiler that does not compile its own output is a transpiler that lies. Every generated Rust file must pass `rustc` type checking before being presented to the user.

**Toyota Way — Jidoka (自働化)**: Build quality in at the source. Detect defects at the point of creation, not downstream. The transpiler must stop the line when it produces uncompilable output.

### Approach

Add `verify_compilation()` to `decy-verify` that:

1. Writes generated Rust to a temporary file
2. Invokes `rustc --emit=metadata --edition=2021` (type-check only, no codegen)
3. Parses structured stderr for error codes and spans
4. Returns `CompilationResult { success, errors, warnings }`

Wire into CLI via `--verify` flag on the `transpile` subcommand.

### Falsifiable Predictions

| ID | Prediction | Test |
|----|-----------|------|
| S1-P1 | `verify_compilation("fn main() {}")` returns `success: true` | Unit test |
| S1-P2 | `verify_compilation("fn main() { let x: i32 = \"bad\"; }")` returns error E0308 | Unit test |
| S1-P3 | `decy transpile --verify valid.c` exits 0 with "Compilation verified" on stderr | Integration test |
| S1-P4 | `decy transpile --verify invalid.c` exits non-zero with structured error report | Integration test |

### References

- Rigger & Su, "Testing Database Engines via Pivoted Query Synthesis", OOPSLA 2020 — validates outputs against oracle (analogous: rustc as oracle)
- Csmith (Yang et al., "Finding and Understanding Bugs in C Compilers", PLDI 2011) — compiler output validation methodology
- Le et al., "Compiler Validation via Equivalence Modulo Inputs", PLDI 2014 — differential testing of compiler outputs

## Strategy 2: Unsafe-First Codegen + for(;;) Fix

### Motivation

Real C code uses `for(;;)` for event loops, polling, and state machines. Panicking on legal C99 is a correctness bug. Additionally, the current codegen attempts safe Rust first and fails silently on complex patterns — an unsafe-first approach with progressive refinement produces correct code sooner.

**Toyota Way — Genchi Genbutsu (現地現物)**: Go and see. Real C codebases use `for(;;)` extensively (Linux kernel: 15,000+ instances, Redis: 200+ instances). The transpiler must handle what real code actually does.

### Approach

#### B1: Make For condition optional in HIR

Change `HirStatement::For { condition: HirExpression }` to `condition: Option<HirExpression>`. This correctly models C99 §6.8.5.3: "the controlling expression is omitted, it is replaced by a nonzero constant" — i.e., an omitted condition means infinite loop.

#### B2: Emit `loop {}` when condition is None

In codegen, branch on `condition`:
- `Some(cond)` → current behavior: `while cond { body; increment; }`
- `None` → `loop { body; increment; }` (idiomatic Rust infinite loop)

#### B3-B6: Propagate Option through pipeline

Update all pattern matches in core, optimizer, ownership, analyzer, and test files.

### Falsifiable Predictions

| ID | Prediction | Test |
|----|-----------|------|
| S2-P1 | `for(;;) { break; }` transpiles to `loop { break; }` without panic | Unit test |
| S2-P2 | `for(int i=0; i<10; i++)` still transpiles to `while i < 10 { ... }` | Regression test |
| S2-P3 | `for(;;) { if(done) break; process(); }` produces compilable Rust | Compile verification test |
| S2-P4 | No existing test regresses (all 500+ tests pass) | `cargo test --workspace` |

### References

- ISO/IEC 9899:1999 (C99) §6.8.5.3 — "If the clause-1 expression is omitted, the controlling expression is replaced by a nonzero constant"
- Kernighan & Ritchie, "The C Programming Language", 2nd Ed., §3.5 — `for(;;)` idiom
- Emre et al., "Translating C to Safer Rust", OOPSLA 2021 — handling C control flow in transpilation

## Strategy 3: Progressive Unsafe Refinement

### Motivation

Current approach: attempt safe Rust, fail on complex patterns, produce broken output. Better approach: emit correct `unsafe` Rust first, then progressively refine to safe patterns using ownership inference.

**Toyota Way — Kaizen (改善)**: Continuous improvement. Start with working (unsafe) code, iteratively reduce unsafe blocks. Each iteration must preserve correctness (verified by Strategy 1).

### Approach

Four-phase refinement pipeline. Each phase produces compilable Rust (verified by S1).

**Phase 1: Stdlib Function Mapping** (Done)

Two categories of stdlib mappings based on type safety:

**Category A: Inline expansion** (type-safe, no pointer operands):

| C Function | Rust Equivalent | Status |
|------------|----------------|--------|
| `malloc(n)` | `Box::new()` / `Vec::with_capacity()` | Done (pre-existing) |
| `free(ptr)` | Drop (RAII) | Done (pre-existing) |
| `printf(fmt, ...)` | `println!()` / `print!()` | Done (pre-existing) |
| `strlen(s)` | `s.len()` | Done (pre-existing) |
| `strcpy(dst, src)` | `dst.clone_from(src)` | Done (pre-existing) |
| `atoi(s)` | `s.parse::<i32>().unwrap_or(0)` | Done |
| `atof(s)` | `s.parse::<f64>().unwrap_or(0.0)` | Done |
| `abs(x)` | `(x).abs()` | Done |
| `exit(code)` | `std::process::exit(code)` | Done |
| `puts(s)` | `println!("{}", s)` | Done |
| `snprintf(buf, n, fmt, ...)` | `format!(fmt, args)` | Done |
| `sprintf(buf, fmt, ...)` | `format!(fmt, args)` | Done |
| `qsort(base, n, sz, cmp)` | `base[..n].sort_by(\|a, b\| cmp(a, b))` | Done |

**Category B: Stub mechanism** (pointer-based, can't inline safely):

| C Function | Approach | Rationale |
|------------|----------|-----------|
| `memcpy(dst, src, n)` | Generated stub | Operands are raw pointers (`*mut u8`) in transpiled code |
| `memset(ptr, val, n)` | Generated stub | Operands are raw pointers |
| `strcmp(a, b)` | Generated stub | Arguments may be `*mut u8` or `&str` — type mismatch |
| `strncmp(a, b, n)` | Generated stub | Same as strcmp |
| `strcat(dst, src)` | Generated stub | Pointer-based mutation |

**Key insight**: Inline expansion only works for functions where transpiled argument types match Rust method receivers. Pointer-based C functions (memcpy, memset, strcmp, strncmp, strcat) use raw pointer types (`*mut u8`) in transpiled code, making safe Rust methods like `.copy_from_slice()` or `.cmp()` inapplicable. The stub mechanism generates type-compatible function signatures that bridge the gap.

**Phase 2: Unsafe Fallback Codegen**

For patterns not yet supported by ownership inference, emit correct `unsafe` blocks:

```rust
// C: *ptr = value;
// Phase 2 output (correct but unsafe):
unsafe { *ptr = value; }

// Phase 3 output (after ownership inference):
*ptr = value;  // if ptr: &mut T
```

**Phase 3: Ownership-Driven Refinement**

Apply `decy-ownership` inference to replace unsafe patterns with safe equivalents. Already implemented for malloc/free → Box/Vec, pointer parameters → &T/&mut T.

**Phase 4: Compilation Verification**

Verify output compiles at every intermediate stage using S1 (`verify_compilation`).

### Falsifiable Predictions

| ID | Prediction | Test |
|----|-----------|------|
| S3-P1 | `malloc/free` pair emits `unsafe { Box::new() }` before ownership inference | Unit test |
| S3-P2 | After ownership inference, same code emits `Box::new()` without unsafe | Unit test |
| S3-P3 | Unsafe block count monotonically decreases across refinement passes | Property test |
| S3-P4 | Output compiles at every intermediate stage | Compile verification |

### References

- Astrauskas et al., "Leveraging Rust Types for Modular Specification and Verification", OOPSLA 2019 — type-driven safety verification
- Evans et al., "Is Rust Used Safely by Software Developers?", ICSE 2020 — unsafe Rust usage patterns in practice
- Emre et al., "Translating C to Safer Rust", OOPSLA 2021 — progressive unsafe elimination

## Strategy 4: Mutation-Guided Transpilation Testing

### Motivation

Traditional testing verifies the happy path. Mutation testing systematically injects faults to verify that tests actually detect bugs. For a transpiler, mutations in the codegen reveal which output patterns are under-tested.

### Approach (Future)

1. Define mutation operators for transpiler output (swap `&T`/`&mut T`, remove lifetime annotations, change `Box` to raw pointer)
2. Apply mutations to generated Rust
3. Verify that at least one test detects each mutation
4. Mutations that survive indicate gaps in test coverage

### Falsifiable Predictions

| ID | Prediction | Test |
|----|-----------|------|
| S4-P1 | Swapping `&T` → `&mut T` in output causes at least one test failure | Mutation test |
| S4-P2 | Removing lifetime annotations from output causes compilation failure | Mutation test |
| S4-P3 | Mutation kill rate ≥85% across codegen output | Aggregate metric |

### References

- Jia & Harman, "An Analysis and Survey of the Development of Mutation Testing", IEEE TSE 2011 — mutation testing foundations
- Papadakis et al., "Mutation Testing Advances: An Analysis and Survey", Advances in Computers 2019 — state of the art
- Just et al., "Are Mutants a Valid Substitute for Real Faults in Software Testing?", FSE 2014 — mutation validity

## Strategy 5: Differential Testing Against GCC/Clang Semantics

### Motivation

The transpiler must preserve C program semantics. Differential testing compiles original C and transpiled Rust, runs both on the same inputs, and compares outputs.

### Approach (Future)

1. Compile C source with GCC/Clang
2. Transpile C to Rust, compile with rustc
3. Generate random inputs (guided by function signatures)
4. Execute both binaries on same inputs
5. Compare stdout, exit codes, and side effects

### Falsifiable Predictions

| ID | Prediction | Test |
|----|-----------|------|
| S5-P1 | `int add(int a, int b) { return a+b; }` produces identical results for 10,000 random inputs | Differential test |
| S5-P2 | Array indexing produces identical results (within bounds) | Differential test |
| S5-P3 | String operations produce identical results for ASCII inputs | Differential test |

### References

- Yang et al., "Finding and Understanding Bugs in C Compilers", PLDI 2011 — Csmith differential testing
- Le et al., "Compiler Validation via Equivalence Modulo Inputs", PLDI 2014 — EMI testing
- Chen et al., "An Empirical Comparison of Compiler Testing Techniques", ICSE 2016 — testing technique comparison

## Implementation Priority

| Strategy | Priority | Status | Rationale |
|----------|----------|--------|-----------|
| S2: for(;;) fix | P0 | **Done** | Correctness bug — panics on legal C99 |
| S1: Compile verification | P1 | **Done** | Foundation for all other strategies |
| S3: Progressive unsafe | P2 | **In Progress** | Stdlib mapping + unsafe fallback |
| S4: Mutation testing | P3 | Future | Requires S1 + S3 |
| S5: Differential testing | P3 | Future | Requires S1 + working codegen |

## Implementation Results

### S2: for(;;) Fix — Completed

- **HIR**: `HirStatement::For { condition }` changed from `HirExpression` to `Option<HirExpression>`
- **Codegen**: `None` condition emits `loop {}`, `Some(cond)` emits `while cond { ... }`
- **Pipeline**: Updated 8 files across core, optimizer, ownership, analyzer, and codegen
- **Tests**: 40+ test files updated (`condition: Some(...)` wrapper), 11 previously-falsified tests un-falsified
- **Prediction S2-P1**: Verified — `for(;;) { break; }` transpiles to `loop { break; }`
- **Prediction S2-P4**: Verified — all workspace tests pass

### S1: Compile Verification — Completed

- **API**: `decy_verify::verify_compilation(rust_code: &str) -> Result<CompilationResult>`
- **CLI**: `decy transpile --verify <file>` flag wired into transpile subcommand
- **Prediction S1-P1**: Verified — `verify_compilation("fn main() {}")` returns success
- **Prediction S1-P2**: Verified — type mismatch returns error E0308

### Coverage Results

Post-implementation workspace coverage: **96.70% line coverage** (target: 95%)

| Crate | Coverage | Notes |
|-------|----------|-------|
| decy-parser | 91.99% | FFI/clang-sys boundary code |
| decy-hir | 97.50% | Well-covered via falsification + unit tests |
| decy-analyzer | 97.00% | Lock analysis, subprocess analysis at 90-96% |
| decy-ownership | 98.90% | 17 inference branch tests added via graph helpers |
| decy-codegen | 94.13% | 91 deep coverage tests, 172 total codegen tests |
| decy-verify | 99.23% | Compile verification fully tested |
| decy-core | 94.24% | Pipeline orchestration tests |
| decy-stdlib | 100.00% | Complete coverage |

### Test Corpus

- **Total tests**: 11,839 passing across workspace
- **Falsification tests**: 2,150 total (92 falsified, 95.7% pass rate)
- **Codegen deep tests**: 172 (targeting uncovered statement/expression/helper paths)
- **Inference branch tests**: 17 (via DataflowGraph test helpers for defensive branches)

### Falsification Analysis (Popperian Methodology)

92 falsified tests categorized by root cause:

| Category | Count | % | Priority | Fix Strategy |
|----------|-------|---|----------|-------------|
| C++ features (out of scope) | 21 | 22.8% | N/A | Mark out-of-scope |
| Stdlib functions missing | 21 | 22.8% | P1 | S3 Phase 1: stdlib mapping |
| Preprocessor/macro expressions | 16 | 17.4% | P1 | Constant folding in HIR |
| C11/C99 advanced features | 10 | 10.9% | P2 | VLA, designated initializers |
| Platform/system features | 7 | 7.6% | P3 | setjmp, signals, TLS |
| Control flow (goto) | 5 | 5.4% | P2 | goto → labeled blocks |
| Pointer/type system | 5 | 5.4% | P2 | Triple pointer, flexible arrays |
| Static analysis gaps | 3 | 3.3% | P3 | Double-free, UAF detection |
| GCC extensions | 3 | 3.3% | P4 | Packed structs, nested functions |
| Test input errors | 1 | 1.1% | P0 | Fix invalid C in tests |

**Key insight**: Fixing stdlib mapping (21) + constant folding (16) + removing C++ (21) reduces falsifications from 92 to 34 (63% reduction).

## Quality Gates

All changes in this specification must pass:

- `cargo build --workspace` — clean compile
- `cargo clippy --workspace -- -D warnings` — zero warnings
- `cargo test --workspace` — all tests pass (1700+)
- Line coverage >= 95% (`cargo llvm-cov --workspace`)
- No regressions in existing falsification tests
- New tests for every falsifiable prediction marked as implemented
