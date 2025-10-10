# Quality Gates

DECY enforces quality through **automated gates** that block commits unless strict standards are met.

## The Quality Gate Pipeline

Every change must pass these gates BEFORE merging:

```
┌─────────────┐
│  git commit │
└──────┬──────┘
       │
       ▼
┌─────────────────┐
│  Pre-commit     │ ← Runs locally before commit
│  Hooks          │
└──────┬──────────┘
       │
       ▼
┌─────────────────┐
│  CI/CD          │ ← Runs on GitHub Actions
│  Pipeline       │
└──────┬──────────┘
       │
       ▼
┌─────────────────┐
│  Merge to main  │ ← Only if ALL gates pass
└─────────────────┘
```

## Gate 1: Test Coverage ≥80%

### Enforcement

```bash
cargo llvm-cov --lcov --output-path coverage.lcov
```

**Pass Criteria**: Overall coverage ≥80%

### Example Output

```
Filename                      Regions    Missed Regions     Cover
────────────────────────────────────────────────────────────────
decy-parser/src/lib.rs            245                32    86.94%
decy-hir/src/lib.rs               189                17    91.00%
decy-ownership/src/lib.rs         312                21    93.27%
decy-codegen/src/lib.rs           421                25    94.06%
────────────────────────────────────────────────────────────────
TOTAL                            1167                95    91.86% ✅
```

### Coverage Configuration

Create `llvm-cov.toml`:

```toml
[llvm-cov]
target-dir = "target"
html = true
open = false
ignore-filename-regex = [
    "tests/",
    "benches/",
]

[report]
fail-under-lines = 80
fail-under-functions = 80
```

### What Counts as Covered?

```rust
// ✅ COVERED: Line executed by test
pub fn add(a: i32, b: i32) -> i32 {
    a + b  // ← Test calls this
}

// ❌ NOT COVERED: Unreachable code
pub fn unreachable_branch(x: i32) -> i32 {
    if x > 0 {
        return x;  // ← Covered
    }
    panic!("Never happens");  // ← NOT covered (needs test!)
}
```

## Gate 2: Mutation Testing ≥90%

### Enforcement

```bash
cargo mutants --in-diff origin/main --timeout 120
```

**Pass Criteria**: ≥90% of mutants caught by tests

### Example Mutants

```rust
// Original code
fn is_pointer(ty: &HirType) -> bool {
    matches!(ty, HirType::Pointer(_))
}

// Mutant 1: Replace true with false
fn is_pointer(ty: &HirType) -> bool {
    false  // ← Mutant: always return false
}

// Mutant 2: Negate condition
fn is_pointer(ty: &HirType) -> bool {
    !matches!(ty, HirType::Pointer(_))  // ← Mutant: invert logic
}
```

Good tests will catch these:

```rust
#[test]
fn test_is_pointer_detects_pointers() {
    let ptr = HirType::Pointer(Box::new(HirType::Int));
    assert!(is_pointer(&ptr));  // ← Catches Mutant 1 and 2!
}

#[test]
fn test_is_pointer_rejects_non_pointers() {
    assert!(!is_pointer(&HirType::Int));  // ← Catches Mutant 2!
}
```

### Mutation Report

```
Mutation testing results:
  Caught:   234 mutants (94.3%) ✅
  Missed:    14 mutants (5.7%)  ⚠️
  Timeout:    0 mutants
  Unviable:   3 mutants
─────────────────────────────────────
Kill rate: 94.3% (target: ≥90%) ✅
```

### Common Missed Mutants

```rust
// ❌ BAD: Only tests one value
#[test]
fn test_add() {
    assert_eq!(add(0, 0), 0);  // Passes even with "return 0" mutant!
}

// ✅ GOOD: Tests multiple values
#[test]
fn test_add() {
    assert_eq!(add(2, 3), 5);   // Catches "return 0" mutant
    assert_eq!(add(-1, 1), 0);  // Catches "return a" mutant
    assert_eq!(add(10, 5), 15); // Catches "return b" mutant
}
```

## Gate 3: Clippy with Zero Warnings

### Enforcement

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

**Pass Criteria**: Zero warnings, zero errors

### Clippy Configuration

Create `.cargo/config.toml`:

```toml
[target.'cfg(all())']
rustflags = [
    "-D", "warnings",
    "-D", "clippy::all",
    "-D", "clippy::pedantic",
    "-D", "clippy::cargo",
]
```

### Common Clippy Issues

```rust
// ❌ Clippy: unnecessary `return` statement
fn add(a: i32, b: i32) -> i32 {
    return a + b;
}

// ✅ Fixed: implicit return
fn add(a: i32, b: i32) -> i32 {
    a + b
}

// ❌ Clippy: using `len() == 0` instead of `is_empty()`
if vec.len() == 0 {
    return None;
}

// ✅ Fixed: use `is_empty()`
if vec.is_empty() {
    return None;
}

// ❌ Clippy: needless borrow
fn process(s: &String) {
    println!("{}", s);
}

// ✅ Fixed: use string slice
fn process(s: &str) {
    println!("{}", s);
}
```

## Gate 4: Formatting with rustfmt

### Enforcement

```bash
cargo fmt -- --check
```

**Pass Criteria**: No formatting changes needed

### rustfmt Configuration

Create `rustfmt.toml`:

```toml
edition = "2021"
max_width = 100
tab_spaces = 4
newline_style = "Unix"
use_small_heuristics = "Default"
reorder_imports = true
reorder_modules = true
remove_nested_parens = true
format_code_in_doc_comments = true
normalize_comments = true
wrap_comments = true
```

### Auto-fix

```bash
# Fix formatting automatically
cargo fmt
```

## Gate 5: Zero SATD Comments

### Enforcement

```bash
# Pre-commit hook checks for SATD
git diff --cached | grep -E "TODO|FIXME|HACK|XXX" && exit 1
```

**Pass Criteria**: No TODO/FIXME/HACK/XXX comments

### Examples

```rust
// ❌ BLOCKED: Self-Admitted Technical Debt
// TODO: implement this later
// FIXME: this is broken
// HACK: workaround for bug
// XXX: needs refactoring

// ✅ ALLOWED: Explanatory comments
// Use single lifetime for simplicity - multiple lifetimes require
// dependency analysis which is tracked in DECY-018

// ✅ ALLOWED: Documentation
/// Returns the ownership pattern for the given variable.
/// This uses dataflow analysis to determine if the variable
/// is owned, borrowed, or has a static lifetime.
```

## Gate 6: Documentation Coverage

### Enforcement

```bash
cargo doc --no-deps --document-private-items
```

**Pass Criteria**: All public items documented

### Example

```rust
// ❌ Missing documentation
pub fn transpile(c_code: &str) -> Result<String> {
    // ...
}

// ✅ Documented
/// Transpiles C code to Rust.
///
/// # Arguments
///
/// * `c_code` - The C source code to transpile
///
/// # Returns
///
/// Returns the transpiled Rust code or an error if parsing fails.
///
/// # Examples
///
/// ```
/// use decy_core::transpile;
///
/// let c_code = "int add(int a, int b) { return a + b; }";
/// let rust_code = transpile(c_code)?;
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn transpile(c_code: &str) -> Result<String> {
    // ...
}
```

## Gate 7: Doc Tests Pass

### Enforcement

```bash
cargo test --doc
```

**Pass Criteria**: All documentation examples compile and pass

### Example

```rust
/// Parses C code into an AST.
///
/// # Examples
///
/// ```
/// use decy_parser::CParser;
///
/// let parser = CParser::new()?;
/// let ast = parser.parse("int x = 5;")?;
/// assert_eq!(ast.statements().len(), 1);
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn parse(&self, code: &str) -> Result<Ast> {
    // ...
}
```

This example will be compiled and run by `cargo test --doc`!

## Gate 8: Benchmark Performance

### Enforcement

```bash
cargo bench --bench transpile_bench
```

**Pass Criteria**: No regressions >5%

### Example Benchmark

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_transpile(c: &mut Criterion) {
    let c_code = "int add(int a, int b) { return a + b; }";

    c.bench_function("transpile_simple_function", |b| {
        b.iter(|| {
            transpile(black_box(c_code)).unwrap()
        });
    });
}

criterion_group!(benches, benchmark_transpile);
criterion_main!(benches);
```

## Pre-Commit Hook

Create `.git/hooks/pre-commit`:

```bash
#!/bin/bash
set -e

echo "🔍 Running pre-commit quality gates..."

# Gate 1: Format check
echo "  ✓ Checking formatting..."
cargo fmt -- --check || {
    echo "❌ Code is not formatted. Run: cargo fmt"
    exit 1
}

# Gate 2: Clippy
echo "  ✓ Running clippy..."
cargo clippy --all-targets -- -D warnings || {
    echo "❌ Clippy found issues"
    exit 1
}

# Gate 3: SATD check
echo "  ✓ Checking for SATD comments..."
if git diff --cached | grep -E "TODO|FIXME|HACK|XXX"; then
    echo "❌ SATD comments detected (TODO/FIXME/HACK/XXX)"
    exit 1
fi

# Gate 4: Tests
echo "  ✓ Running tests..."
cargo test --quiet || {
    echo "❌ Tests failed"
    exit 1
}

# Gate 5: Coverage (local check, full check in CI)
echo "  ✓ Checking coverage..."
cargo llvm-cov --quiet || {
    echo "❌ Coverage check failed"
    exit 1
}

echo "✅ All pre-commit gates passed!"
```

Make it executable:

```bash
chmod +x .git/hooks/pre-commit
```

## CI/CD Pipeline

GitHub Actions workflow (`.github/workflows/quality-gates.yml`):

```yaml
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
          cargo install cargo-llvm-cov cargo-mutants

      - name: Check formatting
        run: cargo fmt -- --check

      - name: Run clippy
        run: cargo clippy --all-targets -- -D warnings

      - name: Run tests
        run: cargo test --all-features

      - name: Check coverage
        run: |
          cargo llvm-cov --lcov --output-path coverage.lcov
          # Fail if coverage < 80%
          cargo llvm-cov report --fail-under-lines 80

      - name: Run mutation tests
        run: |
          cargo mutants --in-diff origin/main --timeout 120

      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          files: coverage.lcov
```

## Quality Dashboard

Track metrics over time:

```
┌─────────────────────────────────────────────┐
│ DECY Quality Dashboard                      │
├─────────────────────────────────────────────┤
│ Test Coverage:      93.37% ✅ (target: 80%) │
│ Mutation Score:     94.30% ✅ (target: 90%) │
│ Clippy Warnings:         0 ✅               │
│ SATD Comments:           0 ✅               │
│ Doc Coverage:      100.00% ✅               │
│ Build Time:          2m 34s                 │
│ Test Time:           1m 12s                 │
└─────────────────────────────────────────────┘
```

## Summary

Quality gates ensure:

✅ **Coverage ≥80%**: All code paths tested
✅ **Mutation ≥90%**: Tests are effective
✅ **Clippy clean**: Best practices followed
✅ **Formatted**: Consistent style
✅ **Zero SATD**: No technical debt
✅ **Documented**: All APIs explained
✅ **Doc tests pass**: Examples work
✅ **Benchmarks**: No regressions

These gates are **automated** and **enforced** - no exceptions!

## Next Steps

- [Property Testing](./property-testing.md) - Testing invariants
- [Mutation Testing](./mutation-testing.md) - Deep dive into mutation testing
