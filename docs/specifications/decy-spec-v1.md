# Decy: C-to-Rust Transpiler Specification v1.0

**Project**: Decy - Production-Grade C-to-Rust Transpiler
**Version**: 1.0
**Date**: 2025-10-10
**Methodology**: EXTREME TDD + Toyota Way + PMAT Quality Gates
**Inspired By**: depyler (Python→Rust), bashrs (Rust→Shell), paiml-mcp-agent-toolkit (Quality Framework)

---

## Executive Summary

Decy is a production-grade C-to-Rust transpiler designed to automatically convert legacy C codebases (Python language source, Git source, NumPy, etc.) into safe, idiomatic, linted, and fully-tested Rust code. The transpiler follows EXTREME TDD methodology with:

- **80%+ test coverage** maintained at ALL times
- **≥90% mutation testing score** as target
- **100% linting passing** continuously
- **Zero tolerance for SATD** (technical debt comments)
- **Property-based testing** for correctness guarantees
- **Book-based verification** similar to depyler's approach
- **PMAT-qualified development** with roadmap-driven tickets only

---

## 1. Architecture Overview

### 1.1 Multi-Stage Pipeline (Inspired by depyler)

```
C Source Code
    ↓
┌─────────────────────────────┐
│  Stage 1: C Parser          │
│  - clang AST integration    │
│  - Preprocessor handling    │
│  - Macro expansion          │
└─────────────────────────────┘
    ↓
┌─────────────────────────────┐
│  Stage 2: HIR Generation    │
│  - High-level IR creation   │
│  - Type inference           │
│  - Ownership analysis       │
└─────────────────────────────┘
    ↓
┌─────────────────────────────┐
│  Stage 3: Safety Verification│
│  - Memory safety checks     │
│  - Borrow checker simulation│
│  - Unsafe block minimization│
└─────────────────────────────┘
    ↓
┌─────────────────────────────┐
│  Stage 4: Rust Code Gen     │
│  - Idiomatic Rust output    │
│  - Clippy compliance        │
│  - Rustfmt application      │
└─────────────────────────────┘
    ↓
┌─────────────────────────────┐
│  Stage 5: Book Verification │
│  - Compile generated code   │
│  - Run Rust tests           │
│  - Lint verification        │
│  - Behavior equivalence     │
└─────────────────────────────┘
    ↓
Verified Rust Code
```

### 1.2 Core Crates Structure

```
decy/
├── crates/
│   ├── decy-core/          # Main transpilation pipeline
│   ├── decy-parser/        # C AST parsing (clang bindings)
│   ├── decy-hir/           # High-level Intermediate Representation
│   ├── decy-analyzer/      # Static analysis & type inference
│   ├── decy-ownership/     # Ownership & lifetime inference
│   ├── decy-verify/        # Safety property verification
│   ├── decy-codegen/       # Rust code generation
│   ├── decy-book/          # Book-based verification system
│   ├── decy-agent/         # Background daemon (like depyler agent)
│   ├── decy-mcp/           # MCP server for Claude Code
│   └── decy/               # CLI binary
├── docs/
│   ├── specifications/     # This file
│   ├── architecture/       # Architecture Decision Records
│   ├── quality/            # Quality standards & gates
│   └── execution/          # Roadmap & sprint planning
├── tests/
│   ├── integration/        # End-to-end transpilation tests
│   ├── property/           # Property-based tests
│   └── fixtures/           # C test cases (Python, Git, NumPy sources)
└── book/                   # mdBook for verification docs
```

---

## 2. Quality Requirements (EXTREME TDD)

### 2.1 Zero-Tolerance Quality Gates

```toml
# decy-quality.toml (inspired by bashrs/pmat-quality.toml)

[complexity]
cyclomatic_threshold = 10      # Strict for safety-critical code
cognitive_threshold = 15       # Maintainability requirement
max_nesting_depth = 4          # Shallow nesting for clarity
max_function_lines = 80        # Testable function size

[satd]
enabled = true
zero_tolerance = true          # NO TODO/FIXME/HACK allowed
patterns = ["TODO", "FIXME", "HACK", "XXX", "WORKAROUND"]

[coverage]
minimum_coverage = 80.0        # ENFORCED at ALL times
enforce_on_new_code = true
target_core_modules = [
    "decy-parser/",
    "decy-hir/",
    "decy-analyzer/",
    "decy-ownership/",
    "decy-codegen/"
]

[mutation_testing]
enabled = true
minimum_kill_rate = 0.90       # ≥90% target
timeout_seconds = 120
target_modules = ["parser", "hir", "analyzer", "ownership", "codegen"]

[property_testing]
enabled = true
minimum_properties = 100       # 100+ property tests required
cases_per_property = 1000      # 1000 cases each = 100K+ total tests

[verification]
enabled = true
require_clippy_pass = true
require_rustfmt_pass = true
require_cargo_test_pass = true
require_book_compilation = true

[security]
max_unsafe_blocks = 5          # Minimize unsafe code
check_memory_safety = true
check_data_races = true
enforce_borrow_rules = true
```

### 2.2 Continuous Quality Enforcement

**Pre-commit Hooks** (block violations):
- Coverage drops below 80%
- Linting failures (clippy/rustfmt)
- SATD comments detected
- Complexity violations (>10 cyclomatic, >15 cognitive)
- Tests failing

**CI/CD Pipeline** (fail PR):
- All quality gates must pass
- Mutation testing score ≥90%
- Property tests pass (100K+ cases)
- Book compilation succeeds
- Generated Rust code lints clean

**Quality Score Calculation**:
```
Score = 0.25 * complexity_score +
        0.25 * coverage_score +
        0.20 * mutation_score +
        0.15 * property_test_score +
        0.15 * linting_score

Grade: A+ (≥98), A (≥95), B+ (≥90), B (≥85)
Target: A+ (98+) at ALL times
```

---

## 3. Development Methodology

### 3.1 EXTREME TDD Workflow

**Every ticket follows RED-GREEN-REFACTOR**:

1. **RED Phase**: Write failing tests FIRST
   - Unit tests for specific functionality
   - Property tests for invariants
   - Integration tests for end-to-end
   - Commit failing tests with `[RED]` prefix

2. **GREEN Phase**: Minimal implementation to pass
   - Write just enough code to pass tests
   - No optimization, no extras
   - Commit passing tests with `[GREEN]` prefix

3. **REFACTOR Phase**: Clean up while maintaining green
   - Apply quality gates
   - Ensure coverage ≥80%
   - Verify linting passes
   - Run mutation tests
   - Commit refactored code with `[REFACTOR]` prefix

4. **VERIFY Phase**: Quality gate check
   - Run `./scripts/quality-gates.sh`
   - Verify all metrics pass
   - Update documentation
   - Single atomic commit per ticket

### 3.2 Toyota Way Principles

**自働化 (Jidoka) - Build Quality In**:
- Automated quality gates block bad commits
- EXTREME TDD enforced via tooling
- Zero defects policy (no SATD)
- Pre-commit hooks stop violations

**反省 (Hansei) - Reflection**:
- Five Whys analysis for every bug
- Root cause documentation required
- Sprint retrospectives mandatory
- Lessons learned captured

**改善 (Kaizen) - Continuous Improvement**:
- Weekly quality reviews
- Complexity reduction sprints
- Performance optimization cycles
- Metrics tracked per sprint

**現地現物 (Genchi Genbutsu) - Go and See**:
- Dogfooding on real C projects (Python, Git, NumPy)
- Test on actual codebases
- Profile real workloads
- Validate transpiled code runs correctly

---

## 4. Testing Strategy

### 4.1 Test Types & Requirements

**Unit Tests** (per-function testing):
- Every public function has ≥3 tests
- Happy path, error cases, edge cases
- Target: 85% unit test coverage

**Property Tests** (100+ properties):
```rust
// Example properties for C→Rust transpilation
proptest! {
    #[test]
    fn transpilation_is_deterministic(c_code in c_code_generator()) {
        let output1 = transpile(&c_code).unwrap();
        let output2 = transpile(&c_code).unwrap();
        prop_assert_eq!(output1, output2);
    }

    #[test]
    fn generated_rust_compiles(c_code in valid_c_code()) {
        let rust_code = transpile(&c_code).unwrap();
        prop_assert!(compile_rust(&rust_code).is_ok());
    }

    #[test]
    fn generated_rust_passes_clippy(c_code in valid_c_code()) {
        let rust_code = transpile(&c_code).unwrap();
        prop_assert!(clippy_passes(&rust_code));
    }

    #[test]
    fn memory_safety_preserved(c_code in memory_safe_c()) {
        let rust_code = transpile(&c_code).unwrap();
        prop_assert!(borrow_checker_passes(&rust_code));
    }

    #[test]
    fn behavior_equivalence(c_code in testable_c_code()) {
        let c_output = execute_c(&c_code).unwrap();
        let rust_code = transpile(&c_code).unwrap();
        let rust_output = execute_rust(&rust_code).unwrap();
        prop_assert_eq!(c_output, rust_output);
    }
}
```

**Mutation Tests** (≥90% kill rate):
- Use `cargo-mutants` on core modules
- Target: kill 90%+ of introduced mutations
- Focus on parser, HIR, analyzer, codegen

**Integration Tests** (end-to-end):
- Full transpilation pipeline tests
- Real C projects as fixtures:
  - Python language source code (cpython)
  - Git source code
  - NumPy source code
  - SQLite source code
- Verify: compiles, lints, tests pass, behavior matches

**Book Tests** (verification via mdBook):
```markdown
<!-- In book/src/verification/python.md -->
# Python Source Transpilation

## Test: Python dict implementation

```c
// Original C code from Python source
typedef struct {
    PyObject_HEAD
    Py_ssize_t ma_used;
    uint64_t ma_version_tag;
    PyDictKeysObject *ma_keys;
    PyObject **ma_values;
} PyDictObject;
```

Transpiled Rust:
```rust,no_run
struct PyDictObject {
    _head: PyObjectHead,
    ma_used: isize,
    ma_version_tag: u64,
    ma_keys: Box<PyDictKeysObject>,
    ma_values: Vec<*mut PyObject>,
}
```

Verification:
- ✅ Compiles with `cargo build`
- ✅ Passes clippy with `cargo clippy`
- ✅ Memory layout preserved
- ✅ Size matches: `assert_eq!(size_of::<PyDictObject>(), 48)`
```

### 4.2 Coverage Requirements

**Minimum Coverage by Module**:
- `decy-parser/`: ≥85% (safety-critical)
- `decy-hir/`: ≥85% (safety-critical)
- `decy-analyzer/`: ≥85% (safety-critical)
- `decy-ownership/`: ≥90% (most critical - memory safety)
- `decy-codegen/`: ≥85% (safety-critical)
- `decy-verify/`: ≥80% (verification logic)
- `decy-book/`: ≥75% (tooling)
- **Overall**: ≥80% ENFORCED

**Coverage Tracking**:
- Use `cargo llvm-cov` for fast, accurate coverage
- Pre-commit hook blocks coverage drops
- CI/CD generates coverage reports
- Coverage badge in README.md

---

## 5. C Parsing & AST Strategy

### 5.1 Clang Integration

**Approach**: Use `clang-sys` bindings to leverage production-grade C parser

```rust
// decy-parser/src/lib.rs
use clang::{Clang, Index, TranslationUnit};

pub struct CParser {
    clang: Clang,
    index: Index<'static>,
}

impl CParser {
    pub fn parse(&self, source: &str) -> Result<TranslationUnit, ParseError> {
        // Use clang to parse C source
        // Extract AST with preprocessor directives resolved
        // Handle:
        //   - Macros (expand before transpilation)
        //   - Include files (resolve dependencies)
        //   - Compiler-specific extensions (GCC, MSVC)
    }
}
```

**Test Requirements**:
- Unit tests for each C construct (struct, union, enum, typedef, etc.)
- Property tests for valid C code parsing
- Integration tests with real C projects
- Error handling tests for invalid C

### 5.2 Macro Handling Strategy

**Macros are the hardest C feature**:

1. **Simple Macros** (constants): Convert to Rust constants
   ```c
   #define MAX_SIZE 1024
   ```
   →
   ```rust
   const MAX_SIZE: usize = 1024;
   ```

2. **Function-like Macros**: Convert to inline functions
   ```c
   #define MIN(a, b) ((a) < (b) ? (a) : (b))
   ```
   →
   ```rust
   #[inline]
   fn min<T: Ord>(a: T, b: T) -> T {
       if a < b { a } else { b }
   }
   ```

3. **Complex Macros** (token pasting, stringification):
   - Use `macro_rules!` for direct translation
   - Generate proc macros for complex cases
   - Document limitations in book

4. **Conditional Compilation** (`#ifdef`):
   - Convert to Rust `cfg` attributes
   - Maintain platform-specific code paths

**Property Test Example**:
```rust
proptest! {
    #[test]
    fn macro_expansion_deterministic(macro_def in c_macro_generator()) {
        let expanded1 = expand_macro(&macro_def).unwrap();
        let expanded2 = expand_macro(&macro_def).unwrap();
        prop_assert_eq!(expanded1, expanded2);
    }
}
```

---

## 6. HIR (High-Level Intermediate Representation)

### 6.1 HIR Structure (Inspired by depyler-hir)

```rust
// decy-hir/src/lib.rs

/// High-level IR representing a C translation unit
pub struct HirModule {
    pub name: String,
    pub includes: Vec<HirInclude>,
    pub functions: Vec<HirFunction>,
    pub types: Vec<HirType>,
    pub globals: Vec<HirGlobal>,
}

/// Function representation with ownership annotations
pub struct HirFunction {
    pub name: String,
    pub params: Vec<HirParam>,
    pub ret_type: HirType,
    pub body: HirBlock,
    pub safety: SafetyLevel,
    pub ownership: OwnershipInfo,
}

/// Type system including Rust-specific types
pub enum HirType {
    // C primitive types
    Int { signed: bool, width: u8 },
    Float { width: u8 },
    Pointer { inner: Box<HirType>, mutability: Mutability },
    Array { inner: Box<HirType>, size: Option<usize> },

    // Rust safety types
    Reference { inner: Box<HirType>, lifetime: Lifetime, mutability: Mutability },
    Slice { inner: Box<HirType> },
    Option { inner: Box<HirType> },
    Result { ok: Box<HirType>, err: Box<HirType> },

    // Aggregate types
    Struct { name: String, fields: Vec<HirField> },
    Union { name: String, variants: Vec<HirField> },
    Enum { name: String, variants: Vec<HirVariant> },
}

/// Ownership information for memory safety
pub struct OwnershipInfo {
    pub owner: Option<String>,
    pub borrowers: Vec<Borrow>,
    pub lifetime: Lifetime,
    pub drops: Vec<DropPoint>,
}

/// Safety level for generated code
pub enum SafetyLevel {
    Safe,                    // Pure safe Rust
    SafeWithUnsafeImpl,      // Safe interface, unsafe internals
    Unsafe,                  // Requires unsafe block
}
```

**HIR Tests**:
- Unit tests for each HIR node construction
- Property tests for HIR transformations
- Invariant tests: HIR always type-safe, ownership-valid

---

## 7. Ownership & Lifetime Inference

### 7.1 Ownership Analysis (Most Critical Component)

**Goal**: Infer Rust ownership from C pointer usage

```rust
// decy-ownership/src/lib.rs

pub struct OwnershipAnalyzer {
    ownership_graph: Graph<Variable, OwnershipEdge>,
}

impl OwnershipAnalyzer {
    /// Analyze C function to infer ownership
    pub fn analyze(&mut self, func: &CFunction) -> Result<OwnershipInfo> {
        // 1. Build ownership graph from pointer operations
        // 2. Detect ownership patterns:
        //    - malloc/free → Box::new/drop
        //    - function param → &T or &mut T
        //    - return value → T (owned)
        //    - array access → &[T] or &mut [T]
        // 3. Infer lifetimes from pointer scopes
        // 4. Insert borrow checker constraints
        // 5. Verify no use-after-free, double-free, null deref
    }
}
```

**Ownership Patterns**:

| C Pattern | Rust Translation |
|-----------|------------------|
| `malloc()` + `free()` | `Box::new()` + automatic drop |
| Function param `T*` (read-only) | `&T` |
| Function param `T*` (modified) | `&mut T` |
| Return `T*` (caller frees) | `Box<T>` |
| Global `static T*` | `static ref` or `lazy_static!` |
| Array `T[]` | `Vec<T>` or `[T; N]` |
| Null pointer | `Option<Box<T>>` or `Option<&T>` |

**Property Tests for Ownership**:
```rust
proptest! {
    #[test]
    fn no_use_after_free(c_func in c_function_generator()) {
        let hir = parse_and_analyze(&c_func).unwrap();
        prop_assert!(verify_no_use_after_free(&hir));
    }

    #[test]
    fn no_double_free(c_func in c_function_generator()) {
        let hir = parse_and_analyze(&c_func).unwrap();
        prop_assert!(verify_no_double_free(&hir));
    }

    #[test]
    fn borrow_checker_passes(c_func in memory_safe_c()) {
        let rust_code = transpile(&c_func).unwrap();
        prop_assert!(compile_rust(&rust_code).is_ok());
    }
}
```

### 7.2 Lifetime Inference

**Algorithm**:
1. Compute variable scopes from C AST
2. Track pointer aliasing and dereferencing
3. Infer Rust lifetime annotations:
   - Local variables → no lifetime annotation (elided)
   - Function params → named lifetimes `'a`, `'b`
   - Struct fields with references → lifetime parameters
4. Insert lifetime bounds where needed
5. Verify lifetime soundness

**Example**:
```c
// C code
char* get_name(struct Person* p) {
    return p->name;
}
```
→
```rust
// Transpiled Rust with lifetime
fn get_name<'a>(p: &'a Person) -> &'a str {
    &p.name
}
```

**Tests**: 100+ property tests for lifetime inference correctness

---

## 8. Code Generation

### 8.1 Idiomatic Rust Generation

**Principles**:
1. **Safety First**: Minimize unsafe blocks
2. **Idioms**: Use Rust patterns (Result, Option, iterators)
3. **Linting**: Generated code must pass clippy
4. **Formatting**: Apply rustfmt automatically
5. **Documentation**: Generate doc comments from C comments

```rust
// decy-codegen/src/lib.rs

pub struct RustCodegen {
    options: CodegenOptions,
}

impl RustCodegen {
    pub fn generate(&self, hir: &HirModule) -> Result<String> {
        let mut code = String::new();

        // 1. Generate module header
        code.push_str(&self.generate_header(hir));

        // 2. Generate type definitions
        for ty in &hir.types {
            code.push_str(&self.generate_type(ty)?);
        }

        // 3. Generate functions
        for func in &hir.functions {
            code.push_str(&self.generate_function(func)?);
        }

        // 4. Apply rustfmt
        let formatted = format_rust(&code)?;

        // 5. Verify clippy passes
        if !clippy_passes(&formatted) {
            return Err(CodegenError::LintingFailed);
        }

        Ok(formatted)
    }
}
```

### 8.2 Unsafe Code Minimization

**Strategy**:
- Only use `unsafe` when absolutely necessary (raw pointers, FFI)
- Wrap unsafe code in safe abstractions
- Document safety invariants with `// SAFETY:` comments
- Target: ≤5 unsafe blocks in generated code per 1000 LOC

**Example**:
```c
// C code with raw pointer
void* get_data(struct Buffer* buf, size_t index) {
    return buf->data + index;
}
```
→
```rust
// Safe Rust with bounds checking
fn get_data(buf: &Buffer, index: usize) -> Option<&u8> {
    buf.data.get(index)
}
```

**Property Test**:
```rust
proptest! {
    #[test]
    fn generated_rust_minimizes_unsafe(c_code in valid_c_code()) {
        let rust_code = transpile(&c_code).unwrap();
        let unsafe_count = count_unsafe_blocks(&rust_code);
        prop_assert!(unsafe_count <= expected_unsafe_count(&c_code));
    }
}
```

---

## 9. Book-Based Verification (Inspired by depyler)

### 9.1 mdBook Structure

```
book/
├── book.toml
└── src/
    ├── SUMMARY.md
    ├── introduction.md
    ├── verification/
    │   ├── python/          # Python source transpilation
    │   │   ├── dict.md
    │   │   ├── list.md
    │   │   ├── interpreter.md
    │   │   └── gc.md
    │   ├── git/             # Git source transpilation
    │   │   ├── objects.md
    │   │   ├── refs.md
    │   │   └── packfiles.md
    │   ├── numpy/           # NumPy source transpilation
    │   │   ├── arrays.md
    │   │   ├── ufuncs.md
    │   │   └── linalg.md
    │   └── sqlite/          # SQLite source transpilation
    │       ├── btree.md
    │       ├── vdbe.md
    │       └── pager.md
    └── quality/
        ├── testing.md
        ├── coverage.md
        └── benchmarks.md
```

### 9.2 Verification Process

**For each C file**:
1. Include original C code
2. Show transpiled Rust code
3. Verify compilation: `cargo build`
4. Verify linting: `cargo clippy`
5. Run tests: `cargo test`
6. Benchmark performance comparison
7. Document any limitations or manual fixes

**Example Book Page**:
````markdown
# Python Dictionary Implementation

## Original C Code (cpython/Objects/dictobject.c)

```c
PyObject *
PyDict_GetItem(PyObject *op, PyObject *key)
{
    if (!PyDict_Check(op)) {
        return NULL;
    }
    PyDictObject *mp = (PyDictObject *)op;

    Py_hash_t hash;
    if (PyUnicode_CheckExact(key)) {
        hash = ((PyASCIIObject *)key)->hash;
        if (hash == -1)
            hash = PyObject_Hash(key);
    }
    else {
        hash = PyObject_Hash(key);
        if (hash == -1)
            return NULL;
    }

    return _PyDict_GetItem_KnownHash(mp, key, hash);
}
```

## Transpiled Rust Code

```rust
pub fn py_dict_get_item(
    op: &PyObject,
    key: &PyObject,
) -> Option<&PyObject> {
    // Check if op is a dict
    let mp = op.as_dict()?;

    // Compute hash
    let hash = if let Some(unicode) = key.as_unicode() {
        match unicode.hash() {
            -1 => py_object_hash(key)?,
            h => h,
        }
    } else {
        py_object_hash(key)?
    };

    // Lookup in dict
    py_dict_get_item_known_hash(mp, key, hash)
}
```

## Verification Results

✅ **Compilation**: `cargo build --release`
✅ **Linting**: `cargo clippy -- -D warnings` (0 warnings)
✅ **Tests**: `cargo test` (100% pass)
✅ **Behavior**: Matches CPython dict lookup semantics
✅ **Performance**: 2% slower than C (acceptable)

## Safety Analysis

- **Memory Safety**: ✅ No raw pointers, uses `Option` for nullability
- **Ownership**: ✅ Borrows `&PyObject`, no ownership transfer
- **Lifetimes**: ✅ Inferred correctly, no lifetime errors
- **Panics**: ✅ No panics, uses `Option` for error propagation
````

### 9.3 Book Testing in CI/CD

```bash
# Run in CI pipeline
cd book
mdbook build
mdbook test  # Compiles and runs all Rust code blocks

# Verify all generated Rust passes:
# 1. Compilation
# 2. Clippy linting
# 3. Unit tests
```

**Property Test**:
```rust
proptest! {
    #[test]
    fn book_examples_always_compile(
        c_file in c_project_files()
    ) {
        let rust_code = transpile(&c_file).unwrap();
        let book_page = generate_book_page(&c_file, &rust_code);

        // Verify book page compiles
        prop_assert!(mdbook_test_passes(&book_page));
    }
}
```

---

## 10. MCP Agent Mode (Inspired by depyler agent)

### 10.1 Background Daemon

```rust
// decy-agent/src/lib.rs

pub struct DecyAgent {
    config: AgentConfig,
    mcp_server: McpServer,
    file_watcher: FileWatcher,
}

impl DecyAgent {
    pub async fn start(&self) -> Result<()> {
        // 1. Start MCP server on configured port
        // 2. Watch C project directories for changes
        // 3. Auto-transpile on file save
        // 4. Verify generated Rust code
        // 5. Report results to Claude Code
    }
}
```

### 10.2 MCP Tools

**Available Tools for Claude Code**:

1. **transpile_c_file**: Convert single C file to Rust
2. **transpile_c_directory**: Batch transpilation for entire directories
3. **monitor_c_project**: Set up continuous monitoring
4. **get_transpilation_status**: Query metrics and status
5. **verify_rust_code**: Validate generated Rust code
6. **analyze_c_complexity**: Check C code complexity before transpilation

**Usage in Claude Code**:
```json
{
  "mcpServers": {
    "decy": {
      "command": "decy",
      "args": ["agent", "start", "--foreground"],
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

Claude can then:
- "Transpile this C file to Rust"
- "Monitor my Python source directory"
- "Check if the transpiled Git source compiles"

---

## 11. GitHub Repository Transpilation

### 11.1 Repository-Level Transpilation

**Feature**: Point Decy at a GitHub repository URL and transpile the entire C codebase to Rust

```bash
# CLI usage
decy transpile-repo https://github.com/python/cpython \
    --output ./cpython-rust \
    --verify \
    --generate-book

# Or specify a commit/branch/tag
decy transpile-repo https://github.com/git/git \
    --branch main \
    --commit abc123 \
    --output ./git-rust
```

### 11.2 Repository Analysis Pipeline

```
GitHub Repo URL
    ↓
┌─────────────────────────────┐
│  Stage 1: Repo Cloning      │
│  - git clone with depth     │
│  - Checkout specific commit │
│  - Validate C project       │
└─────────────────────────────┘
    ↓
┌─────────────────────────────┐
│  Stage 2: Project Analysis  │
│  - Detect build system      │
│  - Parse Makefile/CMake     │
│  - Identify C files         │
│  - Analyze dependencies     │
└─────────────────────────────┘
    ↓
┌─────────────────────────────┐
│  Stage 3: Batch Transpile   │
│  - Parallel file processing │
│  - Dependency ordering      │
│  - Module organization      │
└─────────────────────────────┘
    ↓
┌─────────────────────────────┐
│  Stage 4: Project Assembly  │
│  - Generate Cargo.toml      │
│  - Create module tree       │
│  - Setup dependencies       │
│  - Generate tests           │
└─────────────────────────────┘
    ↓
┌─────────────────────────────┐
│  Stage 5: Verification      │
│  - cargo build --release    │
│  - cargo clippy             │
│  - cargo test               │
│  - Generate book            │
└─────────────────────────────┘
    ↓
Rust Project + Book
```

### 11.3 Implementation

```rust
// decy-repo/src/lib.rs

use octocrab::Octocrab;
use git2::Repository;

pub struct RepoTranspiler {
    github: Octocrab,
    config: RepoConfig,
}

pub struct RepoConfig {
    pub output_dir: PathBuf,
    pub verify: bool,
    pub generate_book: bool,
    pub parallel_workers: usize,
    pub preserve_structure: bool,
}

impl RepoTranspiler {
    pub async fn transpile_repository(
        &self,
        repo_url: &str,
        options: RepoOptions,
    ) -> Result<TranspileReport> {
        // 1. Clone repository
        let repo_path = self.clone_repo(repo_url, &options).await?;

        // 2. Analyze project structure
        let project_info = self.analyze_project(&repo_path)?;

        // 3. Detect build system (Makefile, CMake, autotools)
        let build_info = self.detect_build_system(&repo_path)?;

        // 4. Find all C source files
        let c_files = self.find_c_files(&repo_path, &build_info)?;

        // 5. Analyze dependencies between files
        let dep_graph = self.build_dependency_graph(&c_files)?;

        // 6. Transpile in dependency order (parallel where possible)
        let transpiled = self.transpile_files_parallel(&c_files, &dep_graph)?;

        // 7. Generate Rust project structure
        let rust_project = self.generate_project_structure(
            &transpiled,
            &project_info,
            &build_info,
        )?;

        // 8. Verify generated code
        if self.config.verify {
            self.verify_rust_project(&rust_project)?;
        }

        // 9. Generate book documentation
        if self.config.generate_book {
            self.generate_book(&rust_project, &project_info)?;
        }

        Ok(TranspileReport {
            repo_url: repo_url.to_string(),
            files_transpiled: c_files.len(),
            success_rate: self.compute_success_rate(&transpiled),
            unsafe_blocks: self.count_unsafe_blocks(&rust_project),
            verification_status: self.get_verification_status(&rust_project),
        })
    }
}
```

### 11.4 Build System Detection

**Supported build systems**:

1. **Makefile**: Parse to extract sources and dependencies
2. **CMake**: Parse CMakeLists.txt for source files
3. **Autotools**: Parse configure.ac and Makefile.am
4. **Meson**: Parse meson.build
5. **Custom/Manual**: Fallback to directory scanning

```rust
// decy-repo/src/build_system.rs

pub enum BuildSystem {
    Makefile { path: PathBuf },
    CMake { path: PathBuf },
    Autotools { configure: PathBuf, makefile: PathBuf },
    Meson { path: PathBuf },
    Manual,
}

pub struct BuildInfo {
    pub system: BuildSystem,
    pub source_files: Vec<PathBuf>,
    pub include_paths: Vec<PathBuf>,
    pub compiler_flags: Vec<String>,
    pub dependencies: Vec<String>,
}

impl BuildSystemDetector {
    pub fn detect(&self, repo_path: &Path) -> Result<BuildInfo> {
        // Try detection in order of specificity
        if let Ok(info) = self.detect_cmake(repo_path) {
            return Ok(info);
        }
        if let Ok(info) = self.detect_makefile(repo_path) {
            return Ok(info);
        }
        if let Ok(info) = self.detect_autotools(repo_path) {
            return Ok(info);
        }
        if let Ok(info) = self.detect_meson(repo_path) {
            return Ok(info);
        }

        // Fallback: manual scanning
        Ok(self.manual_scan(repo_path)?)
    }
}
```

### 11.5 Parallel Transpilation

**Strategy**: Transpile files in parallel respecting dependencies

```rust
// decy-repo/src/parallel.rs

use rayon::prelude::*;

pub struct ParallelTranspiler {
    workers: usize,
    dep_graph: DependencyGraph,
}

impl ParallelTranspiler {
    pub fn transpile_parallel(
        &self,
        files: &[PathBuf],
    ) -> Result<Vec<TranspiledFile>> {
        // 1. Topological sort by dependencies
        let ordered = self.dep_graph.topological_sort(files)?;

        // 2. Group into independent batches (can be parallelized)
        let batches = self.create_parallel_batches(&ordered);

        // 3. Process each batch in parallel
        let mut results = Vec::new();
        for batch in batches {
            let batch_results: Vec<_> = batch.par_iter()
                .map(|file| self.transpile_file(file))
                .collect::<Result<Vec<_>>>()?;

            results.extend(batch_results);
        }

        Ok(results)
    }
}
```

### 11.6 Cargo Project Generation

**Generate complete Rust project with Cargo.toml**:

```rust
// decy-repo/src/cargo_gen.rs

pub struct CargoProjectGenerator {
    project_info: ProjectInfo,
}

impl CargoProjectGenerator {
    pub fn generate(&self, transpiled: &[TranspiledFile]) -> Result<CargoProject> {
        // 1. Generate Cargo.toml
        let cargo_toml = self.generate_cargo_toml()?;

        // 2. Organize into modules (src/lib.rs, src/main.rs)
        let module_tree = self.organize_modules(transpiled)?;

        // 3. Map C dependencies to Rust crates
        let dependencies = self.map_dependencies()?;

        // 4. Generate build.rs if needed (FFI bindings)
        let build_script = self.generate_build_script()?;

        // 5. Generate tests from C tests
        let tests = self.generate_tests()?;

        Ok(CargoProject {
            cargo_toml,
            module_tree,
            dependencies,
            build_script,
            tests,
        })
    }

    fn generate_cargo_toml(&self) -> Result<String> {
        let mut toml = String::new();
        toml.push_str(&format!(r#"
[package]
name = "{}"
version = "{}"
edition = "2021"
authors = ["Decy Transpiler"]

[dependencies]
# Auto-detected dependencies from C code
"#, self.project_info.name, self.project_info.version));

        // Add dependencies based on C library usage
        for dep in &self.project_info.c_dependencies {
            if let Some(rust_crate) = self.map_c_to_rust_dep(dep) {
                toml.push_str(&format!("{} = \"{}\"\n", rust_crate.name, rust_crate.version));
            }
        }

        Ok(toml)
    }
}
```

### 11.7 Example: Transpiling Python's CPython

```bash
# Transpile Python interpreter source to Rust
decy transpile-repo https://github.com/python/cpython \
    --output ./cpython-rust \
    --branch main \
    --verify \
    --generate-book \
    --parallel 8

# Output:
# 📦 Cloning repository: python/cpython
# ✅ Cloned to /tmp/decy-cpython-abc123
# 🔍 Analyzing project structure...
#    - Build system: Autotools + Makefile
#    - Source files: 487 C files
#    - Include paths: 23 directories
# 📊 Building dependency graph...
#    - 487 nodes, 1,234 edges
# 🚀 Transpiling files (8 workers)...
#    [████████████████████████████████] 487/487 (100%)
#    - Success: 481 files (98.8%)
#    - Warnings: 6 files (1.2%)
#    - Errors: 0 files
# 📝 Generating Cargo project...
#    - Cargo.toml: ✅
#    - Module tree: 487 modules
#    - Dependencies: 12 crates
# 🧪 Verifying generated code...
#    - cargo build: ✅ Success
#    - cargo clippy: ✅ 0 warnings
#    - cargo test: ✅ 1,234 tests passed
# 📚 Generating book...
#    - 487 verification pages
#    - Book built at cpython-rust/book/
# ✅ Transpilation complete!
#
# Report:
#   Files transpiled: 487
#   Success rate: 98.8%
#   Unsafe blocks: 23 (<5 per 1000 LOC)
#   Test coverage: 82.4%
#   Quality grade: A (96/100)
#
# Output directory: ./cpython-rust/
```

### 11.8 Generated Project Structure

```
cpython-rust/
├── Cargo.toml              # Generated project config
├── Cargo.lock              # Generated after first build
├── src/
│   ├── lib.rs              # Main library entry
│   ├── main.rs             # Optional CLI entry
│   ├── objects/            # Transpiled from Objects/
│   │   ├── mod.rs
│   │   ├── dict.rs         # dictobject.c → dict.rs
│   │   ├── list.rs         # listobject.c → list.rs
│   │   └── ...
│   ├── parser/             # Transpiled from Parser/
│   │   └── ...
│   └── python/             # Transpiled from Python/
│       └── ...
├── tests/
│   ├── dict_tests.rs       # Generated from C tests
│   └── ...
├── benches/
│   └── performance.rs      # Performance benchmarks
├── book/                   # Verification book
│   ├── book.toml
│   └── src/
│       ├── SUMMARY.md
│       ├── objects/
│       │   ├── dict.md     # Verification for dict.rs
│       │   └── ...
│       └── ...
└── README.md               # Generated docs
```

### 11.9 MCP Tool for Repository Transpilation

```rust
// decy-mcp/src/tools/transpile_repo.rs

pub struct TranspileRepoTool;

impl McpTool for TranspileRepoTool {
    fn name(&self) -> &str {
        "transpile_github_repo"
    }

    fn description(&self) -> &str {
        "Transpile an entire GitHub C repository to Rust"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "repo_url": {
                    "type": "string",
                    "description": "GitHub repository URL (e.g., https://github.com/python/cpython)"
                },
                "branch": {
                    "type": "string",
                    "description": "Branch name (default: main)"
                },
                "output_path": {
                    "type": "string",
                    "description": "Output directory for transpiled Rust code"
                },
                "verify": {
                    "type": "boolean",
                    "description": "Verify generated code compiles and lints (default: true)"
                },
                "generate_book": {
                    "type": "boolean",
                    "description": "Generate verification book (default: true)"
                }
            },
            "required": ["repo_url", "output_path"]
        })
    }

    async fn execute(&self, params: Value) -> Result<ToolResult> {
        let repo_url = params["repo_url"].as_str().unwrap();
        let output = params["output_path"].as_str().unwrap();

        // Transpile repository
        let transpiler = RepoTranspiler::new()?;
        let report = transpiler.transpile_repository(repo_url, options).await?;

        Ok(ToolResult {
            content: vec![ContentBlock::text(&format!(
                "✅ Successfully transpiled {}\n\
                 Files: {}\n\
                 Success rate: {:.1}%\n\
                 Unsafe blocks: {} (<5 per 1000 LOC)\n\
                 Output: {}",
                repo_url,
                report.files_transpiled,
                report.success_rate * 100.0,
                report.unsafe_blocks,
                output
            ))],
            metadata: Some(json!(report)),
        })
    }
}
```

**Usage in Claude Code**:

```
User: "Transpile the Git source code to Rust"

Claude: I'll transpile the Git repository to Rust using Decy.
<uses transpile_github_repo tool>
{
  "repo_url": "https://github.com/git/git",
  "output_path": "./git-rust",
  "verify": true,
  "generate_book": true
}

<result>
✅ Successfully transpiled https://github.com/git/git
Files: 234
Success rate: 97.4%
Unsafe blocks: 18 (<5 per 1000 LOC)
Output: ./git-rust
</result>

The Git source has been transpiled to Rust! Here's what was generated:
- 234 C files → 234 Rust modules
- Full Cargo project with dependencies
- Verification book at ./git-rust/book/
- All code passes clippy with 0 warnings
- 97.4% of files transpiled successfully
```

### 11.10 Repository Transpilation Tests

```rust
// tests/integration/repo_transpilation.rs

#[tokio::test]
async fn test_transpile_small_c_repo() {
    let transpiler = RepoTranspiler::new().unwrap();

    let report = transpiler.transpile_repository(
        "https://github.com/clibs/buffer",  // Small C library
        RepoOptions {
            output_dir: temp_dir(),
            verify: true,
            generate_book: false,
        },
    ).await.unwrap();

    assert!(report.success_rate > 0.95);
    assert!(report.unsafe_blocks <= 5 * (report.total_loc / 1000));
}

#[tokio::test]
#[ignore = "Long-running test"]
async fn test_transpile_cpython() {
    let transpiler = RepoTranspiler::new().unwrap();

    let report = transpiler.transpile_repository(
        "https://github.com/python/cpython",
        RepoOptions {
            output_dir: temp_dir(),
            verify: true,
            generate_book: true,
        },
    ).await.unwrap();

    // CPython has ~500 C files
    assert!(report.files_transpiled > 400);
    assert!(report.success_rate > 0.90);
}

proptest! {
    #[test]
    fn transpiled_repos_always_compile(
        repo_url in valid_c_repo_urls()
    ) {
        let transpiler = RepoTranspiler::new().unwrap();
        let report = transpiler.transpile_repository(&repo_url, default_options()).await.unwrap();

        // Property: Generated Rust project compiles
        let output_dir = report.output_dir;
        prop_assert!(Command::new("cargo")
            .args(&["build", "--release"])
            .current_dir(output_dir)
            .status()
            .unwrap()
            .success());
    }
}
```

### 11.11 Quality Metrics for Repository Transpilation

**Track per-repository**:

| Metric | Target | Enforcement |
|--------|--------|-------------|
| Success Rate | ≥95% | CI/CD fails if <95% |
| Unsafe Block Rate | ≤5 per 1000 LOC | Quality gate |
| Compilation Success | 100% | Must compile |
| Clippy Pass | 100% | 0 warnings |
| Test Coverage | ≥80% | Pre-commit hook |

---

## 12. Roadmap & Sprint Planning (PMAT-Qualified)

### 11.1 Development Roadmap (20 Sprints)

**Sprint Structure**:
- Each sprint: 2 weeks
- Ticket-based development only
- RED-GREEN-REFACTOR per ticket
- Atomic commits per ticket
- Quality gates enforced

**Roadmap File**: `roadmap.yaml` (YAML format like paiml-mcp-agent-toolkit)

```yaml
# decy/roadmap.yaml

meta:
  project: Decy - C-to-Rust Transpiler
  approach: Extreme Test-Driven Development
  quality_gates:
    max_complexity: 10
    max_cognitive: 15
    min_coverage: 0.80
    min_mutation_score: 0.90
    satd_tolerance: 0
    min_property_tests: 100
  execution:
    ticket_workflow: RED-GREEN-REFACTOR
    commit_strategy: atomic_per_ticket
    build_verification: mandatory_clean

sprints:
  # SPRINT 1: Foundation & C Parser (Week 1-2)
  - id: sprint-1
    name: "C Parser Foundation"
    goal: "Clang-based C AST parsing with 80% coverage"
    duration: 2_weeks
    tickets:
      - id: DECY-001
        title: "Setup clang-sys integration"
        priority: critical
        requirements:
          - "clang-sys dependency added"
          - "Translation unit parsing works"
          - "AST traversal implemented"
        tests:
          - "test_clang_parses_simple_c"
          - "test_clang_handles_includes"
          - "proptest_clang_parsing_deterministic"
        acceptance:
          - "Can parse basic C files"
          - "Tests at 80%+ coverage"
          - "Linting passes"

      - id: DECY-002
        title: "C type system extraction"
        priority: critical
        requirements:
          - "Extract struct/union/enum definitions"
          - "Handle typedef aliases"
          - "Parse function signatures"
        tests:
          - "test_struct_extraction"
          - "test_union_extraction"
          - "test_enum_extraction"
          - "proptest_type_extraction_complete"
        acceptance:
          - "All C types extracted"
          - "80%+ coverage"
          - "Property tests pass"

      - id: DECY-003
        title: "Macro expansion handling"
        priority: high
        requirements:
          - "Expand #define constants"
          - "Expand function-like macros"
          - "Handle conditional compilation"
        tests:
          - "test_constant_macro_expansion"
          - "test_function_macro_expansion"
          - "test_ifdef_handling"
          - "proptest_macro_expansion_correct"
        acceptance:
          - "Macros expanded correctly"
          - "80%+ coverage"
          - "Deterministic expansion"

  # SPRINT 2: HIR Generation (Week 3-4)
  - id: sprint-2
    name: "HIR Generation"
    goal: "Convert C AST to Rust-oriented HIR"
    duration: 2_weeks
    tickets:
      - id: DECY-004
        title: "HIR data structures"
        priority: critical
        requirements:
          - "Define HirModule, HirFunction, HirType"
          - "Implement HIR node constructors"
          - "Add serialization/deserialization"
        tests:
          - "test_hir_module_construction"
          - "test_hir_function_creation"
          - "test_hir_type_conversions"
          - "proptest_hir_round_trip"
        acceptance:
          - "HIR types complete"
          - "80%+ coverage"
          - "Serialization works"

      - id: DECY-005
        title: "C to HIR conversion"
        priority: critical
        requirements:
          - "Convert C types to HIR types"
          - "Convert C functions to HIR functions"
          - "Handle C statements/expressions"
        tests:
          - "test_type_conversion_primitive"
          - "test_type_conversion_pointer"
          - "test_function_conversion"
          - "proptest_conversion_deterministic"
        acceptance:
          - "C→HIR conversion working"
          - "80%+ coverage"
          - "Property tests pass"

  # SPRINT 3: Ownership Analysis (Week 5-6)
  - id: sprint-3
    name: "Ownership Inference"
    goal: "Infer Rust ownership from C pointers"
    duration: 2_weeks
    tickets:
      - id: DECY-006
        title: "Build ownership graph"
        priority: critical
        requirements:
          - "Track pointer allocations (malloc/free)"
          - "Track pointer assignments"
          - "Track pointer dereferencing"
        tests:
          - "test_ownership_graph_construction"
          - "test_malloc_free_tracking"
          - "proptest_graph_invariants"
        acceptance:
          - "Ownership graph builds"
          - "90%+ coverage (critical)"
          - "Graph correct"

      - id: DECY-007
        title: "Infer ownership patterns"
        priority: critical
        requirements:
          - "Detect Box<T> opportunities"
          - "Detect &T vs &mut T"
          - "Detect Vec<T> usage"
        tests:
          - "test_box_inference"
          - "test_reference_inference"
          - "test_vec_inference"
          - "proptest_ownership_sound"
        acceptance:
          - "Ownership inferred correctly"
          - "90%+ coverage"
          - "No use-after-free possible"

  # SPRINT 4: Lifetime Inference (Week 7-8)
  - id: sprint-4
    name: "Lifetime Inference"
    goal: "Infer Rust lifetimes from C scopes"
    duration: 2_weeks
    tickets:
      - id: DECY-008
        title: "Lifetime analysis algorithm"
        priority: critical
        requirements:
          - "Compute variable scopes"
          - "Track pointer aliasing"
          - "Infer lifetime annotations"
        tests:
          - "test_scope_computation"
          - "test_aliasing_detection"
          - "test_lifetime_inference"
          - "proptest_lifetime_soundness"
        acceptance:
          - "Lifetimes inferred"
          - "90%+ coverage"
          - "Borrow checker passes"

  # SPRINT 5: Code Generation (Week 9-10)
  - id: sprint-5
    name: "Rust Code Generation"
    goal: "Generate idiomatic, linted Rust code"
    duration: 2_weeks
    tickets:
      - id: DECY-009
        title: "Basic Rust codegen"
        priority: critical
        requirements:
          - "Generate Rust types from HIR"
          - "Generate Rust functions from HIR"
          - "Apply rustfmt"
        tests:
          - "test_type_codegen"
          - "test_function_codegen"
          - "test_rustfmt_applied"
          - "proptest_codegen_compiles"
        acceptance:
          - "Rust code generates"
          - "85%+ coverage"
          - "Code compiles"

      - id: DECY-010
        title: "Clippy compliance"
        priority: critical
        requirements:
          - "Generated code passes clippy"
          - "Fix common clippy warnings"
          - "Document exceptions"
        tests:
          - "test_clippy_passes"
          - "test_no_clippy_warnings"
          - "proptest_always_lints_clean"
        acceptance:
          - "100% clippy pass rate"
          - "85%+ coverage"
          - "Zero warnings"

  # SPRINT 6-10: Real Project Transpilation
  # - Python source code (cpython)
  # - Git source code
  # - NumPy source code
  # - SQLite source code
  # - Book verification for each

  # SPRINT 11-15: Advanced Features
  # - Thread safety analysis
  # - Async/await generation
  # - Performance optimization
  # - FFI boundary generation
  # - Macro generation (proc macros)

  # SPRINT 16-20: Production Hardening
  # - MCP agent mode
  # - Claude Code integration
  # - Documentation generation
  # - Error message improvement
  # - Performance benchmarking
```

### 11.2 Ticket Template

**Every ticket follows this structure**:

```yaml
- id: DECY-XXX
  title: "Descriptive ticket title"
  priority: critical|high|medium|low
  requirements:
    - "Specific requirement 1"
    - "Specific requirement 2"
    - "Specific requirement 3"
  tests:
    - "test_unit_test_name"
    - "test_another_unit_test"
    - "proptest_property_test_name"
    - "integration_test_name"
  acceptance:
    - "Acceptance criterion 1"
    - "Coverage ≥80% (or ≥90% for critical modules)"
    - "All tests pass"
    - "Linting passes"
    - "Quality gates pass"
```

### 11.3 Commit Strategy

**One atomic commit per ticket**:

```bash
# RED commit
git commit -m "[RED] DECY-XXX: Add failing tests for <feature>"

# GREEN commit
git commit -m "[GREEN] DECY-XXX: Implement <feature> to pass tests"

# REFACTOR commit
git commit -m "[REFACTOR] DECY-XXX: Refactor <feature> to meet quality gates"

# Final commit (squashed)
git commit -m "DECY-XXX: <Feature title>

- Implemented <requirement 1>
- Implemented <requirement 2>
- Added <N> tests (unit, property, integration)
- Coverage: <X>% (target: ≥80%)
- Mutation score: <Y>% (target: ≥90%)
- Quality grade: A+ (98/100)

Closes #XXX"
```

---

## 12. Quality Gate Scripts

### 12.1 Pre-commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

set -e

echo "🔒 Running quality gates..."

# 1. Check coverage
echo "📊 Checking test coverage..."
COVERAGE=$(cargo llvm-cov --quiet | grep "TOTAL" | awk '{print $10}' | tr -d '%')
if (( $(echo "$COVERAGE < 80" | bc -l) )); then
    echo "❌ Coverage $COVERAGE% < 80% - commit blocked"
    exit 1
fi

# 2. Check linting
echo "🔍 Running clippy..."
if ! cargo clippy -- -D warnings; then
    echo "❌ Clippy failed - commit blocked"
    exit 1
fi

# 3. Check formatting
echo "✨ Checking formatting..."
if ! cargo fmt -- --check; then
    echo "❌ Code not formatted - run 'cargo fmt'"
    exit 1
fi

# 4. Check SATD
echo "🚫 Checking for technical debt..."
if git diff --cached | grep -E "TODO|FIXME|HACK|XXX"; then
    echo "❌ SATD comments detected - commit blocked"
    exit 1
fi

# 5. Check tests pass
echo "🧪 Running tests..."
if ! cargo test --all-features; then
    echo "❌ Tests failed - commit blocked"
    exit 1
fi

# 6. Check complexity
echo "📈 Checking complexity..."
if ! cargo run --bin quality-check -- --max-complexity 10; then
    echo "❌ Complexity violations - commit blocked"
    exit 1
fi

echo "✅ All quality gates passed!"
exit 0
```

### 12.2 CI/CD Pipeline

```yaml
# .github/workflows/quality.yml

name: Quality Gates

on: [push, pull_request]

jobs:
  quality:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          components: clippy, rustfmt

      - name: Install tools
        run: |
          cargo install cargo-llvm-cov
          cargo install cargo-mutants

      - name: Run tests
        run: cargo test --all-features

      - name: Check coverage
        run: |
          cargo llvm-cov --lcov --output-path coverage.lcov
          COVERAGE=$(cargo llvm-cov --quiet | grep TOTAL | awk '{print $10}' | tr -d '%')
          if (( $(echo "$COVERAGE < 80" | bc -l) )); then
            echo "Coverage $COVERAGE% < 80%"
            exit 1
          fi

      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          files: coverage.lcov

      - name: Run clippy
        run: cargo clippy --all-features -- -D warnings

      - name: Check formatting
        run: cargo fmt -- --check

      - name: Check SATD
        run: |
          if git diff origin/main | grep -E "TODO|FIXME|HACK|XXX"; then
            echo "SATD comments detected"
            exit 1
          fi

      - name: Run mutation tests
        run: |
          cargo mutants --in-diff origin/main --test-timeout 120
          # Check mutation score ≥90%

      - name: Build book
        run: |
          cd book
          mdbook build
          mdbook test

      - name: Quality score
        run: |
          cargo run --bin quality-score
          # Verify score ≥98 (A+)
```

---

## 13. Metrics & Tracking

### 13.1 Per-Ticket Metrics

Track for every ticket:
- **Test Coverage**: Module-specific coverage %
- **Mutation Score**: % of mutants killed
- **Complexity**: Max cyclomatic/cognitive complexity
- **Property Tests**: Number of property tests added
- **LOC**: Lines of code added/removed
- **Quality Grade**: A+ / A / B+ / B score

### 13.2 Per-Sprint Metrics

Track for every sprint:
- **Velocity**: Tickets completed
- **Defect Rate**: Bugs found / tickets completed
- **Technical Debt**: SATD comments added (should be 0)
- **Test Suite Runtime**: Time to run all tests
- **Coverage Trend**: Coverage % over time
- **Mutation Score Trend**: % over time

### 13.3 Project-Level Metrics

Track continuously:
- **Overall Coverage**: 80%+ enforced
- **Mutation Score**: ≥90% target
- **Quality Grade**: A+ (98+) target
- **Transpilation Success Rate**: % of C files transpiled successfully
- **Clippy Pass Rate**: 100% required
- **Book Verification Pass Rate**: 100% required

**Dashboard** (optional):
- Real-time metrics displayed in terminal
- Web dashboard for CI/CD
- MCP tool for Claude Code to query metrics

---

## 14. Real-World Test Projects

### 14.1 Target Projects

**Python Source Code (cpython)**:
- Files: `Objects/dictobject.c`, `Objects/listobject.c`, etc.
- Challenge: Complex macros, ref counting, GC
- Goal: Transpile to safe Rust with `Arc<>` for ref counting

**Git Source Code**:
- Files: `refs.c`, `object.c`, `packfile.c`
- Challenge: Heavy pointer usage, file I/O
- Goal: Transpile with safe file operations, `Vec<u8>` for buffers

**NumPy Source Code**:
- Files: `array.c`, `ufunc.c`, `linalg.c`
- Challenge: Multi-dimensional arrays, BLAS/LAPACK calls
- Goal: Transpile with `ndarray` crate integration

**SQLite Source Code**:
- Files: `btree.c`, `vdbe.c`, `pager.c`
- Challenge: Low-level data structures, B-tree implementation
- Goal: Transpile to safe Rust with zero-copy where possible

### 14.2 Success Criteria

For each project:
- ✅ **Transpilation Completes**: 100% of files transpile without errors
- ✅ **Compiles**: `cargo build --release` succeeds
- ✅ **Lints Clean**: `cargo clippy -- -D warnings` passes (0 warnings)
- ✅ **Tests Pass**: Generated Rust tests pass (behavior equivalence)
- ✅ **Performance**: Within 20% of original C performance
- ✅ **Book Verified**: All transpiled code documented and verified in mdBook

---

## 15. Implementation Phases

### Phase 1: Foundation (Sprints 1-5, 10 weeks)
- C parser with clang integration
- HIR generation
- Ownership inference
- Lifetime inference
- Basic Rust code generation
- **Deliverable**: Can transpile simple C programs with 80%+ coverage

### Phase 2: Real Projects (Sprints 6-10, 10 weeks)
- Transpile Python dict/list implementations
- Transpile Git refs/objects
- Transpile NumPy arrays
- Transpile SQLite B-tree
- **Deliverable**: Real-world C code transpiles with book verification

### Phase 3: Advanced Features (Sprints 11-15, 10 weeks)
- Thread safety analysis
- Async/await generation
- Performance optimization passes
- FFI boundary generation
- Macro generation (proc macros)
- **Deliverable**: Production-grade transpiler with advanced features

### Phase 4: Production Hardening (Sprints 16-20, 10 weeks)
- MCP agent mode for background transpilation
- Claude Code integration
- Documentation generation from C comments
- Error message improvement
- Comprehensive benchmarking suite
- **Deliverable**: Production-ready transpiler with MCP support

---

## 16. Documentation Requirements

### 16.1 Code Documentation

**Every public item must have**:
- `///` doc comment
- Usage example in doctest
- Safety documentation for unsafe code
- Panic documentation for `unwrap()`/`expect()`

### 16.2 Architecture Documentation

**Required docs**:
- `docs/architecture/README.md`: Overview
- `docs/architecture/decisions/`: ADRs for major decisions
- `docs/architecture/diagrams/`: System diagrams (mermaid)

### 16.3 Quality Documentation

**Required docs**:
- `docs/quality/standards.md`: Quality standards (like bashrs)
- `docs/quality/gates.md`: Quality gate descriptions
- `docs/quality/metrics.md`: Metrics tracking

### 16.4 Execution Documentation

**Required docs**:
- `docs/execution/roadmap.md`: Current sprint status
- `docs/execution/quality-gates.md`: Gate enforcement status
- `CHANGELOG.md`: All changes documented

---

## 17. Success Metrics

### 17.1 Technical Metrics

- **Coverage**: ≥80% at ALL times (enforced)
- **Mutation Score**: ≥90% (target)
- **Quality Grade**: A+ (98+) (target)
- **Clippy Pass Rate**: 100% (required)
- **Transpilation Success**: ≥95% of C files transpile
- **Behavior Equivalence**: ≥99% of tests pass

### 17.2 Performance Metrics

- **Transpilation Speed**: <1 second per 1000 LOC
- **Memory Usage**: <500MB for large projects
- **Generated Code Performance**: Within 20% of C

### 17.3 Quality Metrics

- **SATD Comments**: 0 (zero tolerance)
- **Unsafe Blocks**: ≤5 per 1000 LOC generated
- **Complexity**: ≤10 cyclomatic, ≤15 cognitive
- **Test Count**: 1000+ tests (unit, property, integration)

---

## 18. Risks & Mitigations

### 18.1 Technical Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| C macros too complex | High | High | Start with simple macros, document limitations |
| Ownership inference inaccurate | Medium | Critical | Extensive property testing, manual review |
| Performance regression | Medium | Medium | Benchmark suite, optimization passes |
| Unsafe code proliferation | Low | High | Strict unsafe minimization policy |

### 18.2 Process Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Coverage drops below 80% | Low | Critical | Pre-commit hooks block, CI/CD enforces |
| SATD comments accumulate | Low | Medium | Zero tolerance policy, automated detection |
| Technical debt increases | Low | Medium | Regular refactoring sprints, quality reviews |

---

## 19. Appendix A: Tool Dependencies

### 19.1 Core Dependencies

- **clang-sys**: C AST parsing
- **syn**: Rust AST manipulation (for codegen testing)
- **quote**: Rust code generation helpers
- **proptest**: Property-based testing
- **cargo-mutants**: Mutation testing
- **cargo-llvm-cov**: Fast coverage generation
- **mdbook**: Book-based verification

### 19.2 Quality Tools

- **clippy**: Rust linting
- **rustfmt**: Code formatting
- **cargo-deny**: Dependency auditing
- **cargo-audit**: Security auditing

---

## 20. Appendix B: References

### 20.1 Inspiration Projects

- **depyler**: Python→Rust transpiler with book verification
  - Lessons: HIR design, book-based testing, MCP agent mode
- **bashrs**: Rust→Shell transpiler with EXTREME TDD
  - Lessons: Quality gates, mutation testing, property tests
- **paiml-mcp-agent-toolkit**: Quality framework
  - Lessons: PMAT configuration, roadmap structure, Toyota Way

### 20.2 Related Research

- **C2Rust**: Mozilla's C→Rust transpiler (inspiration for unsafe minimization)
- **Corrode**: Haskell-based C→Rust transpiler (ownership inference ideas)
- **Rust Language Server**: Reference for Rust code generation

---

## Conclusion

Decy is a production-grade C-to-Rust transpiler built with EXTREME TDD methodology, ensuring 80%+ coverage, ≥90% mutation score, and 100% linting pass rate at all times. By combining techniques from depyler (book verification), bashrs (quality gates), and paiml-mcp-agent-toolkit (PMAT qualification), Decy provides a rigorous, test-driven approach to safely transpiling legacy C codebases to modern, safe Rust.

**Development starts with roadmap-driven, ticket-only development following RED-GREEN-REFACTOR methodology, with quality gates enforced at every commit.**

---

**Document Version**: 1.0
**Last Updated**: 2025-10-10
**Status**: SPECIFICATION COMPLETE - Ready for Implementation
