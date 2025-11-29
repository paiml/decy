# DECY Unified Specification

**Version**: 2.0.0
**Status**: ACTIVE
**Date**: 2025-11-29
**Project**: C-to-Rust Transpiler with Unsafe Minimization

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Architecture](#2-architecture)
3. [Purification Philosophy](#3-purification-philosophy)
4. [Unsafe Minimization Strategy](#4-unsafe-minimization-strategy)
5. [Translation Techniques](#5-translation-techniques)
6. [Oracle System](#6-oracle-system)
7. [Standard Library Support](#7-standard-library-support)
8. [Testing Methodology](#8-testing-methodology)
9. [Quality Standards](#9-quality-standards)
10. [Development Workflow](#10-development-workflow)

---

## 1. Executive Summary

### 1.1 Project Vision (Pivot: AI-First)

DECY is a **data-centric engine for training High-Quality C-to-Rust LLMs**. While it functions as a transpiler, its primary strategic goal is to generate high-quality, verified "Golden Traces" (C -> Safe Rust pairs) to train a dedicated model ASAP. We aim to automate the purification process not just through heuristics, but by training a model that "intuits" safety and idioms, accelerating the global transition from C to Rust.

### 1.2 Core Principles

| Principle | Description |
|-----------|-------------|
| **Data Factory** | The pipeline's primary output is training data, not just code |
| **Model-First** | Manual heuristics exist to bootstrap the Model, then are retired |
| **Purification over Translation** | Train the model to "refactor" C, not just translate it |
| **Safety First** | The Model must learn to eliminate bug classes (Poka-Yoke) |
| **Toyota Way** | Jidoka (Automated quality), Muda (Eliminate manual rules) |

### 1.3 Key Metrics

| Metric | Target | Current |
|--------|--------|---------|
| **Model BLEU/CodeBLEU** | > 40.0 | 0.0 |
| **Golden Traces Generated** | 10,000+ | 0 |
| Unsafe blocks per 1000 LOC | <5 | 0 |
| Test coverage | ≥85% (90% for ownership) | 89.61% |
| Compilation success rate | 100% | 100% |

### 1.4 Safety Guarantees

The Trained Model must learn to eliminate these C bug classes:

| Bug Class | C Frequency | Target Model Behavior |
|-----------|-------------|-----------------------|
| NULL pointer dereference | ~40% of CVEs | Predict `Option<T>` usage |
| Use-after-free | ~30% of CVEs | Predict Ownership/Drop patterns |
| Buffer overflow | ~20% of CVEs | Predict `Vec` & slice patterns |
| Memory leak | Common | Predict RAII patterns |
| Data race | Common | Predict `Mutex`/`Arc` patterns |

---

## 2. Architecture

### 2.1 Six-Stage Pipeline (The Data Factory)

The pipeline acts as the **Teacher** for the model. It deterministically generates high-quality Rust to serve as Ground Truth (Golden Traces).

```
                                    ┌───────────────┐
                                    │  Model Train  │◄──────┐
                                    └───────┬───────┘       │
                                            │ Weights       │
C Source → Parser → HIR → Analyzer → [Hybrid Engine] → Verify → Rust Output
            │          (Heuristics + Model)      │            │
            └────────────────────────────────────┴──► Golden Traces
                                                       (Training Data)
```

Each stage contributes to generating clean training data:

| Stage | Crate | Responsibility | Data Generation Role |
|-------|-------|----------------|----------------------|
| 1. Parse | `decy-parser` | C → AST | Input features |
| 2. Lower | `decy-hir` | AST → High-level IR | Canonical representation |
| 3. Analyze | `decy-analyzer` | Control/Data flow | Labeling context |
| 4. Ownership | `decy-ownership` | Ptr classification | **Key Label**: Ownership types |
| 5. Verify | `decy-verify` | Safety validation | **Quality Gate**: Only pass safe code to dataset |
| 6. Generate | `decy-codegen` | HIR → Rust code | **Target Label**: Idiomatic Rust |

### 2.2 Supporting Crates

| Crate | Purpose |
|-------|---------|
| `decy-core` | Pipeline orchestration |
| `decy-stdlib` | Built-in C standard library prototypes |
| `decy-oracle` | **AI Trainer**: Manages dataset & fine-tuning loop |
| `decy-book` | mdBook verification |
| `decy-debugger` | Pipeline introspection & **Trace Inspection** |

### 2.3 Data Flow

```rust
// Pipeline data types
C Source (&str)
    → CParser::parse() → CAst
    → HirBuilder::lower() → Hir
    → Analyzer::analyze() → AnalysisResults {
          control_flow: ControlFlowGraph,
          data_flow: DataFlowGraph,
          lock_bindings: LockMapping,
          tagged_unions: Vec<TaggedUnion>,
          output_params: OutputParamMap,
      }
    → OwnershipInference::infer() → OwnershipMap + LifetimeMap
    → Verifier::verify() → VerificationResult
    → CodeGen::generate() → RustCode
```

---

## 3. Purification Philosophy

### 3.1 Translation vs Purification

**Translation** (what simple transpilers do):
```c
// C code
int* data = malloc(100 * sizeof(int));
if (data == NULL) return -1;
free(data);
```
```rust
// Simple translation - still unsafe!
let data: *mut i32 = unsafe {
    libc::malloc(100 * std::mem::size_of::<i32>()) as *mut i32
};
if data.is_null() { return -1; }
unsafe { libc::free(data as *mut libc::c_void); }
```

**Purification** (DECY's approach):
```rust
// Purified - safe, idiomatic Rust
let data: Vec<i32> = vec![0; 100];
// No NULL checks needed
// No manual free needed
// Bounds checking automatic
// Memory safety guaranteed
```

### 3.2 Purification Patterns

| C Pattern | Purified Rust | Safety Improvement |
|-----------|--------------|-------------------|
| `malloc/free` | `Box::new()` / Drop | No use-after-free |
| `calloc(n, size)` | `vec![0; n]` | Bounds checked |
| `int* ptr = NULL` | `Option<Box<i32>>` | Compiler-enforced checks |
| `pthread_mutex_t` | `Mutex<T>` | RAII, no forgotten unlocks |
| `enum + union` | `enum Value { ... }` | Type-safe variants |
| Output params | `Result<T>` / `Option<T>` | No sentinel values |

### 3.3 Example: Linked List

**C (unsafe)**:
```c
struct Node {
    int data;
    struct Node* next;
};

struct Node* create_node(int data) {
    struct Node* node = malloc(sizeof(struct Node));
    if (node == NULL) return NULL;
    node->data = data;
    node->next = NULL;
    return node;
}

void free_list(struct Node* head) {
    while (head != NULL) {
        struct Node* next = head->next;
        free(head);
        head = next;
    }
}
```

**Purified Rust**:
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    data: i32,
    next: Option<Box<Node>>,
}

impl Node {
    pub fn new(data: i32) -> Self {
        Self { data, next: None }
    }
}

// No free_list needed - Drop handles it automatically!
```

---

## 4. Unsafe Minimization Strategy

### 4.1 Four-Phase Approach

```
Phase 1: Pattern-Based     (100% → 50% unsafe)
    ↓
Phase 2: Ownership Inference (50% → 20% unsafe)
    ↓
Phase 3: Lifetime Inference  (20% → 10% unsafe)
    ↓
Phase 4: Safe Wrappers       (10% → <5% unsafe)
```

### 4.2 Phase 1: Pattern Detection

Detect common C patterns and generate safe Rust:

| Pattern | Detection | Transformation |
|---------|-----------|----------------|
| `malloc(sizeof(T))` | Single allocation | `Box::new(T::default())` |
| `malloc(n * sizeof(T))` | Array allocation | `Vec::with_capacity(n)` |
| `calloc(n, sizeof(T))` | Zero-initialized array | `vec![0; n]` |
| `free(ptr)` | Deallocation | Remove (RAII handles) |

### 4.3 Phase 2: Ownership Inference

Classify every pointer:

```rust
pub enum PointerOwnership {
    Owning,           // malloc/free → Box<T>
    ImmutableBorrow,  // const T* → &T
    MutableBorrow,    // T* (mutation) → &mut T
    HeapArray,        // array allocation → Vec<T>
    Unknown,          // fallback to raw pointer
}
```

**Inference Algorithm**:
1. Build dataflow graph tracking pointer assignments
2. Detect allocation/deallocation patterns
3. Analyze usage (reads vs writes)
4. Classify each pointer
5. Validate borrow checker rules

### 4.4 Phase 3: Lifetime Inference

Map C variable scopes to Rust lifetimes:

```c
// C scope
void process(const int* data) {
    int local = *data;  // data borrowed for function scope
}
```

```rust
// Rust with inferred lifetime (elision applies)
fn process(data: &i32) {
    let local = *data;
}
```

For complex cases, generate explicit lifetime annotations:
```rust
fn choose<'a>(flag: bool, a: &'a i32, b: &'a i32) -> &'a i32
```

### 4.5 Phase 4: Safe Wrappers

Wrap remaining unsafe in safe abstractions with SAFETY comments:

```rust
/// Safe wrapper for FFI function
///
/// SAFETY: The underlying C function is thread-safe and
/// the input buffer is guaranteed valid for the function duration.
pub fn safe_ffi_call(input: &[u8]) -> Result<Vec<u8>> {
    let mut output = vec![0u8; MAX_OUTPUT];
    let len = unsafe {
        // SAFETY: output buffer is valid for MAX_OUTPUT bytes
        ffi_function(input.as_ptr(), input.len(), output.as_mut_ptr())
    };
    output.truncate(len);
    Ok(output)
}
```

---

## 5. Translation Techniques

### 5.1 Scalar Pointer Replacement

**Two-phase analysis** (from research):

**Phase 1 - Ownership Classification**:
```c
// Pattern 1: Owning pointer
int* p = malloc(sizeof(int)); → Box<i32>

// Pattern 2: Immutable borrow
void print(const int* p) → fn print(p: &i32)

// Pattern 3: Mutable borrow
void increment(int* p) { (*p)++; } → fn increment(p: &mut i32)

// Pattern 4: Array allocation
int* arr = malloc(10 * sizeof(int)); → Vec<i32>
```

**Phase 2 - Lifetime Computation**:
```rust
// Simple case (elision applies)
fn get_first(arr: &[i32]) -> &i32

// Complex case (explicit lifetimes)
fn choose<'a, 'b>(a: &'a i32, b: &'b i32, flag: bool) -> &'a i32
```

### 5.2 Lock API Translation

**C** (manual lock/unlock):
```c
pthread_mutex_t lock;
int shared_data;

void modify() {
    pthread_mutex_lock(&lock);
    shared_data++;
    pthread_mutex_unlock(&lock);
}
```

**Rust** (RAII guard):
```rust
struct State {
    shared_data: Mutex<i32>,
}

impl State {
    fn modify(&self) {
        let mut data = self.shared_data.lock().unwrap();
        *data += 1;
    } // Guard dropped, lock released
}
```

### 5.3 Tagged Union Conversion

**C** (unsafe tag + union):
```c
enum TypeTag { INT, FLOAT };
struct Value {
    enum TypeTag tag;
    union { int as_int; float as_float; } data;
};
```

**Rust** (type-safe enum):
```rust
enum Value {
    Int(i32),
    Float(f64),
}
```

### 5.4 Output Parameter Elimination

**C** (output via pointer):
```c
int parse(const char* str, int* result) {
    *result = atoi(str);
    return 0;  // success
}
```

**Rust** (return Result):
```rust
fn parse(s: &str) -> Result<i32, ParseError> {
    s.parse()
}
```

### 5.5 Array Parameter Detection

Detect `(ptr, len)` pairs and convert to slices:

```c
void process(int* arr, size_t len) {
    for (size_t i = 0; i < len; i++) {
        arr[i] *= 2;
    }
}
```

```rust
fn process(arr: &mut [i32]) {
    for elem in arr.iter_mut() {
        *elem *= 2;
    }
}
```

---

## 6. Oracle System & Model Training

### 6.1 Overview (AI-Driven Translation)

The DECY Oracle is no longer just an error fixer; it is the **central nervous system** for training the `decy-model`. It manages the lifecycle of data generation, model training, and model inference.

### 6.2 Architecture

```
┌────────────────────── Data Factory ─────────────────────────┐
│                                                             │
│ C Corpus ──► [Decy Pipeline] ──► [Golden Traces (JSONL)]    │
│                                          │                  │
└──────────────────────────────────────────┼──────────────────┘
                                           ▼
┌────────────────────── Model Training ───────────────────────┐
│                                                             │
│ [Base LLM] ──► [Fine-Tuning] ◄── [Verified Golden Traces]   │
│                      │                                      │
│                      ▼                                      │
│                [Decy Model] ──────────────────┐             │
│                                               │             │
└───────────────────────────────────────────────┼─────────────┘
                                                ▼
┌───────────────────── Inference ─────────────────────────────┐
│                                                             │
│ C Source ──► [Decy Model] ──► Rust Candidate ──► [Verifier] │
│                                                     │       │
│                 (Kaizen Loop) ◄─── Success/Fail ────┘       │
└─────────────────────────────────────────────────────────────┘
```

### 6.3 Golden Trace Structure (Training Data)

The pipeline emits training examples in a standardized format:

```rust
pub struct GoldenTrace {
    pub c_snippet: String,       // Input C
    pub c_context: Context,      // Types, headers
    pub rust_snippet: String,    // Output Rust (Target)
    pub safety_explanation: String, // CoT (Chain of Thought) for the model
    pub metadata: TraceMetadata, // Complexity, constructs used
}
```

### 6.4 Training Workflow (Model-in-the-Loop)

1. **Cold Start**: Use heuristics (Phases 1-4) to generate initial corpus.
2. **Filter**: `decy-verify` ensures only SAFE Rust enters the dataset.
3. **Train**: Fine-tune a code model (e.g., CodeLlama, StarCoder) on Golden Traces.
4. **Inference**: Deploy model to replace heuristic stages.
5. **Kaizen Loop**: Model failures are corrected by heuristics (or humans) and fed back as new traces.

### 6.5 Training Tiers

| Tier | Complexity | Examples |
|------|------------|----------|
| **P0** | Simple patterns | Type mismatches, missing derives |
| **P1** | I/O patterns | File handling, format strings |
| **P2** | Complex patterns | Ownership, lifetimes, concurrency |

### 6.6 Bootstrap Patterns

25 seed patterns for cold start:

| Error Code | Pattern | Fix |
|------------|---------|-----|
| E0308 | `*mut T` vs `&mut T` | Add dereference or borrow |
| E0133 | Missing `unsafe` block | Wrap in safe abstraction |
| E0382 | Use after move | Clone or restructure |
| E0499 | Multiple mutable borrows | Use `RefCell` or restructure |
| E0506 | Assign to borrowed | Restructure lifetimes |
| E0515 | Return local reference | Return owned value |
| E0597 | Value doesn't live long enough | Extend lifetime or clone |

### 6.7 CLI Commands

```bash
# Query oracle for pattern
decy oracle query --error E0308 --context "let x: &mut i32 = ptr"

# Train on corpus
decy oracle train --corpus ./training_corpus --tier P0

# Show statistics
decy oracle stats

# Export patterns
decy oracle export --format jsonl --output patterns.jsonl
```

---

## 7. Standard Library Support

### 7.1 Problem

C programs use `#include <stdlib.h>` for standard functions. The parser comments these out, causing "undeclared identifier" errors.

### 7.2 Solution: Built-in Prototypes

Inject function prototypes before parsing:

```
C Source
    ↓
Preprocess (comment out #include)
    ↓
Inject prototypes for detected headers
    ↓
Parse (stdlib functions now declared!)
```

### 7.3 Implemented Headers

| Header | Functions | Status |
|--------|-----------|--------|
| `<stdlib.h>` | malloc, free, calloc, realloc, atoi, exit... | ✅ 18 functions |
| `<stdio.h>` | printf, fprintf, fopen, fclose, fread... | ✅ 24 functions |
| `<string.h>` | strlen, strcpy, strcmp, memcpy, memset... | ✅ 20 functions |

### 7.4 Per-Header Filtering

Critical innovation: inject only functions from the specific header requested, not all 55+ at once:

```c
#include <string.h>  // Injects ONLY 20 string.h functions
```

This prevents parser overload and improves performance.

### 7.5 Transformation Rules

| C Function | Rust Equivalent |
|------------|-----------------|
| `strlen(s)` | `s.len()` |
| `strcmp(a, b)` | `a.cmp(b)` or `a == b` |
| `malloc(sizeof(T))` | `Box::new(T::default())` |
| `free(ptr)` | (removed - RAII) |
| `printf(fmt, ...)` | `println!(fmt, ...)` |

---

## 8. Testing Methodology

### 8.1 SQLite-Style Testing

Inspired by SQLite's 614:1 test-to-code ratio:

| Category | Target | Description |
|----------|--------|-------------|
| Unit tests | ≥5 per module | Component-level validation |
| Property tests | ≥3 per module | Mathematical invariants |
| Integration tests | All features | End-to-end transpilation |
| Torture tests | Edge cases | Pathological inputs |
| Regression tests | 100% of bugs | Every bug becomes a test |
| Differential tests | vs GCC/Clang | Compare behavior |
| Mutation tests | ≥90% kill rate | Test quality validation |

### 8.2 Three-Tier Testing Workflow

| Tier | Trigger | Time | Checks |
|------|---------|------|--------|
| **Tier 1** | On save | <1s | Fast unit tests, clippy, fmt |
| **Tier 2** | On commit | 1-5 min | Full tests, coverage, properties |
| **Tier 3** | On merge/nightly | 1-4 hrs | Mutation, Kani, Miri, fuzz |

### 8.3 CLI Contract Testing

Every CLI command requires contract tests:

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn cli_transpile_valid_file_exits_zero() {
    Command::cargo_bin("decy")
        .arg("transpile")
        .arg("test.c")
        .assert()
        .success();
}

#[test]
fn cli_transpile_invalid_syntax_exits_nonzero() {
    Command::cargo_bin("decy")
        .arg("transpile")
        .arg("bad.c")
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}
```

### 8.4 Property Tests

```rust
proptest! {
    #[test]
    fn prop_unique_owner_per_allocation(c_program in c_malloc_free()) {
        let ownership = infer_ownership(&c_program)?;
        for alloc in c_program.allocations() {
            prop_assert_eq!(ownership.owners(alloc).count(), 1);
        }
    }

    #[test]
    fn prop_borrows_dont_outlive_owner(c_program in c_with_pointers()) {
        let ownership = infer_ownership(&c_program)?;
        for borrow in ownership.borrows() {
            let owner = ownership.find_owner(borrow);
            prop_assert!(borrow.lifetime <= owner.lifetime);
        }
    }
}
```

### 8.5 Unsafe Auditing

```rust
#[test]
fn audit_unsafe_block_count() {
    let rust_code = transpile(include_str!("typical_program.c"))?;
    let unsafe_count = count_unsafe_blocks(&rust_code);
    let loc = count_lines(&rust_code);
    let unsafe_per_1000 = (unsafe_count as f64 / loc as f64) * 1000.0;

    assert!(unsafe_per_1000 < 5.0, "Exceeded unsafe limit");
}
```

---

## 9. Quality Standards

### 9.1 Zero Tolerance Policies

| Policy | Threshold | Enforcement |
|--------|-----------|-------------|
| Coverage | ≥80% (90% for ownership) | Pre-commit hook |
| Clippy warnings | 0 | `-D warnings` |
| SATD comments | 0 | grep check |
| Unsafe per 1000 LOC | <5 | Audit tests |
| Cyclomatic complexity | ≤10 | cargo-geiger |
| Cognitive complexity | ≤15 | clippy |

### 9.2 SATD Comment Ban

These comments are banned (enforced in pre-commit):
- `TODO`
- `FIXME`
- `HACK`
- `XXX`
- `TEMP`
- `WIP`
- `BROKEN`

### 9.3 Quality Gates

```bash
# Pre-commit checks (Tier 2)
make quality-gates

# Runs:
# 1. cargo fmt --check
# 2. cargo clippy -- -D warnings
# 3. cargo test --all
# 4. Coverage check (≥80%)
# 5. SATD comment check
```

### 9.4 Documentation Requirements

Every public item needs:

```rust
//! Module-level documentation

/// Function documentation
///
/// # Arguments
/// * `param` - Description
///
/// # Returns
/// Description
///
/// # Examples
/// ```
/// let result = function(input)?;
/// assert_eq!(result, expected);
/// ```
pub fn function(param: Type) -> Result<Output>
```

---

## 10. Development Workflow

### 10.1 PMAT Roadmap-Driven Development

All work is ticket-driven via `roadmap.yaml`:

```yaml
DECY-XXX:
  title: "Short title"
  status: not_started | in_progress | done
  phase: RED | GREEN | REFACTOR | DONE
  sprint: <number>
  priority: critical | high | medium | low
  type: feature | bug | refactor | quality | docs
```

### 10.2 EXTREME TDD Workflow

**RED Phase** (failing tests):
```bash
# Write tests that should fail
cargo test  # Fails as expected
git commit --no-verify -m "[RED] DECY-XXX: Add failing tests"
```

**GREEN Phase** (minimal implementation):
```bash
# Implement just enough to pass
cargo test  # Passes
git commit -m "[GREEN] DECY-XXX: Minimal implementation"
```

**REFACTOR Phase** (quality gates):
```bash
# Add docs, improve code, meet quality standards
make quality-gates  # Must pass
git commit -m "[REFACTOR] DECY-XXX: Meet quality gates"
```

**Final** (squash and close):
```bash
git rebase -i HEAD~3
git commit -m "DECY-XXX: Description

- Coverage: 87% ✅
- Mutation score: 91% ✅

Closes #XXX"
```

### 10.3 Release Policy

**Releases happen ONLY on Fridays**:

| Day | Activity |
|-----|----------|
| Mon-Thu | Prepare, test, dry-run |
| Friday | Release to crates.io |
| Sat-Sun | Emergency fixes only |

**Model Release Cycle**:
- **Nightly**: Retrain on new Golden Traces collected during the day.
- **Weekly**: Evaluate model against "Torture Tests" and BLEU benchmarks.

**Release Checklist**:
1. All quality gates pass
2. `cargo publish --dry-run` succeeds
3. CHANGELOG updated
4. Git tag created
5. GitHub release created

### 10.4 Toyota Way Principles

| Principle | Application |
|-----------|-------------|
| **Jidoka** (自働化) | Build quality in - never merge incomplete features |
| **Genchi Genbutsu** (現地現物) | Direct observation - test with real C code |
| **Kaizen** (改善) | Continuous improvement - fix bugs before features |
| **Hansei** (反省) | Reflection - sprint retrospectives |
| **Yokoten** (横展) | Share learnings - oracle pattern sharing |

### 10.5 STOP THE LINE Protocol (Andon Cord)

When validation reveals a bug:

1. **STOP** all feature development
2. **Create P0 ticket** with C99/K&R reference
3. **Apply EXTREME TDD** fix
4. **Verify** unsafe count didn't increase
5. **Resume** only after fix verified

---

## Appendix A: Reference Standards

### A.1 C Standards

- **ISO C99**: ISO/IEC 9899:1999
- **K&R C**: Kernighan & Ritchie, 2nd Edition

### A.2 Rust Guidelines

- **Rust API Guidelines**: https://rust-lang.github.io/api-guidelines/
- **Unsafe Code Guidelines**: https://rust-lang.github.io/unsafe-code-guidelines/

### A.3 Research Papers

1. **CACM**: "Automatically Translating C to Rust" (2024)
2. **Zhang et al.**: Ownership identification
3. **Emre et al.**: Lifetime computation
4. **RustBelt**: Formal verification of Rust's type system

---

## Appendix B: File Locations

| File | Purpose |
|------|---------|
| `roadmap.yaml` | PMAT ticket tracking |
| `decy-quality.toml` | Quality thresholds |
| `docs/C-VALIDATION-ROADMAP.yaml` | C99 validation north star |
| `crates/decy-*/` | Core crates |
| `tests/cli_contract_*.rs` | CLI contract tests |

---

## Appendix C: Commands Reference

```bash
# Build and test
make build              # Build workspace
make test               # Run all tests
make quality-gates      # Pre-commit checks

# Development
make coverage           # Generate coverage report
cargo test -p decy-XXX  # Test specific crate

# Oracle
decy oracle stats       # Show pattern statistics
decy oracle train       # Train on corpus
decy oracle query       # Query for fix

# Release (Fridays only)
cargo publish -p decy   # Publish to crates.io
```

---

## Appendix D: Toyota Way Kaizen Review Log

*Review Context: Shift from Heuristic-Only to Model-Training Focus*

| # | Principle | Annotation / Critique | Action Taken |
|---|-----------|-----------------------|--------------|
| 1 | **Vision** | The goal was too static. A static transpiler is muda (waste) compared to a learning model. | Pivot vision to "Data Factory" & "Model Training". |
| 2 | **Jidoka** | Manual heuristics are prone to error. Automation should include *learning* from errors, not just fixing them. | Oracle redefined as "Model Training Engine". |
| 3 | **Muda** | Manual `stdlib` prototypes are waste. We should learn these mappings from data. | Annotated `stdlib` section as a candidate for future learning. |
| 4 | **Genchi Genbutsu** | Testing on synthetic code is insufficient for training a robust model. | Integrated "Golden Traces" from real corpora into the pipeline. |
| 5 | **Standardization** | `ErrorPattern` was too specific. We need a standard `GoldenTrace` format for LLM training. | Introduced `GoldenTrace` struct in Spec 6.3. |
| 6 | **Visual Control** | No visibility into model performance (BLEU, etc.). | Added Model Metrics to Section 1.3. |
| 7 | **Poka-Yoke** | The model will hallucinate. We need a hard guardrail. | Emphasized `decy-verify` as the Poka-Yoke (mistake-proofing) gate. |
| 8 | **Flow** | The previous flow was linear. Training requires a loop. | Updated Architecture (2.1) to show the circular Training Loop. |
| 9 | **Respect for People** | Don't make devs write repetitive rules. Let the model learn them. | Shifted focus to "Data Centric" dev workflow. |
| 10 | **Kaizen** | The ultimate improvement is a model that improves itself. | Added "Kaizen Loop" to the Inference architecture. |

---

**END OF UNIFIED SPECIFICATION**

*This document consolidates: decy-spec-v1.md, training-oracle-spec.md, oracle-integration-spec.md, decy-unsafe-minimization-strategy.md, header-support-spec.md, testing-sqlite-style.md, purify-c-spec.md, improve-testing-quality-using-certeza-concepts.md, and translation-ideas-spec.md*
