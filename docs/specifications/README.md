# Decy Specification

**Project**: Decy - C/C++/CUDA to Rust Transpiler
**Version**: 2.2.0
**Status**: Active Development
**Last Updated**: 2026-03-30

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Architecture](#2-architecture)
3. [Pipeline Stages](#3-pipeline-stages)
4. [Research Foundation](#4-research-foundation)
5. [Ownership Inference Algorithm](#5-ownership-inference-algorithm)
6. [Language Support Matrix](#6-language-support-matrix)
7. [Safety Guarantees](#7-safety-guarantees)
8. [Provable Contracts](#8-provable-contracts)
9. [Verification Ladder](#9-verification-ladder)
10. [Quality Standards](#10-quality-standards)
11. [Development Methodology](#11-development-methodology)
12. [Component Specifications](#12-component-specifications)
13. [Key Metrics](#13-key-metrics)
14. [Glossary](#14-glossary)

---

## 1. Executive Summary

Decy is a production-grade transpiler that converts C, C++, and CUDA source code into
safe, idiomatic Rust with minimal `unsafe` blocks (<5 per 1000 LOC). The project serves
a dual purpose:

1. **Transpiler**: Automated migration of legacy C/C++ codebases to memory-safe Rust.
2. **Data Factory**: Generation of verified C-to-Rust "Golden Traces" for training
   domain-specific LLMs that learn safety patterns.

Decy follows EXTREME TDD methodology, Toyota Way principles (Jidoka, Genchi Genbutsu,
Kaizen), and PMAT-qualified roadmap-driven development.

### 1.1 Design Principles

| Principle | Description |
|-----------|-------------|
| **Purification over Translation** | Refactor C idioms into safe Rust patterns, not 1:1 syntax mapping |
| **Safety First** | Eliminate entire bug classes: NULL deref, use-after-free, buffer overflow |
| **Data Factory** | Pipeline output is both code AND training data for ML models |
| **Toyota Way** | Jidoka (built-in quality), Muda (eliminate waste), Kaizen (continuous improvement) |
| **Popperian Falsification** | Every claim is a testable prediction; failures are discoveries |

### 1.2 Target Bug Class Elimination

| Bug Class | C Frequency | Decy Strategy |
|-----------|-------------|---------------|
| NULL pointer dereference | ~40% of CVEs | `Option<T>` inference |
| Use-after-free | ~30% of CVEs | Ownership/Drop pattern inference |
| Buffer overflow | ~20% of CVEs | `Vec<T>` and slice transformation |
| Memory leak | Common | RAII pattern generation |
| Data races | Growing | `Mutex<T>`, `Arc<T>` transformation |

---

## 2. Architecture

### 2.1 Multi-Stage Pipeline

```
C/C++/CUDA Source
       |
       v
+------------------+     +------------------+     +-------------------+
|  Stage 1: Parse  | --> |  Stage 2: Lower  | --> |  Stage 3: Analyze |
|  (decy-parser)   |     |  (decy-hir)      |     |  (decy-analyzer)  |
|  clang-sys FFI   |     |  AST -> HIR      |     |  CFG, DFG, types  |
+------------------+     +------------------+     +-------------------+
                                                          |
       +--------------------------------------------------+
       v
+------------------+     +------------------+     +-------------------+
|  Stage 4: Own    | --> |  Stage 5: Verify | --> |  Stage 6: Codegen |
|  (decy-ownership)|     |  (decy-verify)   |     |  (decy-codegen)   |
|  Pointer classify|     |  Safety props    |     |  Idiomatic Rust   |
+------------------+     +------------------+     +-------------------+
       |
       v
+------------------+
|  Optimization    |
|  (decy-core)     |
|  Pattern rewrites|
+------------------+
```

### 2.2 Crate Map

| Crate | Role | Unsafe Allowed? |
|-------|------|-----------------|
| `decy-parser` | C/C++/CUDA parsing via clang-sys | Yes (FFI only) |
| `decy-hir` | High-level Intermediate Representation | No |
| `decy-analyzer` | Static analysis: CFG, DFG, type inference | No |
| `decy-ownership` | Ownership and lifetime inference | No |
| `decy-verify` | Safety property verification | No |
| `decy-codegen` | Rust code generation | No |
| `decy-core` | Pipeline orchestration, optimization | No |
| `decy-llm` | LLM-assisted codegen and golden trace generation | No |
| `decy-stdlib` | C standard library mapping to Rust | No |
| `decy-debugger` | Pipeline introspection and visualization | No |
| `decy` | CLI binary and REPL | No |

---

## 3. Pipeline Stages

### 3.1 Stage 1: Parsing (decy-parser)

Parses C/C++/CUDA source using `clang-sys` (libclang bindings). Produces a typed AST
with full preprocessor expansion.

- **Input**: `.c`, `.h`, `.cpp`, `.hpp`, `.cu` files
- **Output**: `Ast` (functions, structs, enums, typedefs, macros, variables)
- **Key constraint**: Only crate permitted to use `unsafe` (FFI boundary)
- **Env requirements**: `LLVM_CONFIG_PATH`, `LIBCLANG_PATH`

**Component spec**: [decy-spec-v1.md](components/decy-spec-v1.md) Section 3

### 3.2 Stage 2: HIR Lowering (decy-hir)

Converts the C-oriented AST into a Rust-oriented High-level Intermediate Representation.

- **Type mapping**: `int` -> `i32`, `char*` -> `&str`/`String`, `void*` -> generic
- **Serializable**: JSON output for debugging and golden trace generation
- **Preserves source location** for error reporting

**Component spec**: [decy-spec-v1.md](components/decy-spec-v1.md) Section 4

### 3.3 Stage 3: Analysis (decy-analyzer)

Static analysis using `petgraph` for control flow and data flow graphs.

- **Control flow graph**: Loop detection, reachability, dead code
- **Data flow graph**: Def-use chains, reaching definitions
- **Type inference**: Resolves implicit C types to explicit Rust types

**Component spec**: [decy-spec-v1.md](components/decy-spec-v1.md) Section 5

### 3.4 Stage 4: Ownership Inference (decy-ownership)

The most critical stage for unsafe minimization. Classifies every pointer.

- **Owning pointers**: `malloc`/`free` pairs -> `Box<T>`, array alloc -> `Vec<T>`
- **Borrowing pointers**: Read-only access -> `&T`, mutating access -> `&mut T`
- **Lifetime inference**: C variable scopes -> Rust lifetime annotations
- **Coverage target**: 90% (higher than other crates)

**Component spec**: [decy-unsafe-minimization-strategy.md](components/decy-unsafe-minimization-strategy.md)

### 3.5 Stage 5: Verification (decy-verify)

Validates safety properties before code generation.

- **Memory safety**: No dangling pointers, no double-free
- **Type safety**: All casts are sound
- **Borrow checker simulation**: Validates ownership decisions
- **Compilation check**: Generated Rust must pass `rustc` type checking

**Component spec**: [real-world-c.md](components/real-world-c.md) Strategy 1

### 3.6 Stage 6: Code Generation (decy-codegen)

Produces idiomatic, formatted Rust code.

- **Output**: Clippy-clean, `rustfmt`-formatted Rust
- **Target**: <5 unsafe blocks per 1000 LOC
- **Uses**: `quote`, `proc-macro2` for AST construction

**Component spec**: [decy-spec-v1.md](components/decy-spec-v1.md) Section 8

---

## 4. Research Foundation

Decy's design is grounded in peer-reviewed transpilation research. Key techniques
adopted or planned, with arXiv references:

### 4.1 Ownership Inference (Static Analysis)

| Technique | Source | Impact | Status |
|-----------|--------|--------|--------|
| **CROWN type qualifier system** | Zhang et al. [2303.10515] (CAV 2023) | 500K LOC in <10s; 3 qualifiers: ownership, mutability, fatness | Adopted |
| **Split trees for pointer arithmetic** | Fromherz & Protzenko [2412.15042] (Scylla) | Formal correctness: `p + offset` -> `&p[offset..]` | Planned |
| **Decision-tree pointer classification** | Gao et al. [2505.04852] (PR2, NeurIPS 2025) | 13.22% raw pointer elimination per project | Planned |
| **DFG-guided type migration** | SOAP '25 at PLDI 2025 | Enum generation for multiply-borrowed globals | Planned |

### 4.2 LLM-Assisted Translation

| Technique | Source | Impact | Status |
|-----------|--------|--------|--------|
| **Iterative feedback with error categories** | Shiraishi et al. [2409.10506] (SmartC2Rust) | 99.4% unsafe reduction via multi-round repair | Planned (decy-llm) |
| **Skeleton-first compilation** | Wang et al. [2508.04295] (EvoC2Rust) | +43.59% safety rate; compile stubs then fill bodies | Planned |
| **C pre-refactoring before translation** | Dehghan et al. [2511.20617] (Rustine) | 100% compilability; maximize constness, expand macros | Partial |
| **Dependency-graph decomposition** | Cai et al. [2503.17741] (RustMap) | Project-scale via call graph ordering | Adopted (petgraph) |
| **Two-phase: semantics then safety** | Zhou et al. [2503.12511] (SACTOR) | 100% unsafe-free after idiomatic phase | Planned |

### 4.3 Verification Techniques

| Technique | Source | Impact | Status |
|-----------|--------|--------|--------|
| **WASM oracle differential testing** | Yang et al. [2404.18852] (VERT, ICLR 2025) | 1% -> 42% bounded model checking pass rate | Planned |
| **Symbolic equivalence scoring (S3)** | Bai & Palit [2510.07604] (RustAssure) | Per-function semantic similarity metric | Planned |
| **FFI-based incremental verification** | SACTOR [2503.12511] | Link Rust back to C for end-to-end testing | Planned |
| **Boundary sanitization** | Braunsdorf et al. [2510.20688] (SafeFFI) | 98% fewer runtime checks at safe/unsafe boundary | Planned |

### 4.4 Benchmarks

| Benchmark | Source | Ceiling |
|-----------|--------|---------|
| **CRUST-Bench** | Khatry et al. [2504.15254] (COLM 2025) | 100 repos; best: 48% with Claude Opus 4 + repair loop |
| **HPCTransCompile** | Lv et al. [2506.10401] | CUDA transpilation dataset; 43.8% avg speedup |

### 4.5 Cross-Stack Intelligence (batuta oracle)

Patterns validated across sibling transpilers (depyler, bashrs, ruchy):

- **Testing triple**: proptest (1000 cases/property) + mutation testing (>=90% kill) + extreme TDD
- **Oracle/CITL**: Compiler-in-the-Loop Training with cross-project pattern transfer (Yokoten)
- **Renacer source maps**: Unified profiling across transpilers via `--source-map` flag
- **Provable contracts**: YAML -> trait scaffold -> compile-time enforcement (adopted by 13 repos)

---

## 5. Ownership Inference Algorithm

The `decy-ownership` crate implements a CROWN-inspired type qualifier system.

### 5.1 Three-Qualifier Model

Every pointer is classified along three axes (Zhang et al. [2303.10515]):

```
           Ownership          Mutability        Fatness
           ---------          ----------        -------
Pointer -> Owning | Borrowing  Mutable | Const   Thin | Fat(slice)
           |                   |                  |
           v                   v                  v
           Box<T> / Vec<T>     &mut T / &T        *const T / &[T]
```

### 5.2 Classification Rules

| C Pattern | Ownership | Mutability | Fatness | Rust Output |
|-----------|-----------|------------|---------|-------------|
| `malloc` + `free` | Owning | Mutable | Thin | `Box<T>` |
| `malloc(n * sizeof)` | Owning | Mutable | Fat | `Vec<T>` |
| Read-only parameter | Borrowing | Const | Thin | `&T` |
| Mutated parameter | Borrowing | Mutable | Thin | `&mut T` |
| Array parameter | Borrowing | Const | Fat | `&[T]` |
| `p + offset` (arithmetic) | Borrowing | * | Fat | `&p[offset..]` (split tree) |
| NULL-checked pointer | * | * | * | `Option<...>` wrapper |
| Escapes scope | Owning | * | * | `Box<T>` (forced) |

### 5.3 Confidence Scoring

Each classification carries a confidence score (0.0-1.0):

| Signal | Confidence |
|--------|------------|
| `malloc` allocation | 0.9 Owning |
| Function parameter (default) | 0.8 ImmutableBorrow |
| Mutating write through pointer | 0.9 MutableBorrow |
| Pointer arithmetic detected | 0.7 Fat (slice) |
| Escapes via return | 0.95 Owning |
| Ambiguous (multiple uses) | 0.5 Unknown -> fallback to `*mut T` |

Below 0.6 confidence, the pointer remains raw (`*const T` / `*mut T`) with an
`unsafe` block and a `// SAFETY:` comment documenting the ambiguity.

**Full algorithm**: [decy-unsafe-minimization-strategy.md](components/decy-unsafe-minimization-strategy.md)

---

## 6. Language Support Matrix

### 6.1 C Support (Production)

| Feature | Status | C99 Reference |
|---------|--------|---------------|
| Primitive types (int, float, char, etc.) | Complete | SS6.2.5 |
| Pointers and arrays | Complete | SS6.5.2.1, SS6.5.3.2 |
| Structs and unions | Complete | SS6.7.2.1 |
| Enums | Complete | SS6.7.2.2 |
| Control flow (if, for, while, switch) | Complete | SS6.8 |
| Functions and prototypes | Complete | SS6.9.1 |
| Typedefs | Complete | SS6.7.8 |
| Preprocessor macros | Partial | SS6.10 |
| Standard library headers | Partial | SS7 |
| Variadic functions | Planned | SS6.10.3 |
| Bitfields | Planned | SS6.7.2.1 |
| Inline assembly | Preserved | N/A |

**Validation north star**: `docs/C-VALIDATION-ROADMAP.yaml` (150 constructs mapped)

### 6.2 C++ Support (Phase 2 Complete)

| Feature | Phase | Status | Rust Mapping |
|---------|-------|--------|-------------|
| Classes (data + methods) | Phase 1 | **Complete** (DECY-200) | `struct` + `impl` |
| Namespaces | Phase 1 | **Complete** (DECY-201) | `mod` modules |
| Constructors / destructors | Phase 1 | **Complete** (DECY-202) | `new()` + `impl Drop` |
| `new`/`delete` | Phase 2 | **Complete** (DECY-207) | `Box::new()` / `drop()` |
| Operator overloading | Phase 2 | **Complete** (DECY-208) | `std::ops` traits |
| Single inheritance | Phase 2 | **Complete** (DECY-209) | Composition + `Deref`/`DerefMut` |
| Virtual dispatch | Phase 3 | Planned | `dyn Trait` |
| Simple templates (1 param) | Phase 3 | Medium | Generics + trait bounds |
| Exceptions (try/catch/throw) | Phase 4 | Med-Hard | `Result<T, E>` + `?` |
| Lambdas | Phase 4 | Medium | Closures |
| Template specialization | Phase 5 | Hard | Manual / LLM-assisted |
| SFINAE / enable_if | Phase 5 | Very Hard | Trait bounds redesign |
| Multiple inheritance | Phase 5 | Hard | Trait composition |
| Template metaprogramming | Out of scope | N/A | Manual rewrite |

**libclang cursor support**: All C++ cursor types are exposed by `clang-sys` 1.7
(`CXCursor_ClassDecl`, `CXCursor_CXXMethod`, `CXCursor_Namespace`,
`CXCursor_ClassTemplate`, etc.).

### 6.3 CUDA Support (Phase 1 Complete)

| Feature | Phase | Status | Strategy |
|---------|-------|--------|----------|
| `.cu` file parsing | Phase 1 | **Complete** (DECY-198) | C++ mode for .cu files |
| `__global__`, `__device__`, `__host__` qualifiers | Phase 1 | **Complete** (DECY-199) | Extract via `CXCursor_CUDAGlobalAttr` (414), `CUDADeviceAttr` (413), `CUDAHostAttr` (415) |
| `__global__` kernel FFI codegen | Phase 1 | **Complete** (DECY-211) | `extern "C"` declarations with raw pointer types |
| `__device__` function handling | Phase 1 | **Complete** (DECY-211) | Comment noting GPU-only (not transpiled) |
| `__shared__` memory | Phase 2 | Planned | Extract via `CXCursor_CUDASharedAttr` (416) |
| Host-side C code | Phase 1 | **Complete** | Normal transpilation pipeline |
| `cudaMalloc`/`cudaFree`/`cudaMemcpy` | Phase 2 | RAII wrappers or `cudarc` bindings |
| Kernel launch `<<<grid, block>>>` | Phase 2 | FFI stubs or `cudarc` API |
| Thread indexing (`threadIdx`, `blockIdx`) | Phase 2 | Preserved in FFI kernels |
| Inline PTX assembly | Out of scope | Preserved verbatim |

**Note**: Full CUDA-to-Rust-GPU transpilation (e.g., targeting `rust-gpu`) is out of
scope. The strategy is: transpile host code to safe Rust, generate FFI wrappers for
device kernels.

---

## 7. Safety Guarantees

### 7.1 Unsafe Minimization (4-Phase Strategy)

```
Phase 1: Pattern-Based     100% -> 50%   malloc/free -> Box, arrays -> Vec
Phase 2: Ownership Infer    50% -> 20%   Pointer classification, &T / &mut T
Phase 3: Lifetime Infer     20% -> 10%   Scope analysis, lifetime annotations
Phase 4: Safe Wrappers      10% ->  <5%  Abstractions around remaining unsafe
```

**Full strategy**: [decy-unsafe-minimization-strategy.md](components/decy-unsafe-minimization-strategy.md)

### 7.2 Concurrency Transformations

| C Pattern | Rust Output |
|-----------|-------------|
| `pthread_mutex_t` | `Mutex<T>` |
| `pthread_rwlock_t` | `RwLock<T>` |
| `pthread_create` | `std::thread::spawn` |
| `atomic_*` | `std::sync::atomic::*` |

### 7.3 Verification Properties

Every transpilation output is verified for:

1. **Compilability**: Output passes `rustc --edition 2021`
2. **No new unsafe**: Unsafe count does not regress
3. **Type soundness**: All casts are valid
4. **Ownership validity**: Unique owner per allocation, borrows don't outlive owner

---

## 8. Provable Contracts

Decy adopts the [provable-contracts](../../../provable-contracts) framework to enforce
transpilation correctness invariants at compile time. The chain:

```
Paper (arXiv) -> Equation -> Contract (YAML) -> Trait (scaffold) -> Kernel (impl)
  -> Test (probar) -> Proof (Kani) -> Theorem (Lean 4)
```

### 8.1 Transpilation Contracts

Four domain-specific contracts govern Decy's correctness:

| Contract | Equations | What It Guarantees |
|----------|-----------|-------------------|
| `type-preservation-v1.yaml` | `type_map: C_type -> Rust_type` (injective on base types) | `int` always becomes `i32`, `char*` always becomes `&str`/`String`, sizeof preserved |
| `semantic-equivalence-v1.yaml` | `observational_equivalence: output(C) = output(Rust)` | Control flow, arithmetic, side effects preserved across transpilation |
| `pointer-safety-v1.yaml` | `pointer_to_reference: classify(ptr) -> {Box, &T, &mut T, Vec}` | NULL checks inserted, bounds checking added, ownership correctly inferred |
| `memory-safety-v1.yaml` | `escape_analysis`, `ownership_invariant`, `lifetime_safety` | No use-after-free, no double-free, borrows don't outlive owner |

### 8.2 Enforcement Mechanism

Zero runtime cost in release builds (all assertions are `debug_assert!()`):

```
Stage A: Equation exists          YAML schema parse
Stage B: Lean proof (no sorry)    Lean 4 theorem (long-term)
Stage C: YAML validation          pv lint (7 gates)
Stage D: build.rs codegen         CONTRACT_* env vars + debug_assert!()
Stage E: #[contract] macro        Compile-time binding check
Stage F: Test execution           cargo test blocks merge
```

### 8.3 Binding Registry

Each transpilation function maps to a contract equation:

```yaml
# contracts/decy/binding.yaml
bindings:
  - contract: pointer-safety-v1.yaml
    equation: pointer_to_reference
    function: classify_pointer
    module_path: decy_ownership::classifier
    status: implemented
  - contract: type-preservation-v1.yaml
    equation: type_map
    function: c_type_to_hir
    module_path: decy_hir::type_mapping
    status: implemented
  - contract: semantic-equivalence-v1.yaml
    equation: control_flow_equivalence
    function: generate_statement
    module_path: decy_codegen::statements
    status: implemented
```

### 8.4 Annotated Functions

Core transpilation functions carry `#[contract]` annotations:

```rust
#[contract("type-preservation-v1", equation = "type_map")]
pub fn c_type_to_hir(c_ty: &ast::Type) -> HirType { ... }

#[contract("pointer-safety-v1", equation = "pointer_to_reference")]
pub fn classify_pointer(ptr: &PointerInfo) -> OwnershipKind { ... }
```

Build fails if any binding is `not_implemented` (policy: `AllImplemented`).

---

## 9. Verification Ladder

Transpilation correctness is verified at six levels, from weakest to strongest:

```
Level   Method                  Tool            Guarantee
-----   ------                  ----            ---------
  L5    Theorem proving         Lean 4          True for ALL inputs. Period.
  L4    Bounded model check     Kani            True for ALL inputs <= size N.
  L3    Property-based test     probar/proptest True for ~10,000 random inputs.
  L2    Falsification test      #[test]         True for specific edge cases.
  L1    Type system             rustc           True by construction.
  L0    Code review             Human eyes      "Looks right to me."
```

### 9.1 Current Coverage by Level

| Level | Decy Status |
|-------|-------------|
| L0 | All code reviewed via PR |
| L1 | `#![deny(missing_docs)]`, strong typing throughout |
| L2 | 150 falsification tests (86.7% pass rate, 20 falsified and marked) |
| L3 | proptest: 1000 cases/property, 100+ properties per crate |
| L4 | Kani harnesses planned for ownership invariants |
| L5 | Lean 4 proofs planned for type preservation equations |

### 9.2 Differential Testing (Planned)

Two complementary approaches from recent research:

1. **WASM Oracle** (VERT [2404.18852]): Compile C source to WASM as reference
   implementation, compare against Rust output for bounded model checking.

2. **Symbolic Equivalence (S3)** (RustAssure [2510.07604]): Per-function symbolic
   execution of C and Rust, comparing return value constraints.

3. **FFI Bridge** (SACTOR [2503.12511]): Link transpiled Rust back against original C
   via `extern "C"` for end-to-end integration testing.

---

## 10. Quality Standards

| Metric | Minimum | Target | Enforcement |
|--------|---------|--------|-------------|
| Test coverage | 80% | 85% (90% for ownership) | Pre-commit hook |
| Mutation score | 85% | 90% | CI/nightly |
| Clippy warnings | 0 | 0 | Pre-commit hook |
| SATD comments | 0 | 0 | Pre-commit hook |
| Unsafe per 1000 LOC | <5 | <3 | Sprint tracking |
| Cyclomatic complexity | <=10 | <=8 | Pre-commit hook |
| Cognitive complexity | <=15 | <=12 | Pre-commit hook |

**Testing methodology**: [improve-testing-quality-using-certeza-concepts.md](components/improve-testing-quality-using-certeza-concepts.md)
**SQLite-style testing**: [testing-sqlite-style.md](components/testing-sqlite-style.md)

---

## 11. Development Methodology

### 11.1 EXTREME TDD (RED-GREEN-REFACTOR)

Every ticket follows:
1. **RED**: Write failing tests, commit with `[RED]` prefix
2. **GREEN**: Minimal implementation to pass, commit with `[GREEN]` prefix
3. **REFACTOR**: Clean up, meet quality gates, commit with `[REFACTOR]` prefix

### 11.2 Three-Tiered Testing

| Tier | Trigger | Time | Scope |
|------|---------|------|-------|
| Tier 1: ON-SAVE | Every file save | <1s | Unit tests, clippy, fmt |
| Tier 2: ON-COMMIT | Pre-commit hook | 1-5min | Full suite, coverage, property tests |
| Tier 3: ON-MERGE | PR/nightly CI | 1-4hr | Mutation, Kani, Miri, fuzz |

### 11.3 Release Policy

- **Release day**: Friday only (weekly cadence)
- **Exception**: Security patches (any day, expedited testing)
- **Rationale**: Toyota Way Jidoka - quality built in through predictable cadence

### 11.4 Roadmap-Driven Development

All work is ticket-driven via `roadmap.yaml`. No code without a ticket. State changes
(status, phase) are committed to git.

**Roadmap commands**: `make sync-roadmap`, `make roadmap-status`, `make check-roadmap`

---

## 12. Component Specifications

### 12.1 Core Architecture

| Document | Description | LOC |
|----------|-------------|-----|
| [decy-spec-v1.md](components/decy-spec-v1.md) | Complete technical specification v1.0: pipeline architecture, crate responsibilities, 20-sprint roadmap | 2,708 |
| [decy-unified-spec.md](components/decy-unified-spec.md) | Unified v2.0 spec: AI-first pivot, golden trace generation, model training strategy | 900 |

### 12.2 Safety and Correctness

| Document | Description | LOC |
|----------|-------------|-----|
| [decy-unsafe-minimization-strategy.md](components/decy-unsafe-minimization-strategy.md) | 4-phase unsafe reduction strategy: pattern-based, ownership inference, lifetime inference, safe wrappers | 568 |
| [purify-c-spec.md](components/purify-c-spec.md) | Purification philosophy: transform C idioms into safe Rust patterns (inspired by bashrs) | 790 |
| [translation-ideas-spec.md](components/translation-ideas-spec.md) | Research-backed translation techniques: scalar pointer replacement, array parameter transformation, concurrency mapping | 1,214 |

### 12.3 Testing and Quality

| Document | Description | LOC |
|----------|-------------|-----|
| [improve-testing-quality-using-certeza-concepts.md](components/improve-testing-quality-using-certeza-concepts.md) | Three-tiered testing workflow (ON-SAVE / ON-COMMIT / ON-MERGE), Certeza methodology | 903 |
| [testing-sqlite-style.md](components/testing-sqlite-style.md) | SQLite-inspired testing: 100% branch coverage target, OOM testing, fuzz testing | 841 |
| [real-world-c.md](components/real-world-c.md) | Compile-the-output verification, real-world C corpus testing, incremental adoption | 366 |

### 12.4 Oracle and ML

| Document | Description | LOC |
|----------|-------------|-----|
| [oracle-integration-spec.md](components/oracle-integration-spec.md) | Oracle integration: REPL commands, batuta RAG, C-to-Rust pattern lookup | 338 |
| [training-oracle-spec.md](components/training-oracle-spec.md) | Oracle training pipeline: golden trace generation, model fine-tuning, evaluation metrics | 980 |
| [improvements-ml-techniques.md](components/improvements-ml-techniques.md) | ML-enhanced ownership inference: GNN for pointer graphs, transformer for pattern recognition | 511 |

### 12.5 Infrastructure and Release

| Document | Description | LOC |
|----------|-------------|-----|
| [10.0-decy-release.md](components/10.0-decy-release.md) | NASA-level release criteria: safety-critical compliance, Popperian falsification checklist | 593 |
| [header-support-spec.md](components/header-support-spec.md) | C standard library header support: stdio.h, stdlib.h, string.h mapping to Rust | 781 |
| [improve-decy-spec.md](components/improve-decy-spec.md) | Cross-project improvements from depyler/bashrs: hermetic cache, corpus convergence | 290 |

---

## 13. Key Metrics

### 13.1 Current Status

| Metric | Value |
|--------|-------|
| Version | 2.2.0 |
| Test coverage | 97.60% |
| Workspace crates | 11 |
| C constructs mapped | 150 (see C-VALIDATION-ROADMAP.yaml) |
| C++ features supported | 9 (classes, namespaces, ctor/dtor, new/delete, operators, inheritance, implicit this, bool/nullptr) |
| CUDA features supported | 5 (.cu parsing, qualifier extraction, kernel FFI, device annotation, inline keyword detection) |
| Parser tests | 173 passing |
| HIR tests | 192 passing |
| E2E semantic tests | 10 passing |
| E2E rustc compile tests | 5 passing (1 ignored: Box<T> vs *mut T) |
| Provable contracts | 2 (cpp-type-preservation-v1, cuda-kernel-safety-v1) with compile-time macros |
| Runnable examples | 3 (cpp_class, cuda, dogfood_validation) |
| Dogfood status | 5/5 patterns compile with rustc (class, namespace, operators, inheritance, CUDA) |
| Compilation success | 100% |
| Unsafe per 1000 LOC | <5 |
| PMAT TDG average | 92.8 |

### 13.2 Supported Transformations

| C Pattern | Rust Output | Status |
|-----------|-------------|--------|
| `malloc`/`free` | `Box::new()` / drop | Complete |
| Array allocation | `Vec::with_capacity()` | Complete |
| `char*` strings | `&str` / `String` | Complete |
| Pointer arithmetic | Safe slice indexing | Complete |
| Array parameters | `&[T]` slices | Complete |
| `pthread_mutex_t` | `Mutex<T>` | Complete |
| `switch`/`case` | `match` | Complete |
| `for`/`while`/`do` | `loop`/`while`/`for` | Complete |
| `struct` with methods | `struct` + `impl` | Complete |
| `typedef` | `type` alias | Complete |
| `enum` | `enum` | Complete |
| `#define` constants | `const` | Partial |
| Function pointers | `fn()` / `Fn` traits | Complete |

---

## 14. Glossary

| Term | Definition |
|------|------------|
| **HIR** | High-level Intermediate Representation: Rust-oriented IR bridging C AST and Rust codegen |
| **Golden Trace** | Verified (C input, Rust output) pair used for ML model training |
| **Purification** | Transforming C idioms into safe, idiomatic Rust (not 1:1 syntax translation) |
| **Ownership Inference** | Algorithm that classifies C pointers as owning (`Box`), borrowing (`&T`), or mutable (`&mut T`) |
| **SATD** | Self-Admitted Technical Debt: `TODO`, `FIXME`, `HACK` comments (zero tolerance) |
| **PMAT** | Project Management and Automation Toolkit: roadmap-driven development framework |
| **Jidoka** | Toyota Way principle: build quality in at the source, stop the line on defects |
| **Andon Cord** | Emergency stop protocol when validation reveals a bug (see C-VALIDATION-ROADMAP.yaml) |
| **Falsification** | Popperian method: every test is a prediction that can be refuted by evidence |
| **Certeza** | Three-tiered testing methodology: ON-SAVE (<1s), ON-COMMIT (1-5min), ON-MERGE (hours) |
| **CROWN** | Type qualifier system for ownership inference: ownership x mutability x fatness (Zhang et al. CAV 2023) |
| **Split Tree** | Static analysis converting pointer arithmetic to safe Rust slice splitting (Fromherz, Scylla) |
| **Provable Contract** | YAML specification encoding equations, proof obligations, and falsification tests for compile-time enforcement |
| **Verification Ladder** | L0 (review) through L5 (Lean theorem) hierarchy of transpilation correctness guarantees |
| **S3 Score** | Semantic Similarity Score: per-function symbolic equivalence metric between C and transpiled Rust |
| **CITL** | Compiler-in-the-Loop Training: oracle system that learns fix patterns from rustc error feedback |
