# Improve Decy: Cross-Project Analysis & Improvement Specification

**Status**: Implemented (8/10 items complete)
**Date**: 2026-02-09
**Sources**: depyler (Python-to-Rust), bashrs (Shell-to-Rust), claudes-c-compiler (C compiler)

## Motivation

Decy sits in a family of transpiler projects (depyler, bashrs) and shares the C-parsing problem space with claudes-c-compiler (CCC). Each sibling project has evolved distinct strengths that decy lacks. This specification proposes 10 concrete improvements drawn from cross-project analysis.

---

## 1. Hermetic Compilation Cache (from depyler)

**Gap**: Decy re-transpiles every file from scratch on every invocation. For large codebases this is prohibitively slow.

**What depyler does**: Content-addressable storage (CAS) using SQLite with hermetic cache keys derived from SHA-256 of the transpiler binary itself (not version strings). Invalidation happens automatically when the transpiler changes. See `depyler/crates/depyler/src/converge/cache.rs`.

**Proposal**:
- Add `decy-cache` module to `decy-core`
- Cache key = `SHA256(decy_binary) + SHA256(c_source) + transpiler_flags`
- Store transpiled Rust output + ownership decisions + unsafe count
- Backend: SQLite via `rusqlite` (same as depyler)
- Enable incremental repo-wide transpilation in `decy-repo`

**Impact**: 10-50x faster iteration on large codebases. Enables corpus convergence testing (Idea #2) to run in minutes instead of hours.

**Reference**: Venti (FAST 2002), Nix input-addressed stores (LISA 2004)

---

## 2. Corpus Convergence Loop (from depyler)

**Gap**: Decy validates individual constructs but has no systematic measure of "what percentage of real C code transpiles successfully on first pass."

**What depyler does**: A dedicated `depyler-corpus` crate with tiered corpora. Primary KPI is **single-shot compile rate** (percentage of source files whose transpiled output compiles with `rustc` without manual fixes). Depyler tracks: Tier 1 stdlib (92.7%), Tier 2 typed-cli (62.5%), Tier 5 algorithms (47.5%).

**Proposal**:
- Create `tests/corpus/` with tiered C programs:
  - **Tier 1**: Minimal C (variables, arithmetic, control flow) - target 95%+
  - **Tier 2**: Standard library usage (stdio, stdlib, string.h) - target 80%+
  - **Tier 3**: Pointer-heavy code (linked lists, trees) - target 60%+
  - **Tier 4**: Adversarial edge cases (macro-heavy, `void*` casts) - target 30%+
  - **Tier 5**: Real-world extracts (SQLite fragments, Redis snippets) - target 20%+
- KPIs: compile rate, unsafe block count per 1K LOC, borrow checker pass rate
- Track convergence over time: each sprint should monotonically improve compile rate

**Impact**: Transforms quality from anecdotal ("it handles structs") to measurable ("Tier 2 compile rate is 73%, up from 61% last sprint").

---

## 3. From-Scratch Parser Option (from CCC)

**Gap**: Decy depends on `clang-sys` (LLVM/Clang FFI) for parsing. This creates a heavy installation requirement (LLVM 14), limits portability, and is the only source of `unsafe` FFI code in the project.

**What CCC does**: Implements a complete C preprocessor, lexer, and recursive-descent parser from scratch in pure safe Rust (zero dependencies). This parser is battle-tested against PostgreSQL, FFmpeg, the Linux kernel, and 150+ real-world projects.

**Proposal**:
- Add a `decy-parser-pure` crate implementing a from-scratch C99 parser in safe Rust
- Keep `decy-parser` (clang-sys) as the default for accuracy
- Feature flag: `--parser=pure` vs `--parser=clang` (default)
- Pure parser eliminates:
  - LLVM installation requirement
  - All FFI `unsafe` code
  - Platform-specific `LLVM_CONFIG_PATH` / `LIBCLANG_PATH` setup
- Start with C89/C99 subset; expand iteratively using corpus convergence (Idea #2) as the quality measure

**Impact**: Zero-dependency installation (`cargo install decy` just works), eliminates the only `unsafe` FFI in the project, enables WASM compilation of decy itself.

---

## 4. Falsification Test Suite (from bashrs)

**Gap**: Decy has 5,363 tests but they primarily verify happy paths. There is no systematic attempt to *falsify* that each C construct transpiles correctly.

**What bashrs does**: A 120-point T-code checklist where each test attempts to falsify a specific feature. The corpus is append-only: when tests fail, the transpiler is fixed, never the test. Based on Popperian falsification methodology. See `bashrs/rash/tests/transpiler_tcode_tests.rs`.

**Proposal**:
- Create `tests/c_falsification_tests.rs` with C-codes (C001-C150):
  - C001-C010: Integer types and arithmetic
  - C011-C020: Control flow (if/else, switch, loops)
  - C021-C040: Pointers and arrays
  - C041-C060: Structs and unions
  - C061-C080: Functions and calling conventions
  - C081-C100: Standard library (malloc/free, stdio, string)
  - C101-C120: Preprocessor constructs
  - C121-C150: Advanced (function pointers, variadic, volatile, bitfields)
- Each test: transpile C fragment, compile Rust output, compare execution result
- Map each C-code to ISO C99 section and K&R chapter (ties into existing `C-VALIDATION-ROADMAP.yaml`)
- Tests are append-only: NEVER delete or weaken a test

**Impact**: Systematic coverage of the C language surface area. Makes regressions impossible to hide.

---

## 5. Optimization Pass Infrastructure (from CCC)

**Gap**: Decy generates Rust code but does not optimize the HIR before codegen. The output can be verbose and unidiomatic (e.g., unnecessary temporaries, redundant bounds checks, un-collapsed match arms).

**What CCC does**: 15 optimization passes on SSA IR including constant folding, copy propagation, dead code elimination, global value numbering, loop-invariant code motion, and function inlining. Passes run in a fixed-point loop (up to 3 iterations).

**Proposal**:
- Add an `decy-optimize` crate that operates on HIR before codegen:
  - **Constant folding**: Evaluate `#define`-derived constants at transpile time
  - **Dead code elimination**: Remove unreachable branches from `#ifdef` expansion
  - **Temporary elimination**: Collapse `let tmp = x; let y = tmp;` chains
  - **Pattern simplification**: `if (x != 0)` becomes `if x` in Rust idiom
  - **Loop idiom detection**: `for(i=0; i<n; i++) a[i]` becomes `.iter()` / `.iter_mut()`
  - **Match arm collapse**: Merge identical switch-case arms
  - **Borrow narrowing**: Upgrade `&mut` to `&` where mutation is absent
- Run passes in fixed-point loop until HIR stabilizes (max 3 iterations)
- Measure: LOC reduction ratio, idiom score (count of Rust-idiomatic patterns used)

**Impact**: More readable, more idiomatic Rust output. Fewer unnecessary `unsafe` blocks from overly conservative codegen.

---

## 6. Decision Tracing / Flight Recorder (from depyler)

**Gap**: When decy produces bad output, debugging *why* the transpiler made a particular ownership or lifetime decision requires reading through the entire pipeline. There is no audit trail.

**What depyler does**: A CITL (Confidence-based Iterative Transpilation Learning) flight recorder that logs every transpilation decision: type mapping chosen, confidence score, alternatives considered, and source location. Enables post-mortem analysis and ML training. See `depyler/crates/depyler-oracle/`.

**Proposal**:
- Add `--trace` flag to decy CLI that emits a JSON decision log:
  ```json
  {
    "source": "input.c:42:5",
    "decision": "pointer_classification",
    "input": "int *p = malloc(sizeof(int))",
    "chosen": "Box<i32>",
    "alternatives": ["&i32", "&mut i32", "*mut i32"],
    "confidence": 0.92,
    "reason": "single_alloc_single_free_pattern"
  }
  ```
- Log decisions for: pointer classification, lifetime inference, unsafe block insertion, type mapping, idiom detection
- Output as structured JSON for tooling integration
- Feed into `decy-oracle` for ML-based improvement over time

**Impact**: Debuggable transpilation. When output is wrong, trace shows exactly which decision was incorrect and why.

---

## 7. Multi-Architecture Codegen Validation (from CCC)

**Gap**: Decy generates Rust source code but never validates that the transpiled code produces the *same behavior* as the original C when compiled and run.

**What CCC does**: Validates correctness by compiling and running real-world projects (PostgreSQL, FFmpeg, etc.) and checking that test suites pass identically. This is the gold standard for transpiler correctness.

**Proposal**:
- Add `make validate-equivalence` that for each corpus program:
  1. Compile C source with `gcc` and run, capture output + exit code
  2. Transpile with `decy`, compile Rust output with `rustc`, run, capture output + exit code
  3. Diff results: stdout must match, exit code must match
- Start with simple programs (Tier 1-2 corpus) and expand
- Track **semantic equivalence rate** as a primary KPI alongside compile rate
- Use `decy-verify` to prove equivalence for pure functions where possible

**Impact**: Moves from "does it compile?" to "does it behave correctly?" - the actual goal of transpilation.

---

## 8. Deterministic Output Guarantee (from bashrs)

**Gap**: No guarantee that transpiling the same C file twice produces identical Rust output. Non-determinism in HashMap iteration order, graph traversal, or naming could produce different (but equivalent) output on each run.

**What bashrs does**: Explicit determinism and idempotency tests. Transpiling the same input twice must produce byte-identical output. This is enforced in CI.

**Proposal**:
- Add determinism property test:
  ```rust
  proptest! {
      #[test]
      fn transpile_is_deterministic(input in valid_c_program()) {
          let output1 = decy_transpile(&input);
          let output2 = decy_transpile(&input);
          assert_eq!(output1, output2);
      }
  }
  ```
- Audit all `HashMap` usage in the pipeline; replace with `BTreeMap` or `IndexMap` where iteration order matters
- Audit `petgraph` traversals for deterministic ordering
- Enforce in CI: transpile corpus twice, diff must be empty

**Impact**: Enables caching (Idea #1), makes diffs meaningful, prevents "phantom changes" that confuse users.

---

## 9. Builtin Assembler/Linker Awareness (from CCC)

**Gap**: Decy has no strategy for inline assembly (`__asm__`, `asm()`) in C source code. These blocks are common in systems code (Linux kernel, crypto libraries, SIMD code).

**What CCC does**: Implements a full assembler with support for x86-64, i686, AArch64, and RISC-V instruction sets. Bundles 17 SIMD intrinsic headers (SSE through AVX-512, ARM NEON).

**Proposal**:
- Add inline assembly handling strategy to `decy-parser` and `decy-codegen`:
  - **Preserve**: Emit `unsafe { asm!(...) }` with Rust inline assembly syntax translation
  - **Replace**: For known SIMD patterns, emit `std::arch` intrinsics instead of raw asm
  - **Skip**: Flag unsupported asm blocks with `// DECY: manual review required` and skip
- Add SIMD intrinsic mapping table: C intrinsics (`_mm_add_ps`) to Rust `std::arch` equivalents
- Track: percentage of asm blocks handled automatically vs requiring manual review

**Impact**: Enables transpilation of performance-critical systems code that uses inline assembly.

---

## 10. Zero-Dependency Aspiration (from CCC)

**Gap**: Decy has a large dependency tree (clang-sys, petgraph, syn, quote, proc-macro2, serde, tokio, rayon, etc.). Each dependency is a supply chain risk, compilation cost, and maintenance burden.

**What CCC does**: Zero external crate dependencies. Everything from scratch. While extreme, this eliminates all supply chain risk and makes the project entirely self-contained.

**Proposal** (pragmatic version):
- Audit dependency tree: `cargo tree --depth 1 | wc -l`
- Classify each dependency as essential vs removable
- Target reductions:
  - Replace `petgraph` with a purpose-built lightweight graph (decy only uses a fraction of petgraph's API)
  - Replace `syn`/`quote` with direct string-based Rust codegen (simpler, faster)
  - Make `serde` optional (only needed for `--trace` output, Idea #6)
  - Make `tokio`/`rayon` optional (only needed for `decy-repo` parallel processing)
- Goal: `cargo install decy` compile time under 60 seconds on a cold cache

**Impact**: Faster builds, smaller binary, reduced supply chain surface. If combined with Idea #3 (pure parser), decy could be fully self-contained.

---

## Priority Matrix

| # | Improvement | Effort | Impact | Source | Status |
|---|------------|--------|--------|--------|--------|
| 1 | Compilation Cache | Medium | High | depyler | Deferred (existing TranspilationCache) |
| 2 | Corpus Convergence Loop | Medium | **Critical** | depyler | **DONE** (DECY-191) |
| 3 | From-Scratch Parser | High | High | CCC | Deferred |
| 4 | Falsification Test Suite | Medium | **Critical** | bashrs | **DONE** (DECY-192, 200 tests, 180 pass, 20 falsified) |
| 5 | Optimization Passes | High | High | CCC | **DONE** (DECY-196) |
| 6 | Decision Tracing | Low | Medium | depyler | **DONE** (DECY-193) |
| 7 | Equivalence Validation | Medium | **Critical** | CCC | **DONE** (DECY-195) |
| 8 | Deterministic Output | Low | Medium | bashrs | **DONE** (DECY-194) |
| 9 | Inline Assembly Handling | Medium | Medium | CCC | **DONE** (DECY-197) |
| 10 | Dependency Reduction | Low | Low | CCC | Deferred |

**Implementation Date**: 2026-02-09

**Implemented**: Items 2, 4, 5, 6, 7, 8, 9
**Deferred**: Items 1, 3, 10 (lower priority, higher effort)

### Falsification Results Summary

| Range | Category | Tests | Passing | Falsified |
|-------|----------|-------|---------|-----------|
| C001-C050 | Basic types, control flow, structs, functions | 50 | 50 | 0 |
| C051-C060 | Pointers and address-of | 10 | 4 | 6 |
| C061-C070 | Arrays and indexing | 10 | 10 | 0 |
| C071-C080 | Type casting and coercion | 10 | 9 | 1 |
| C081-C090 | String and char operations | 10 | 2 | 8 |
| C091-C100 | Nested expressions, operator precedence | 10 | 9 | 1 |
| C101-C110 | Global variables, static storage | 10 | 8 | 2 |
| C111-C120 | Do-while, control flow edge cases | 10 | 9 | 1 |
| C121-C130 | Typedef, enum, sizeof | 10 | 9 | 1 |
| C131-C140 | Multi-dimensional arrays, recursion | 10 | 10 | 0 |
| C141-C150 | Compound assignment, algorithms | 10 | 10 | 0 |
| C151-C160 | Preprocessor-style, macro edge cases | 10 | 10 | 0 |
| C161-C170 | Advanced pointer patterns | 10 | 10 | 0 |
| C171-C180 | Nested data structures | 10 | 10 | 0 |
| C181-C190 | Bitfield, union, volatile patterns | 10 | 10 | 0 |
| C191-C200 | Real-world algorithm patterns | 10 | 10 | 0 |
| **TOTAL** | | **200** | **180** | **20** |

**Falsification Rate**: 90.0% passing (180/200)

Key gaps identified by falsification:
- **Char arithmetic**: char+int, char comparisons, escape char math
- **Pointer operations**: double pointer, void* cast, null check, pointer arithmetic
- **String operations**: char* parameters, string literal assignment
- **Boolean negation**: !x operator
- **Global struct**: struct typed global variables
- **Do-while break**: break inside do-while loop

---

## Summary

The three sibling projects each contribute a distinct lesson:

- **depyler** teaches **convergence discipline**: cache results, measure compile rate, trace decisions, iterate scientifically.
- **bashrs** teaches **falsification rigor**: test by trying to break things, enforce determinism, never weaken a test.
- **CCC** teaches **self-sufficiency and validation at scale**: own your parser, optimize your output, prove correctness against real-world programs.

Decy has strong process discipline (TDD, quality gates, roadmaps) but lacks the measurement infrastructure (corpus metrics, equivalence testing) and output optimization (HIR passes, idiom detection) that would elevate it from "generates Rust that compiles" to "generates Rust that a human would write."
