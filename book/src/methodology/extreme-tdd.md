# EXTREME TDD Methodology

DECY follows **EXTREME Test-Driven Development** - a rigorous approach to software quality that goes beyond traditional TDD.

## The RED-GREEN-REFACTOR Cycle

Every feature in DECY follows this cycle:

### 🔴 RED: Write Failing Tests First

Write tests BEFORE writing any production code:

```rust,ignore
#[test]
fn test_transpile_simple_function() {
    let c_code = "int add(int a, int b) { return a + b; }";
    let rust_code = transpile(c_code).unwrap();

    assert!(rust_code.contains("fn add"));
    assert!(rust_code.contains("i32"));
}
```

This test will FAIL because `transpile()` doesn't exist yet.

### 🟢 GREEN: Write Minimal Code to Pass

Write just enough code to make the test pass:

```rust
pub fn transpile(c_code: &str) -> Result<String> {
    // Minimal implementation
    let parser = CParser::new()?;
    let ast = parser.parse(c_code)?;
    let hir = HirFunction::from_ast_function(&ast.functions()[0]);
    let codegen = CodeGenerator::new();
    Ok(codegen.generate_function(&hir))
}
```

Run `cargo test` - the test should now PASS ✅

### ♻️ REFACTOR: Improve While Keeping Green

Now refactor for quality while keeping tests green:

```rust
pub fn transpile(c_code: &str) -> Result<String> {
    // Step 1: Parse C code
    let parser = CParser::new()
        .context("Failed to create C parser")?;
    let ast = parser.parse(c_code)
        .context("Failed to parse C code")?;

    // Step 2: Convert to HIR
    let hir_functions: Vec<HirFunction> = ast
        .functions()
        .iter()
        .map(HirFunction::from_ast_function)
        .collect();

    // Step 3: Generate Rust code
    let code_generator = CodeGenerator::new();
    let mut rust_code = String::new();

    for func in &hir_functions {
        let generated = code_generator.generate_function(func);
        rust_code.push_str(&generated);
        rust_code.push('\n');
    }

    Ok(rust_code)
}
```

All tests still pass ✅ but code is cleaner!

## Quality Requirements

EXTREME TDD enforces strict quality gates:

### ✅ Coverage: ≥80% at ALL Times

```bash
cargo llvm-cov --lcov --output-path coverage.lcov
# Coverage must be ≥80% or commit is BLOCKED
```

**Example from DECY:**

```
decy-parser/     : 87.3% coverage ✅
decy-hir/        : 91.2% coverage ✅
decy-ownership/  : 93.4% coverage ✅
decy-codegen/    : 94.1% coverage ✅
──────────────────────────────────────
Overall          : 93.37% coverage ✅
```

### ✅ Mutation Testing: ≥90% Kill Rate

Mutation testing introduces bugs into your code to verify tests catch them:

```bash
cargo mutants --in-diff origin/main
# ≥90% of mutants must be caught
```

**Example mutation:**

```rust
// Original code
fn add(a: i32, b: i32) -> i32 {
    a + b  // ← Mutation: change + to -
}

// Mutant version
fn add(a: i32, b: i32) -> i32 {
    a - b  // ← If tests don't fail, BAD TESTS!
}
```

Good tests will catch this mutant:

```rust
#[test]
fn test_add() {
    assert_eq!(add(2, 3), 5);  // Fails with mutant (2-3 = -1) ✅
}
```

### ✅ Clippy: Zero Warnings

```bash
cargo clippy --all-targets -- -D warnings
# ANY warning = build fails
```

**Example clippy fix:**

```rust
// ❌ Clippy warning: using len() == 0
if items.len() == 0 {
    return None;
}

// ✅ Fixed: use is_empty()
if items.is_empty() {
    return None;
}
```

### ✅ Property Testing: 100+ Properties

Test invariants with randomized inputs:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_transpilation_deterministic(c_code in c_code_generator()) {
        let output1 = transpile(&c_code).unwrap();
        let output2 = transpile(&c_code).unwrap();

        // Property: Same input → same output
        prop_assert_eq!(output1, output2);
    }

    #[test]
    fn prop_generated_rust_compiles(c_code in valid_c_code()) {
        let rust_code = transpile(&c_code).unwrap();

        // Property: Generated code must compile
        prop_assert!(compile_rust(&rust_code).is_ok());
    }

    #[test]
    fn prop_no_unsafe_blocks(c_code in memory_safe_c()) {
        let rust_code = transpile(&c_code).unwrap();

        // Property: Safe C → no unsafe Rust
        prop_assert!(!rust_code.contains("unsafe"));
    }
}
```

These run 1000+ times each with different random inputs!

### ✅ Zero SATD (Technical Debt)

**SATD (Self-Admitted Technical Debt)** is forbidden:

```rust
// ❌ BLOCKED by pre-commit hook
// TODO: implement this later
// FIXME: this is broken
// HACK: workaround for bug

// ✅ Allowed: explain WHY, not WHAT
// Use single lifetime for simplicity - multiple lifetimes require
// dependency analysis which is tracked in DECY-018
```

## Test Types

DECY uses multiple test types for comprehensive coverage:

### 1. Unit Tests

Test individual functions:

```rust
#[test]
fn test_map_type_int() {
    assert_eq!(CodeGenerator::map_type(&HirType::Int), "i32");
}

#[test]
fn test_map_type_pointer() {
    let ptr = HirType::Pointer(Box::new(HirType::Int));
    assert_eq!(CodeGenerator::map_type(&ptr), "*mut i32");
}
```

### 2. Integration Tests

Test complete workflows:

```rust
#[test]
fn test_end_to_end_transpilation() {
    let c_code = r#"
        int calculate(int a, int b) {
            int result = a + b;
            return result;
        }
    "#;

    let rust_code = transpile(c_code).unwrap();

    assert!(rust_code.contains("fn calculate"));
    assert!(rust_code.contains("let mut result"));
    assert!(rust_code.contains("i32"));
}
```

### 3. Property Tests

Test invariants with random inputs:

```rust
proptest! {
    #[test]
    fn prop_parser_never_panics(input in "\\PC*") {
        // Parser should never panic, even with garbage input
        let _ = CParser::new().unwrap().parse(&input);
    }
}
```

### 4. Doc Tests

Test documentation examples:

```rust
/// Transpile C code to Rust.
///
/// # Examples
///
/// ```
/// use decy_core::transpile;
///
/// let c_code = "int add(int a, int b) { return a + b; }";
/// let rust_code = transpile(c_code)?;
/// assert!(rust_code.contains("fn add"));
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn transpile(c_code: &str) -> Result<String> {
    // ...
}
```

Run with `cargo test --doc` - examples MUST work!

### 5. Mutation Tests

Verify test quality by introducing bugs:

```bash
cargo mutants --test-tool=nextest \
              --timeout=120 \
              --in-diff origin/main
```

Example output:

```
Mutation testing results:
  Caught: 234 mutants (94.3%) ✅
  Missed:  14 mutants (5.7%)  ⚠️
  Timeout: 0 mutants
──────────────────────────────────
Kill rate: 94.3% (target: ≥90%) ✅
```

## Commit Strategy

Every commit follows this format:

```
[RED] DECY-XXX: Add failing tests for <feature>
[GREEN] DECY-XXX: Implement <feature> to pass tests
[REFACTOR] DECY-XXX: Refactor <feature> for quality
```

Or squashed:

```
DECY-XXX: <Feature title>

- Implemented <requirement 1>
- Implemented <requirement 2>
- Added N tests (unit, property, integration)
- Coverage: X% (target: ≥80%)
- Mutation score: Y% (target: ≥90%)
- Quality grade: A+ (98/100)

Closes #XXX
```

## Pre-Commit Hooks

Quality gates run BEFORE commit:

```bash
#!/bin/bash
# .git/hooks/pre-commit

# 1. Check coverage ≥80%
cargo llvm-cov || exit 1

# 2. Check linting
cargo clippy -- -D warnings || exit 1

# 3. Check formatting
cargo fmt -- --check || exit 1

# 4. Check SATD
git diff --cached | grep -E "TODO|FIXME|HACK" && exit 1

# 5. Check tests pass
cargo test || exit 1
```

## Example: Adding a Feature

Let's add ownership inference for malloc/free:

### Step 1: RED - Write Failing Test

```rust
#[test]
fn test_malloc_becomes_box() {
    let c_code = "int* p = malloc(sizeof(int));";
    let ownership = infer_ownership(c_code).unwrap();

    assert_eq!(ownership.pattern, OwnershipPattern::Owning);
    assert_eq!(ownership.rust_type, "Box<i32>");
}
```

Run: `cargo test` → ❌ FAILS (function doesn't exist)

### Step 2: GREEN - Minimal Implementation

```rust
pub fn infer_ownership(c_code: &str) -> Result<OwnershipInfo> {
    // Minimal code to pass test
    if c_code.contains("malloc") {
        return Ok(OwnershipInfo {
            pattern: OwnershipPattern::Owning,
            rust_type: "Box<i32>".to_string(),
        });
    }
    Err(anyhow!("Not implemented"))
}
```

Run: `cargo test` → ✅ PASSES

### Step 3: REFACTOR - Add Quality

```rust
pub fn infer_ownership(c_code: &str) -> Result<OwnershipInfo> {
    let parser = CParser::new()?;
    let ast = parser.parse(c_code)?;

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&ast);

    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    Ok(inferences[0].clone())
}
```

Add more tests:

```rust
proptest! {
    #[test]
    fn prop_malloc_always_owning(size in 1..1024usize) {
        let c_code = format!("int* p = malloc({});", size);
        let ownership = infer_ownership(&c_code).unwrap();
        prop_assert_eq!(ownership.pattern, OwnershipPattern::Owning);
    }
}
```

Run: `cargo test` → ✅ ALL PASS

Check coverage: `cargo llvm-cov` → 94.2% ✅

Run clippy: `cargo clippy` → 0 warnings ✅

Commit:

```bash
git commit -m "DECY-012: Ownership inference for malloc/free

- Implemented dataflow-based ownership analysis
- Added detection of malloc → Box<T> pattern
- Added 15 tests (unit, property, integration)
- Coverage: 94.2% (target: ≥80%)
- Mutation score: 92.1% (target: ≥90%)
```

## Benefits of EXTREME TDD

1. **Confidence**: Every feature is proven to work
2. **Regression Prevention**: Tests catch breaking changes
3. **Documentation**: Tests show how to use the API
4. **Refactoring Safety**: Change internals without breaking behavior
5. **Quality**: Enforced by automated gates

## Next Steps

- [Quality Gates](./quality-gates.md) - Learn about the enforcement mechanisms
- [Property Testing](./property-testing.md) - Deep dive into property-based testing
- [Mutation Testing](./mutation-testing.md) - Verify your tests are good
