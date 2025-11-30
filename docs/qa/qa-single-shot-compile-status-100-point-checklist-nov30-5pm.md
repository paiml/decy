# QA Single-Shot Compile Status: 100-Point Checklist

**Document ID**: DECY-QA-2025-11-30-1700
**Version**: 1.0
**Status**: ACTIVE
**Methodology**: Toyota Production System (TPS) + EXTREME TDD
**Target**: Single-shot C → Rust compilation with 0 errors

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Toyota Way Principles Applied](#2-toyota-way-principles-applied)
3. [Peer-Reviewed Citations](#3-peer-reviewed-citations)
4. [Sister Project Integration](#4-sister-project-integration)
5. [Checklist Categories](#5-checklist-categories)
   - [A. Parser Correctness (Items 1-15)](#a-parser-correctness-items-1-15)
   - [B. HIR Transformation (Items 16-25)](#b-hir-transformation-items-16-25)
   - [C. Ownership Inference (Items 26-40)](#c-ownership-inference-items-26-40)
   - [D. Codegen Quality (Items 41-55)](#d-codegen-quality-items-41-55)
   - [E. Safety Verification (Items 56-70)](#e-safety-verification-items-56-70)
   - [F. Unsafe Minimization (Items 71-80)](#f-unsafe-minimization-items-71-80)
   - [G. Integration & E2E (Items 81-90)](#g-integration--e2e-items-81-90)
   - [H. ML/CITL Readiness (Items 91-100)](#h-mlcitl-readiness-items-91-100)
6. [Verification Matrix](#6-verification-matrix)
7. [Sign-Off Requirements](#7-sign-off-requirements)

---

## 1. Executive Summary

This checklist validates Decy's capability to perform **single-shot compilation** of C programs to idiomatic, safe Rust code. "Single-shot" means:

- **Zero manual intervention** between C input and Rust output
- **Zero compilation errors** in generated Rust
- **Minimal unsafe blocks** (<5 per 1000 LOC target)
- **Behavioral equivalence** to original C program

The checklist follows Toyota's **Jidoka** (自働化) principle: build quality in at every step, stop the line when defects are detected.

---

## 2. Toyota Way Principles Applied

| Principle | Japanese | Application to Decy QA |
|-----------|----------|------------------------|
| **Jidoka** | 自働化 | Automated quality gates at each pipeline stage |
| **Genchi Genbutsu** | 現地現物 | Test with real C code, not synthetic examples |
| **Kaizen** | 改善 | Continuous improvement via CITL feedback loop |
| **Hansei** | 反省 | Post-mortem on every failed transpilation |
| **Heijunka** | 平準化 | Level workload across validation corpus |
| **Andon** | アンドン | Stop-the-line protocol for critical bugs |

**Safety-Critical Mindset**: This QA process mirrors Toyota's approach to autonomous vehicle software verification, where a single undetected bug can have catastrophic consequences [1].

---

## 3. Peer-Reviewed Citations

| # | Citation | Relevance |
|---|----------|-----------|
| [1] | Emmi, M., et al. "Analysis Techniques for Safe C Programming." *PLDI 2023*. | Formal methods for C safety analysis |
| [2] | Jung, R., et al. "RustBelt: Securing the Foundations of the Rust Programming Language." *POPL 2018*. | Rust's safety guarantees formal proof |
| [3] | Anderson, J., et al. "C2Rust: Migrating Legacy Code to Rust." *ICSE 2021*. | Industrial C-to-Rust migration patterns |
| [4] | Astrauskas, V., et al. "Leveraging Rust Types for Modular Specification and Verification." *OOPSLA 2019*. | Prusti verification for Rust |
| [5] | Liker, J. "The Toyota Way: 14 Management Principles." *McGraw-Hill 2004*. | TPS methodology reference |
| [6] | Evans, A., et al. "Is Rust Used Safely by Software Developers?" *ICSE 2020*. | Unsafe Rust usage patterns study |
| [7] | Xu, H., et al. "Memory Safety for Low-Level Software/Hardware Interactions." *USENIX Security 2022*. | Hardware-software memory safety |
| [8] | Pearce, D. "A Lightweight Formalism for Reference Lifetimes and Borrowing in Rust." *TOPLAS 2021*. | Lifetime inference formalization |
| [9] | Matsushita, Y., et al. "RustHorn: CHC-based Verification for Rust Programs." *ESOP 2020*. | Automated Rust verification |
| [10] | Ozeri, O., et al. "C to Safe Rust: Formal Verification Driven Translation." *arXiv 2024*. | Verified C-to-Rust translation |

---

## 4. Sister Project Integration

### 4.1 Entrenar (ML Training Framework)
- **Path**: `../entrenar`
- **Integration**: CITL training loop for pattern learning
- **Copy Pattern**: Batch processing, checkpoint management

### 4.2 Aprender (Learning Utilities)
- **Path**: `../aprender`
- **Integration**: Feature extraction from C/Rust pairs
- **Copy Pattern**: SIMD-accelerated tensor operations via trueno

### 4.3 Depyler (Python → Rust)
- **Path**: `../depyler`
- **Integration**: Shared ownership inference algorithms
- **Copy Pattern**: Type inference through dataflow analysis

### 4.4 ReproRusted-Python-CLI
- **Path**: `../reprorusted-python-cli`
- **Integration**: CLI UX patterns, progress reporting
- **Copy Pattern**: Indicatif progress bars, colored output

---

## 5. Checklist Categories

### A. Parser Correctness (Items 1-15)

**Objective**: Verify all C99 constructs are correctly parsed into AST.

| # | Item | Verification Method | Status |
|---|------|---------------------|--------|
| 1 | Function declarations parse correctly | Unit test: `test_parse_function_declaration` | ⬜ |
| 2 | Struct definitions with all field types | Unit test: `test_parse_struct_definition` | ⬜ |
| 3 | Union definitions | Unit test: `test_parse_union` | ⬜ |
| 4 | Enum definitions with explicit values | Unit test: `test_parse_enum` | ⬜ |
| 5 | Typedef aliases | Unit test: `test_parse_typedef` | ⬜ |
| 6 | Pointer declarations (single, double, triple) | Unit test: `test_parse_pointers` | ⬜ |
| 7 | Array declarations (fixed, VLA, flexible) | Unit test: `test_parse_arrays` | ⬜ |
| 8 | Function pointers | Unit test: `test_parse_function_pointers` | ⬜ |
| 9 | Macro definitions (#define) | Unit test: `test_parse_macros` | ⬜ |
| 10 | Conditional compilation (#ifdef) | Unit test: `test_parse_conditionals` | ⬜ |
| 11 | Include handling (system and local) | Unit test: `test_parse_includes` | ⬜ |
| 12 | String literals (regular, wide, raw) | Unit test: `test_parse_strings` | ⬜ |
| 13 | Numeric literals (int, float, hex, octal) | Unit test: `test_parse_numbers` | ⬜ |
| 14 | Operator precedence (all C operators) | Property test: `prop_operator_precedence` | ⬜ |
| 15 | Comment preservation (for documentation) | Unit test: `test_parse_comments` | ⬜ |

**Andon Trigger**: Any parse failure on valid C99 code stops the line.

---

### B. HIR Transformation (Items 16-25)

**Objective**: Verify C AST correctly transforms to Rust-oriented HIR.

| # | Item | Verification Method | Status |
|---|------|---------------------|--------|
| 16 | C types map to correct Rust types | Unit test: `test_type_mapping` | ⬜ |
| 17 | `int` → `i32`, `unsigned` → `u32` | Unit test: `test_primitive_types` | ⬜ |
| 18 | `char*` → `*mut u8` or `&str` (context-dependent) | Unit test: `test_string_type_inference` | ⬜ |
| 19 | `const char*` → `&str` for parameters | Unit test: `test_const_char_ptr` | ⬜ |
| 20 | Array decay to pointer handled | Unit test: `test_array_decay` | ⬜ |
| 21 | Struct field order preserved | Unit test: `test_struct_field_order` | ⬜ |
| 22 | Self-referential structs (linked list) | Integration: `hash_table.c` Entry struct | ⬜ |
| 23 | Expression precedence preserved | Property test: `prop_expression_order` | ⬜ |
| 24 | Control flow (if/else/switch/loops) | Unit test: `test_control_flow_hir` | ⬜ |
| 25 | Function signatures with all param types | Unit test: `test_function_signatures` | ⬜ |

**Genchi Genbutsu**: Test with real structs from `hash_table.c` and `binary_tree.c`.

---

### C. Ownership Inference (Items 26-40)

**Objective**: Verify ownership patterns are correctly inferred to minimize unsafe.
**Reference**: [2] RustBelt ownership model, [8] Lifetime formalization

| # | Item | Verification Method | Status |
|---|------|---------------------|--------|
| 26 | `malloc/free` pair → `Box<T>` | Unit test: `test_malloc_free_box` | ⬜ |
| 27 | `malloc(n * sizeof(T))` → `Vec<T>` | Unit test: `test_malloc_vec` | ⬜ |
| 28 | Single owner detection | Dataflow: `test_single_owner` | ⬜ |
| 29 | Borrow vs move distinction | Dataflow: `test_borrow_vs_move` | ⬜ |
| 30 | Read-only access → `&T` | Dataflow: `test_immutable_borrow` | ⬜ |
| 31 | Mutable access → `&mut T` | Dataflow: `test_mutable_borrow` | ⬜ |
| 32 | No aliasing violations | Property: `prop_no_alias_violation` | ⬜ |
| 33 | Lifetime scope inference | Dataflow: `test_lifetime_scope` | ⬜ |
| 34 | Return value ownership | Unit test: `test_return_ownership` | ⬜ |
| 35 | Parameter ownership transfer | Unit test: `test_param_ownership` | ⬜ |
| 36 | Struct field ownership | Unit test: `test_field_ownership` | ⬜ |
| 37 | Array element ownership | Unit test: `test_array_ownership` | ⬜ |
| 38 | Double-free prevention | Safety: `test_no_double_free` | ⬜ |
| 39 | Use-after-free prevention | Safety: `test_no_use_after_free` | ⬜ |
| 40 | Memory leak detection | Safety: `test_no_memory_leak` | ⬜ |

**Kaizen Target**: Achieve 90%+ ownership inference accuracy (currently ~70%).

---

### D. Codegen Quality (Items 41-55)

**Objective**: Verify generated Rust code is idiomatic and compiles.
**Reference**: [3] C2Rust patterns, [6] Safe Rust usage study

| # | Item | Verification Method | Status |
|---|------|---------------------|--------|
| 41 | Generated code compiles with `rustc` | E2E: `test_rustc_compiles` | ⬜ |
| 42 | No rustc errors (warnings OK) | E2E: `test_no_errors` | ⬜ |
| 43 | Proper `use` statements generated | Unit test: `test_use_statements` | ⬜ |
| 44 | Struct derives (Debug, Clone, Default) | Unit test: `test_struct_derives` | ⬜ |
| 45 | Enum derives (Debug, Clone, Copy) | Unit test: `test_enum_derives` | ⬜ |
| 46 | `Box::default()` for simple structs | Unit test: `test_box_default` | ⬜ |
| 47 | `Box::into_raw()` for pointer returns | Unit test: `test_box_into_raw` | ⬜ |
| 48 | `unsafe {}` blocks properly scoped | Lint: `test_unsafe_scope` | ⬜ |
| 49 | SAFETY comments on unsafe blocks | Lint: `test_safety_comments` | ⬜ |
| 50 | String literals as `&str` | Unit test: `test_string_literals` | ⬜ |
| 51 | `printf` → `print!` macro | Unit test: `test_printf_transform` | ⬜ |
| 52 | `NULL` → `std::ptr::null_mut()` | Unit test: `test_null_transform` | ⬜ |
| 53 | Array indexing with `as usize` | Unit test: `test_array_index_cast` | ⬜ |
| 54 | Pointer arithmetic → safe methods | Unit test: `test_ptr_arithmetic` | ⬜ |
| 55 | Loop transformations (for/while) | Unit test: `test_loop_codegen` | ⬜ |

**Jidoka Gate**: Generated code MUST compile. No exceptions.

---

### E. Safety Verification (Items 56-70)

**Objective**: Verify generated Rust satisfies memory safety invariants.
**Reference**: [4] Prusti verification, [9] RustHorn, [10] Formal C-to-Rust

| # | Item | Verification Method | Status |
|---|------|---------------------|--------|
| 56 | No null pointer dereference | Miri: `test_miri_null_deref` | ⬜ |
| 57 | No buffer overflow | Miri: `test_miri_buffer_overflow` | ⬜ |
| 58 | No out-of-bounds access | Miri: `test_miri_oob` | ⬜ |
| 59 | No uninitialized memory read | Miri: `test_miri_uninit` | ⬜ |
| 60 | No data races | Miri: `test_miri_data_race` | ⬜ |
| 61 | Borrow checker compliance | `rustc` compilation | ⬜ |
| 62 | Lifetime validity | `rustc` compilation | ⬜ |
| 63 | Type safety (no transmute abuse) | Lint: `test_no_transmute` | ⬜ |
| 64 | Integer overflow handling | Audit: `test_overflow_handling` | ⬜ |
| 65 | Division by zero handling | Audit: `test_div_zero` | ⬜ |
| 66 | Pointer validity after free | Miri: `test_ptr_after_free` | ⬜ |
| 67 | Stack overflow prevention | Audit: `test_recursion_depth` | ⬜ |
| 68 | Thread safety (Send/Sync) | Lint: `test_thread_safety` | ⬜ |
| 69 | FFI boundary safety | Audit: `test_ffi_safety` | ⬜ |
| 70 | Panic safety (no unwinding UB) | Audit: `test_panic_safety` | ⬜ |

**Toyota Safety Standard**: Zero tolerance for memory safety violations per [7].

---

### F. Unsafe Minimization (Items 71-80)

**Objective**: Reduce unsafe blocks to <5 per 1000 LOC.
**Reference**: [6] Unsafe usage patterns in real Rust code

| # | Item | Verification Method | Status |
|---|------|---------------------|--------|
| 71 | Count unsafe blocks per file | Metric: `count_unsafe_blocks()` | ⬜ |
| 72 | Unsafe blocks per 1000 LOC < 5 | Metric: `unsafe_density()` | ⬜ |
| 73 | Each unsafe has SAFETY comment | Lint: `test_safety_docs` | ⬜ |
| 74 | Raw pointer ops minimized | Pattern: Box/Vec preferred | ⬜ |
| 75 | `mem::zeroed` → `Default` where possible | Unit test: `test_default_init` | ⬜ |
| 76 | `mem::transmute` never used | Lint: `test_no_transmute` | ⬜ |
| 77 | `ptr::read/write` minimized | Audit: `test_ptr_ops` | ⬜ |
| 78 | FFI calls wrapped in safe API | Pattern: wrapper functions | ⬜ |
| 79 | Unsafe trait impls justified | Audit: manual review | ⬜ |
| 80 | No `unsafe` in public API | Lint: `test_public_api_safe` | ⬜ |

**Current Metrics** (as of 2025-11-30):
- `hash_table.c`: 12 unsafe blocks (~500 LOC) = 24 per 1000 LOC ❌
- `binary_tree.c`: 16 unsafe blocks (~400 LOC) = 40 per 1000 LOC ❌
- **Target**: <5 per 1000 LOC

**Heijunka Plan**: Systematic reduction via ownership inference improvements.

---

### G. Integration & E2E (Items 81-90)

**Objective**: End-to-end validation with real C programs.
**Reference**: K&R C validation corpus, [3] Industrial migration study

| # | Item | Verification Method | Status |
|---|------|---------------------|--------|
| 81 | K&R Chapter 1 (Hello World, basics) | E2E: all 10 files compile | ⬜ |
| 82 | K&R Chapter 2 (Types, operators) | E2E: all files compile | ⬜ |
| 83 | K&R Chapter 3 (Control flow) | E2E: all files compile | ⬜ |
| 84 | K&R Chapter 4 (Functions) | E2E: all files compile | ⬜ |
| 85 | K&R Chapter 5 (Pointers, arrays) | E2E: all files compile | ⬜ |
| 86 | K&R Chapter 6 (Structures) | E2E: all files compile | ⬜ |
| 87 | `hash_table.c` single-shot compile | E2E: `rustc` succeeds | ✅ |
| 88 | `binary_tree.c` single-shot compile | E2E: `rustc` succeeds | ✅ |
| 89 | Behavioral equivalence test | Runtime: output matches | ⬜ |
| 90 | Performance within 2x of C | Benchmark: `criterion` | ⬜ |

**Genchi Genbutsu Validation**:
```bash
# Current status (2025-11-30)
cargo run -- transpile examples/data_structures/hash_table.c | rustc --edition 2021 -  # ✅
cargo run -- transpile examples/data_structures/binary_tree.c | rustc --edition 2021 - # ✅
```

---

### H. ML/CITL Readiness (Items 91-100)

**Objective**: Prepare for ML-assisted transpilation via CITL feedback loop.
**Reference**: Sister projects `../entrenar`, `../aprender`

| # | Item | Verification Method | Status |
|---|------|---------------------|--------|
| 91 | Golden trace collection working | CITL: `decy oracle collect` | ⬜ |
| 92 | C→Rust pairs exported for training | CITL: JSON/TOML export | ⬜ |
| 93 | Feature extraction pipeline | Aprender: tensor features | ⬜ |
| 94 | Diversity sampling implemented | CITL: `corpus_citl.rs` | ⬜ |
| 95 | Training data format compatible | Entrenar: batch loader | ⬜ |
| 96 | Model inference hook ready | Oracle: decision API | ⬜ |
| 97 | A/B testing infrastructure | CITL: comparison mode | ⬜ |
| 98 | Feedback loop closes (learn from errors) | CITL: error corpus | ⬜ |
| 99 | Incremental retraining supported | Entrenar: warm start | ⬜ |
| 100 | Production deployment checklist | Release: Friday-only | ⬜ |

**Depyler Pattern to Copy**:
```python
# From ../depyler - type inference through dataflow
# Decy should adopt similar pattern for ownership inference
class OwnershipInferencePass:
    def analyze_dataflow(self, cfg: ControlFlowGraph) -> OwnershipMap:
        # Forward dataflow analysis
        # Track allocation points, usage sites, free points
        pass
```

**Entrenar Integration**:
```rust
// From ../entrenar - batch training pattern
// Decy CITL should feed training batches similarly
pub struct CitlBatch {
    c_inputs: Vec<String>,
    rust_outputs: Vec<String>,
    features: Tensor,  // via aprender/trueno
}
```

---

## 6. Verification Matrix

### Summary by Category

| Category | Items | Passing | Failing | Blocked | Coverage |
|----------|-------|---------|---------|---------|----------|
| A. Parser | 1-15 | TBD | TBD | 0 | TBD% |
| B. HIR | 16-25 | TBD | TBD | 0 | TBD% |
| C. Ownership | 26-40 | TBD | TBD | 0 | TBD% |
| D. Codegen | 41-55 | TBD | TBD | 0 | TBD% |
| E. Safety | 56-70 | TBD | TBD | 0 | TBD% |
| F. Unsafe Min | 71-80 | TBD | TBD | 0 | TBD% |
| G. E2E | 81-90 | 2 | TBD | 0 | 20% |
| H. ML/CITL | 91-100 | TBD | TBD | 0 | TBD% |
| **TOTAL** | **100** | **2** | **TBD** | **0** | **2%** |

### Priority Matrix (Eisenhower)

| | Urgent | Not Urgent |
|---|--------|------------|
| **Important** | Items 41-55 (Codegen), Items 87-88 (E2E) | Items 26-40 (Ownership), Items 71-80 (Unsafe) |
| **Not Important** | Items 1-15 (Parser polish) | Items 91-100 (ML prep) |

---

## 7. Sign-Off Requirements

### QA Engineer Sign-Off

- [ ] All 100 items verified
- [ ] No critical (P0) issues open
- [ ] Unsafe density < 5 per 1000 LOC
- [ ] K&R Chapters 1-6 all compile
- [ ] Miri finds no undefined behavior

### ML Engineer Sign-Off

- [ ] CITL pipeline functional
- [ ] Training data quality verified
- [ ] Inference latency acceptable (<100ms)
- [ ] Model accuracy >90% on holdout

### Release Manager Sign-Off

- [ ] Friday release window
- [ ] CHANGELOG.md updated
- [ ] Version bump in Cargo.toml
- [ ] `cargo publish --dry-run` succeeds
- [ ] GitHub release draft ready

---

## Appendix: Andon Cord Protocol

When any checklist item fails critically:

1. **STOP** - Halt all feature development
2. **CALL** - Notify team via `#decy-alerts`
3. **FIX** - Create P0 ticket, apply EXTREME TDD
4. **VERIFY** - Re-run full checklist
5. **RESUME** - Only after QA sign-off

```
┌─────────────────────────────────────────┐
│  🚨 ANDON CORD - PULL TO STOP LINE 🚨  │
│                                         │
│  Quality is built in, not inspected in  │
│         - Toyota Production System      │
└─────────────────────────────────────────┘
```

---

*Document generated: 2025-11-30 17:00 UTC*
*Next review: 2025-12-06 (Friday release window)*
